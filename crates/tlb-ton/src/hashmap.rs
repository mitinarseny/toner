use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use tlb::{
    bits::{
        de::{args::BitUnpackWithArgs, BitReader, BitReaderExt},
        r#as::VarNBits,
    },
    de::{r#as::CellDeserializeAsOwned, CellDeserializeOwned, CellParser, CellParserError},
    r#as::{Ref, Same},
    Cell, Error,
};

use crate::Unary;

/// ```tlb
/// hme_empty$0 {n:#} {X:Type} = HashmapE n X;
/// hme_root$1 {n:#} {X:Type} root:^(Hashmap n X) = HashmapE n X;
/// ```
#[derive(Debug, Clone, Default)]
pub enum HashmapE<T> {
    #[default]
    Empty,
    Root(Hashmap<T>),
}

impl<T> HashmapE<T> {
    pub const fn new() -> Self {
        Self::Empty
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Root(root) => root.len(),
        }
    }

    pub fn get(&self, key: &BitSlice<u8, Msb0>) -> Option<&T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get(key),
        }
    }

    pub fn get_mut(&mut self, key: &BitSlice<u8, Msb0>) -> Option<&mut T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get_mut(key),
        }
    }

    pub fn parse_n_as<'de, As>(
        parser: &mut CellParser<'de>,
        n: u32,
    ) -> Result<Self, CellParserError<'de>>
    where
        As: CellDeserializeAsOwned<T> + ?Sized,
    {
        Ok(match parser.unpack()? {
            false => Self::Empty,
            true => {
                let cell: Cell = parser.parse_as::<_, Ref>()?;
                let mut parser = cell.parser();
                let root = Hashmap::parse_n_as::<As>(&mut parser, n)?;
                parser.ensure_empty()?;
                Self::Root(root)
            }
        })
    }

    pub fn parse_n<'de>(parser: &mut CellParser<'de>, n: u32) -> Result<Self, CellParserError<'de>>
    where
        T: CellDeserializeOwned,
    {
        Self::parse_n_as::<Same>(parser, n)
    }
}

/// ```tlb
/// hm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
/// {n = (~m) + l} node:(HashmapNode m X) = Hashmap n X;
/// ```
#[derive(Debug, Clone)]
pub struct Hashmap<T> {
    label: BitVec<u8, Msb0>,
    node: HashmapNode<T>,
}

impl<T> Hashmap<T> {
    #[inline]
    pub fn prefix(&self) -> &BitSlice<u8, Msb0> {
        &self.label
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.node.len()
    }

    #[inline]
    pub fn get(&self, mut key: &BitSlice<u8, Msb0>) -> Option<&T> {
        key = key.strip_prefix(&self.label)?;
        self.node.get(key)
    }

    #[inline]
    pub fn get_mut(&mut self, mut key: &BitSlice<u8, Msb0>) -> Option<&mut T> {
        key = key.strip_prefix(&self.label)?;
        self.node.get_mut(key)
    }

    pub fn parse_n_as<'de, As>(
        parser: &mut CellParser<'de>,
        n: u32,
    ) -> Result<Self, CellParserError<'de>>
    where
        As: CellDeserializeAsOwned<T> + ?Sized,
    {
        // label:(HmLabel ~l n)
        let label = HmLabel::unpack_with(&mut *parser, n)?;
        // {n = (~m) + l}
        let m = n - label.n;
        Ok(Self {
            label: label.s,
            // node:(HashmapNode m X)
            node: HashmapNode::parse_n_as::<As>(parser, m)?,
        })
    }
}

/// ```tlb
/// hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
/// hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
///            right:^(Hashmap n X) = HashmapNode (n + 1) X;
/// ```
#[derive(Debug, Clone)]
enum HashmapNode<T> {
    Leaf(T),
    /// [left, right]
    Fork([Box<Hashmap<T>>; 2]),
}

impl<T> HashmapNode<T> {
    fn len(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Fork([l, r]) => l.len() + r.len(),
        }
    }

    fn get(&self, key: &BitSlice<u8, Msb0>) -> Option<&T> {
        match self {
            Self::Leaf(v) if key.is_empty() => Some(v),
            Self::Fork([left, right]) => {
                let (is_right, key) = key.split_first()?;
                if *is_right { right } else { left }.get(key)
            }
            _ => None,
        }
    }

    fn get_mut(&mut self, key: &BitSlice<u8, Msb0>) -> Option<&mut T> {
        match self {
            Self::Leaf(v) if key.is_empty() => Some(v),
            Self::Fork([left, right]) => {
                let (is_right, key) = key.split_first()?;
                if *is_right { right } else { left }.get_mut(key)
            }
            _ => None,
        }
    }

    fn parse_n_as<'de, As>(
        parser: &mut CellParser<'de>,
        n: u32,
    ) -> Result<Self, CellParserError<'de>>
    where
        As: CellDeserializeAsOwned<T> + ?Sized,
    {
        if n == 0 {
            return parser.parse_as::<_, As>().map(Self::Leaf);
        }
        let [lc, rc]: [Cell; 2] = parser.parse_as::<_, [Ref; 2]>()?;
        let [mut lp, mut rp] = [&lc, &rc].map(Cell::parser);
        let [l, r] = [
            Hashmap::parse_n_as::<As>(&mut lp, n - 1)?,
            Hashmap::parse_n_as::<As>(&mut rp, n - 1)?,
        ];

        for p in [lp, rp] {
            p.ensure_empty()?;
        }
        Ok(Self::Fork([l, r].map(Into::into)))
    }
}

/// ```tlb
/// hml_short$0 {m:#} {n:#} len:(Unary ~n) {n <= m} s:(n * Bit) = HmLabel ~n m;
/// hml_long$10 {m:#} n:(#<= m) s:(n * Bit) = HmLabel ~n m;
/// hml_same$11 {m:#} v:Bit n:(#<= m) = HmLabel ~n m;
/// ```
struct HmLabel {
    n: u32,
    s: BitVec<u8, Msb0>,
}

impl BitUnpackWithArgs for HmLabel {
    /// m
    type Args = u32;

    fn unpack_with<R>(mut reader: R, m: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(match reader.unpack()? {
            // hml_short$0
            false => {
                // len:(Unary ~n)
                let n: u32 = reader.unpack_as::<_, Unary>()?;
                // {n <= m}
                if n > m {
                    return Err(Error::custom("n > m"));
                }
                Self {
                    n,
                    // s:(n * Bit)
                    s: reader.read_bitvec(n as usize)?,
                }
            }
            true => match reader.unpack()? {
                // hml_long$10
                false => {
                    // n:(#<= m)
                    let n: u32 = reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?;
                    Self {
                        n,
                        // s:(n * Bit)
                        s: reader.read_bitvec(n as usize)?,
                    }
                }
                // hml_same$11
                true => {
                    // v:Bit
                    let s = reader.read_bitvec(1)?;
                    Self {
                        // n:(#<= m)
                        n: reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?,
                        s,
                    }
                }
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0};
    use tlb::{
        r#as::Data,
        ser::{r#as::CellSerializeWrapAsExt, CellSerializeExt},
    };

    use super::*;

    /// https://docs.ton.org/develop/data-formats/tl-b-types#hashmap-parsing-example
    #[test]
    fn parse() {
        let cell = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            (
                bits![u8, Msb0; 0,0].wrap_as::<Data>(),
                (
                    bits![u8, Msb0; 1,0,0,1,0,0,0].wrap_as::<Data>(),
                    bits![u8, Msb0; 1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,1,0,0,1]
                        .wrap_as::<Ref<Data>>(),
                    bits![u8, Msb0; 1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1]
                        .wrap_as::<Ref<Data>>(),
                )
                    .wrap_as::<Ref>(),
                bits![u8, Msb0; 1,0,1,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,1,0,0,1]
                    .wrap_as::<Ref<Data>>(),
            )
                .wrap_as::<Ref>(),
        )
            .to_cell()
            .unwrap();

        let hm: HashmapE<u16> = HashmapE::parse_n_as::<Data>(&mut cell.parser(), 8).unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(bits![u8, Msb0; 0,0,0,0,0,0,0,1]), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(bits![u8, Msb0; 0,0,0,1,0,0,0,1]), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(bits![u8, Msb0; 1,0,0,0,0,0,0,0]), Some(&777));
    }
}
