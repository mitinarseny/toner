use std::marker::PhantomData;

use tlb::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        de::{args::r#as::BitUnpackAsWithArgs, BitReader, BitReaderExt},
        r#as::{NBits, VarNBits},
        ser::{args::r#as::BitPackAsWithArgs, BitWriter, BitWriterExt},
    },
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        r#as::CellDeserializeAs,
        CellParser, CellParserError,
    },
    r#as::{NoArgs, ParseFully, Ref, Same},
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        r#as::CellSerializeAs,
        CellBuilder, CellBuilderError,
    },
    Error, ResultExt,
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
    #[inline]
    pub const fn new() -> Self {
        Self::Empty
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Root(root) => root.len(),
        }
    }

    #[inline]
    pub fn contains_key(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> bool {
        match self {
            Self::Empty => false,
            Self::Root(root) => root.contains_key(key),
        }
    }

    #[inline]
    pub fn get(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get(key),
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&mut T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get_mut(key),
        }
    }
}

impl<T, As> CellSerializeAsWithArgs<HashmapE<T>> for HashmapE<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    fn store_as_with(
        source: &HashmapE<T>,
        builder: &mut tlb::ser::CellBuilder,
        args: Self::Args,
    ) -> Result<(), tlb::ser::CellBuilderError> {
        match source {
            HashmapE::Empty => builder.pack(false)?,
            HashmapE::Root(root) => builder
                .pack(true)?
                .store_as_with::<_, Ref<&Hashmap<As>>>(root, args)?,
        };
        Ok(())
    }
}

impl<T> CellSerializeWithArgs for HashmapE<T>
where
    T: CellSerializeWithArgs,
{
    // (n, As::Args)
    type Args = (u32, T::Args);

    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, Same>(self, args)?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for HashmapE<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
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

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, Same>(args)
    }
}

pub struct HashmapEN<const N: u32, As: ?Sized = Same>(PhantomData<As>);

impl<const N: u32, T, As> CellSerializeAsWithArgs<HashmapE<T>> for HashmapEN<N, As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    fn store_as_with(
        source: &HashmapE<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, &HashmapE<As>>(source, (N, args))?;
        Ok(())
    }
}

impl<const N: u32, T, As> CellSerializeAs<HashmapE<T>> for HashmapEN<N, As>
where
    As: CellSerializeAs<T>,
{
    fn store_as(source: &HashmapE<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, &HashmapE<NoArgs<_, As>>>(source, (N, ()))?;
        Ok(())
    }
}

impl<'de, const N: u32, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for HashmapEN<N, As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
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
    #[inline]
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
    prefix: BitVec<u8, Msb0>,
    node: HashmapNode<T>,
}

impl<T> Hashmap<T> {
    #[inline]
    pub fn new(prefix: impl Into<BitVec<u8, Msb0>>, node: HashmapNode<T>) -> Self {
        Self {
            prefix: prefix.into(),
            node,
        }
    }

    #[inline]
    pub fn prefix(&self) -> &BitSlice<u8, Msb0> {
        &self.prefix
    }

    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.node.len()
    }

    #[inline]
    pub fn contains_key(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> bool {
        key.as_ref()
            .strip_prefix(&self.prefix)
            .map_or(false, |key| self.node.contains_key(key))
    }

    #[inline]
    pub fn get(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&T> {
        self.node.get(key.as_ref().strip_prefix(&self.prefix)?)
    }

    #[inline]
    pub fn get_mut(&mut self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&mut T> {
        self.node.get_mut(key.as_ref().strip_prefix(&self.prefix)?)
    }
}

impl<T, As> CellSerializeAsWithArgs<Hashmap<T>> for Hashmap<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    /// (n, As::Args)
    type Args = (u32, As::Args);

    fn store_as_with(
        source: &Hashmap<T>,
        builder: &mut CellBuilder,
        (n, args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            // label:(HmLabel ~l n)
            .pack_as_with::<_, &HmLabel>(source.prefix.as_bitslice(), n)
            .context("label")?
            // node:(HashmapNode m X)
            .store_as_with::<_, &HashmapNode<As>>(
                &source.node,
                // {n = (~m) + l}
                (n - source.prefix.len() as u32, args),
            )
            .context("node")?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Hashmap<T>> for Hashmap<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<Hashmap<T>, CellParserError<'de>> {
        // label:(HmLabel ~l n)
        let prefix: BitVec<u8, Msb0> = parser.unpack_as_with::<_, HmLabel>(n).context("label")?;
        // {n = (~m) + l}
        let m = n - prefix.len() as u32;
        Ok(Hashmap {
            prefix,
            // node:(HashmapNode m X)
            node: parser
                .parse_as_with::<_, HashmapNode<As>>((m, args))
                .context("node")?,
        })
    }
}

/// ```tlb
/// hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
/// hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
///            right:^(Hashmap n X) = HashmapNode (n + 1) X;
/// ```
#[derive(Debug, Clone)]
pub enum HashmapNode<T> {
    Leaf(T),
    /// [left, right]
    Fork([Box<Hashmap<T>>; 2]),
}

impl<T> HashmapNode<T> {
    #[inline]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        match self {
            Self::Leaf(_) => 1,
            Self::Fork([l, r]) => l.len() + r.len(),
        }
    }

    #[inline]
    pub fn contains_key(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> bool {
        let key = key.as_ref();
        match self {
            Self::Leaf(_) if key.is_empty() => true,
            Self::Fork([left, right]) => {
                let Some((is_right, key)) = key.split_first() else {
                    return false;
                };
                if *is_right { right } else { left }.contains_key(key)
            }
            _ => false,
        }
    }

    #[inline]
    pub fn get(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&T> {
        let key = key.as_ref();
        match self {
            Self::Leaf(v) if key.is_empty() => Some(v),
            Self::Fork([left, right]) => {
                let (is_right, key) = key.split_first()?;
                if *is_right { right } else { left }.get(key)
            }
            _ => None,
        }
    }

    #[inline]
    pub fn get_mut(&mut self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&mut T> {
        let key = key.as_ref();
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

impl<T, As> CellSerializeAsWithArgs<HashmapNode<T>> for HashmapNode<As>
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    fn store_as_with(
        source: &HashmapNode<T>,
        builder: &mut CellBuilder,
        (n, args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            HashmapNode::Leaf(leaf) => {
                if n != 0 {
                    return Err(CellBuilderError::custom(format!(
                        "key is too small, {n} more bits required"
                    )));
                }
                // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
                builder.store_as_with::<_, &As>(leaf, args)?
            }
            HashmapNode::Fork(fork) => {
                if n == 0 {
                    return Err(CellBuilderError::custom("key is too long"));
                }
                // hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
                // right:^(Hashmap n X) = HashmapNode (n + 1) X;
                builder.store_as_with::<_, &[Box<Ref<Hashmap<As>>>; 2]>(fork, (n - 1, args))?
            }
        };
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, HashmapNode<T>> for HashmapNode<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n + 1, T::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<HashmapNode<T>, CellParserError<'de>> {
        if n == 0 {
            // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
            return parser.parse_as_with::<_, As>(args).map(HashmapNode::Leaf);
        }

        Ok(HashmapNode::Fork(
            parser
                // hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
                // right:^(Hashmap n X) = HashmapNode (n + 1) X;
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
struct HmLabel;

impl BitPackAsWithArgs<BitSlice<u8, Msb0>> for HmLabel {
    /// m
    type Args = u32;

    fn pack_as_with<W>(
        source: &BitSlice<u8, Msb0>,
        mut writer: W,
        m: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let n = source.len() as u32;
        // {n <= m}
        if n < m {
            writer
                // hml_short$0
                .pack(false)?
                // len:(Unary ~n)
                .pack_as::<_, Unary>(source.len())?
                // s:(n * Bit)
                .pack(source)?;
            return Ok(());
        }

        let n_bits = m.ilog2() + 1;
        let v = if source.all() {
            true
        } else if source.not_any() {
            false
        } else {
            writer
                // hml_long$10
                .pack_as::<_, NBits<2>>(0b10)?
                // n:(#<= m)
                .pack_as_with::<_, VarNBits>(n, n_bits)?
                // s:(n * Bit)
                .pack(source)?;
            return Ok(());
        };
        writer
            // hml_same$11
            .pack_as::<_, NBits<2>>(0b11)?
            // v:Bit
            .pack(v)?
            // n:(#<= m)
            .pack_as_with::<_, VarNBits>(n, n_bits)?;
        Ok(())
    }
}

impl BitUnpackAsWithArgs<BitVec<u8, Msb0>> for HmLabel {
    /// m
    type Args = u32;

    fn unpack_as_with<R>(mut reader: R, m: Self::Args) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader,
    {
        match reader.unpack()? {
            // hml_short$0
            false => {
                // len:(Unary ~n)
                let n: u32 = reader.unpack_as::<_, Unary>()?;
                // {n <= m}
                if n > m {
                    return Err(Error::custom("n > m"));
                }
                // s:(n * Bit)
                reader.unpack_with(n as usize)
            }
            true => match reader.unpack()? {
                // hml_long$10
                false => {
                    // n:(#<= m)
                    let n: u32 = reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?;
                    // s:(n * Bit)
                    reader.unpack_with(n as usize)
                }
                // hml_same$11
                true => {
                    // v:Bit
                    let v: bool = reader.unpack()?;
                    // n:(#<= m)
                    let n: u32 = reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?;
                    Ok(BitVec::repeat(v, n as usize))
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use tlb::{
        bits::bitvec::{bits, order::Msb0, view::AsBits},
        r#as::Data,
        ser::{r#as::CellSerializeWrapAsExt, CellSerializeExt},
        Cell,
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
                    // original example uses 0b1001000 due to hml_long$10,
                    // but hml_short$0 is more efficient here
                    bits![u8, Msb0; 0,1,1,0,0,0].wrap_as::<Data>(),
                    bits![u8, Msb0; 1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,1,0,0,1]
                        .wrap_as::<Ref<Data>>(),
                    bits![u8, Msb0; 1,0,1,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,1,1,0,1,1,1,1]
                        .wrap_as::<Ref<Data>>(),
                )
                    .wrap_as::<Ref>(),
                // original example uses 0b1011100000000000001100001001
                // due to hml_long$10, but hml_same$11 is more efficient
                bits![u8, Msb0; 1,1,0,1,1,1,0,0,0,0,0,0,1,1,0,0,0,0,1,0,0,1].wrap_as::<Ref<Data>>(),
            )
                .wrap_as::<Ref>(),
        )
            .to_cell()
            .unwrap();

        let hm: HashmapE<u16> = cell.parse_fully_as::<_, HashmapEN<8, Data>>().unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(1u8.to_be_bytes().as_bits()), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(17u8.to_be_bytes().as_bits()), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(128u8.to_be_bytes().as_bits()), Some(&777));

        let mut builder = Cell::builder();
        builder.store_as::<_, HashmapEN<8, Data>>(hm).unwrap();
        let got = builder.into_cell();
        assert_eq!(got, cell);
    }
}
