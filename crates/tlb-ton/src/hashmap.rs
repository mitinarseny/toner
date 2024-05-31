use std::marker::PhantomData;

use tlb::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        de::{args::BitUnpackWithArgs, BitReader, BitReaderExt},
        r#as::VarNBits,
    },
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellParser, CellParserError,
    },
    r#as::{NoArgs, ParseFully, Ref, Same},
    Error,
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
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for HashmapE<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<HashmapE<T>, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            // hme_empty$0
            false => HashmapE::Empty,
            // hme_root$1
            true => parser
                // root:^(Hashmap n X)
                .parse_as_with::<_, Ref<ParseFully<Hashmap<As>>>>((n, args))
                .map(HashmapE::Root)?,
        })
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for HashmapE<T>
where
    T: CellDeserializeWithArgs<'de>,
    T::Args: Clone,
{
    /// (n, T::Args)
    type Args = (u32, T::Args);

    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, Same>(args)
    }
}

pub struct HashmapEN<const N: u32, As: ?Sized = Same>(PhantomData<As>);

impl<'de, const N: u32, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for HashmapEN<N, As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<HashmapE<T>, CellParserError<'de>> {
        parser.parse_as_with::<_, HashmapE<As>>((N, args))
    }
}

impl<'de, const N: u32, T, As> CellDeserializeAs<'de, HashmapE<T>> for HashmapEN<N, As>
where
    As: CellDeserializeAs<'de, T>,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<HashmapE<T>, CellParserError<'de>> {
        parser.parse_as_with::<_, HashmapE<NoArgs<_, As>>>((N, ()))
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
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Hashmap<T>> for Hashmap<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n, As::Args)
    type Args = (u32, As::Args);

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<Hashmap<T>, CellParserError<'de>> {
        // label:(HmLabel ~l n)
        let label: HmLabel = parser.unpack_with(n)?;
        // {n = (~m) + l}
        let m = n - label.n;
        Ok(Hashmap {
            label: label.s,
            // node:(HashmapNode m X)
            node: parser.parse_as_with::<_, HashmapNode<As>>((m, args))?,
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
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, HashmapNode<T>> for HashmapNode<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n, T::Args)
    type Args = (u32, As::Args);

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<HashmapNode<T>, CellParserError<'de>> {
        if n == 0 {
            return parser.parse_as_with::<_, As>(args).map(HashmapNode::Leaf);
        }

        Ok(HashmapNode::Fork(
            parser
                .parse_as_with::<_, [Ref<ParseFully<Hashmap<As>>>; 2]>((n - 1, args))?
                .map(Into::into),
        ))
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
    use tlb::{
        bits::bitvec::{bits, order::Msb0},
        r#as::Data,
        ser::{r#as::CellSerializeWrapAsExt, CellSerializeExt},
    };

    use super::*;

    /// See <https://docs.ton.org/develop/data-formats/tl-b-types#hashmap-parsing-example>
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

        let hm: HashmapE<u16> = cell
            .parse_fully_as::<_, HashmapEN<8, Data>>()
            .unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(bits![u8, Msb0; 0,0,0,0,0,0,0,1]), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(bits![u8, Msb0; 0,0,0,1,0,0,0,1]), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(bits![u8, Msb0; 1,0,0,0,0,0,0,0]), Some(&777));
    }
}
