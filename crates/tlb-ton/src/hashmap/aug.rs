use std::iter::once;

use impl_tools::autoimpl;
use tlb::{
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        de::BitReaderExt,
        ser::BitWriterExt,
    },
    de::{
        args::{r#as::CellDeserializeAsWithArgs, CellDeserializeWithArgs},
        CellParser, CellParserError,
    },
    r#as::{ParseFully, Ref, Same},
    ser::{
        args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
        CellBuilder, CellBuilderError,
    },
    Error, ResultExt,
};

use super::hm_label::HmLabel;

/// [`HashmapAugE n X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmapauge).  
/// When `E = ()` it is equivalent to [`HashmapE n X`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmap)
/// ```tlb
/// ahme_empty$0 {n:#} {X:Type} {Y:Type} extra:Y = HashmapAugE n X Y;      
/// ahme_root$1 {n:#} {X:Type} {Y:Type} root:^(HashmapAug n X Y)
/// extra:Y = HashmapAugE n X Y;
/// ```
#[derive(Debug, Clone)]
#[autoimpl(Deref using self.m)]
#[autoimpl(DerefMut using self.m)]
#[autoimpl(Default where E: Default)]
pub struct HashmapAugE<T, E = ()> {
    pub m: HashmapE<T, E>,
    pub extra: E,
}

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<HashmapAugE<T, E>> for HashmapAugE<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    /// (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn store_as_with(
        source: &HashmapAugE<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_as_with::<_, &HashmapE<AsT, AsE>>(&source.m, (n, node_args, extra_args.clone()))?
            // extra:Y
            .store_as_with::<_, &AsE>(&source.extra, extra_args)
            .context("extra")?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, HashmapAugE<T, E>>
    for HashmapAugE<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    /// (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<HashmapAugE<T, E>, CellParserError<'de>> {
        Ok(HashmapAugE {
            m: parser.parse_as_with::<_, HashmapE<AsT, AsE>>((n, node_args, extra_args.clone()))?,
            // extra:Y
            extra: parser
                .parse_as_with::<_, AsE>(extra_args)
                .context("extra")?,
        })
    }
}

/// [`HashmapE n X`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmap).  
/// Type parameter `E` is optional and stands for `extra`, so it can be reused
/// for [`HashmapAugE n X E`](HashmapAugE)
/// ```tlb
/// hme_empty$0 {n:#} {X:Type} = HashmapE n X;
/// hme_root$1 {n:#} {X:Type} root:^(Hashmap n X) = HashmapE n X;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashmapE<T, E = ()> {
    Empty,
    Root(Hashmap<T, E>),
}

impl<T, E> Default for HashmapE<T, E> {
    #[inline]
    fn default() -> Self {
        Self::Empty
    }
}

impl<T, E> HashmapE<T, E> {
    /// Create empty hashmap
    #[inline]
    pub const fn new() -> Self {
        Self::Empty
    }

    /// Return whether this hashmap is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Return number of leaf nodes in this hashmap
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Root(root) => root.len(),
        }
    }

    /// Returns whether this hashmap contains given key
    #[inline]
    pub fn contains_key(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> bool {
        match self {
            Self::Empty => false,
            Self::Root(root) => root.contains_key(key),
        }
    }

    /// Returns reference to leaf value associated with given key
    #[inline]
    pub fn get(&self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get(key),
        }
    }

    /// Returns mutable reference to leaf value associated with given key
    #[inline]
    pub fn get_mut(&mut self, key: impl AsRef<BitSlice<u8, Msb0>>) -> Option<&mut T> {
        match self {
            Self::Empty => None,
            Self::Root(root) => root.get_mut(key),
        }
    }
}

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<HashmapE<T, E>> for HashmapE<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    // (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn store_as_with(
        source: &HashmapE<T, E>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            HashmapE::Empty => builder
                // hme_empty$0
                .pack(false)?,
            HashmapE::Root(root) => builder
                // hme_root$1
                .pack(true)?
                // root:^(Hashmap n X)
                .store_as_with::<_, Ref<&Hashmap<AsT, AsE>>>(root, args)?,
        };
        Ok(())
    }
}

impl<T, E> CellSerializeWithArgs for HashmapE<T, E>
where
    T: CellSerializeWithArgs,
    T::Args: Clone,
    E: CellSerializeWithArgs,
    E::Args: Clone,
{
    // (n, T::Args, E::Args)
    type Args = (u32, T::Args, E::Args);

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, Same>(self, args)?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, HashmapE<T, E>> for HashmapE<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    // (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<HashmapE<T, E>, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            // hme_empty$0
            false => HashmapE::Empty,
            // hme_root$1
            true => parser
                // root:^(Hashmap n X)
                .parse_as_with::<_, Ref<ParseFully<Hashmap<AsT, AsE>>>>((n, node_args, extra_args))
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

impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, C> for HashmapE<As>
where
    C: IntoIterator<Item = (Key, T)> + Extend<(Key, T)> + Default, // IntoIterator used as type constraint for T
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    // (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args): Self::Args,
    ) -> Result<C, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            // hme_empty$0
            false => C::default(),
            // hme_root$1
            true => parser
                // root:^(Hashmap n X)
                .parse_as_with::<_, Ref<ParseFully<Hashmap<As, ()>>>>((n, node_args))?,
        })
    }
}

/// [`Hashmap n X`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmap)  
/// Type parameter `E` is optional and stands for `extra`, so it can be reused
/// for [`HashmapAug n X E`](HashmapAugE)
/// ```tlb
/// hm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
/// {n = (~m) + l} node:(HashmapNode m X) = Hashmap n X;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hashmap<T, E = ()> {
    pub(super) prefix: BitVec<u8, Msb0>,
    pub(super) node: HashmapAugNode<T, E>,
}

impl<T, E> Hashmap<T, E> {
    #[inline]
    pub fn new(prefix: impl Into<BitVec<u8, Msb0>>, node: HashmapAugNode<T, E>) -> Self {
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
            .is_some_and(|key| self.node.contains_key(key))
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

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<Hashmap<T, E>> for Hashmap<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    /// (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn store_as_with(
        source: &Hashmap<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            // label:(HmLabel ~l n)
            .pack_as_with::<_, &HmLabel>(source.prefix.as_bitslice(), n)
            .context("label")?
            // node:(HashmapNode m X)
            .store_as_with::<_, &HashmapAugNode<AsT, AsE>>(
                &source.node,
                (
                    // {n = (~m) + l}
                    n - source.prefix.len() as u32,
                    node_args,
                    extra_args,
                ),
            )
            .context("node")?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, Hashmap<T, E>> for Hashmap<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    /// (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<Hashmap<T, E>, CellParserError<'de>> {
        // label:(HmLabel ~l n)
        let prefix: BitVec<u8, Msb0> = parser.unpack_as_with::<_, HmLabel>(n).context("label")?;
        // {n = (~m) + l}
        let m = n - prefix.len() as u32;
        Ok(Hashmap {
            prefix,
            // node:(HashmapNode m X)
            node: parser
                .parse_as_with::<_, HashmapAugNode<AsT, AsE>>((m, node_args, extra_args))
                .context("node")?,
        })
    }
}

pub type Key = BitVec<u8, Msb0>;
impl<'de, T, As, C> CellDeserializeAsWithArgs<'de, C> for Hashmap<As, ()>
where
    C: IntoIterator<Item = (Key, T)> + Extend<(Key, T)> + Default, // IntoIterator used as type constraint for T
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    /// (n, As::Args)
    type Args = (u32, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, args): Self::Args,
    ) -> Result<C, CellParserError<'de>> {
        let mut output = C::default();
        let mut stack: Vec<(u32, Key, CellParser<'de>)> = Vec::new();

        #[inline]
        fn parse<'de, T, As, C>(
            parser: &mut CellParser<'de>,
            stack: &mut Vec<(u32, Key, CellParser<'de>)>,
            output: &mut C,
            n: u32,
            mut prefix: Key,
            args: As::Args,
        ) -> Result<(), CellParserError<'de>>
        where
            C: Extend<(Key, T)>,
            As: CellDeserializeAsWithArgs<'de, T>,
        {
            // label:(HmLabel ~l n)
            let next_prefix: BitVec<u8, Msb0> =
                parser.unpack_as_with::<_, HmLabel>(n).context("label")?;
            // {n = (~m) + l}
            let m = n - next_prefix.len() as u32;

            prefix.extend_from_bitslice(&next_prefix);

            match m {
                // bt_leaf$0
                0 => output.extend(once((prefix, parser.parse_as_with::<_, As>(args)?))),
                // bt_fork$1
                1.. => stack.extend(
                    parser
                        .parse_as::<_, [Ref; 2]>()?
                        .into_iter()
                        .enumerate()
                        // HashmapNode (n + 1)
                        .map(|(next_prefix, parser)| {
                            let mut prefix = prefix.clone();
                            prefix.push(next_prefix != 0);

                            (m - 1, prefix, parser)
                        })
                        // inverse ordering
                        .rev(),
                ),
            }
            Ok(())
        }

        parse::<_, As, C>(
            parser,
            &mut stack,
            &mut output,
            n,
            Key::default(),
            args.clone(),
        )?;

        while let Some((n, prefix, mut parser)) = stack.pop() {
            parse::<_, As, C>(
                &mut parser,
                &mut stack,
                &mut output,
                n,
                prefix,
                args.clone(),
            )?;
        }

        Ok(output)
    }
}

/// [`HashmapNode n X`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmap)  
/// Type parameter `E` is optional and stands for `extra`, so it can be reused
/// for [`HashmapAugNode n X E`](HashmapAugNode)
/// ```tlb
/// hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
/// hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
///            right:^(Hashmap n X) = HashmapNode (n + 1) X;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashmapNode<T, E = ()> {
    Leaf(T),
    /// [left, right]
    Fork([Box<Hashmap<T, E>>; 2]),
}

impl<T, E> HashmapNode<T, E> {
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

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<HashmapNode<T, E>> for HashmapNode<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    // (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn store_as_with(
        source: &HashmapNode<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            HashmapNode::Leaf(value) => {
                if n != 0 {
                    return Err(CellBuilderError::custom(format!(
                        "key is too small, {n} more bits required"
                    )));
                }
                // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
                builder.store_as_with::<_, &AsT>(value, node_args)?
            }
            HashmapNode::Fork(fork) => {
                if n == 0 {
                    return Err(CellBuilderError::custom("key is too long"));
                }
                // hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
                // right:^(Hashmap n X) = HashmapNode (n + 1) X;
                builder.store_as_with::<_, &[Box<Ref<Hashmap<AsT, AsE>>>; 2]>(
                    fork,
                    (n - 1, node_args, extra_args),
                )?
            }
        };
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, HashmapNode<T, E>>
    for HashmapNode<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    /// (n + 1, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<HashmapNode<T, E>, CellParserError<'de>> {
        if n == 0 {
            // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
            return parser
                .parse_as_with::<_, AsT>(node_args)
                .map(HashmapNode::Leaf);
        }

        Ok(HashmapNode::Fork(
            parser
                // left:^(Hashmap n X) right:^(Hashmap n X)
                .parse_as_with::<_, [Box<Ref<ParseFully<Hashmap<AsT, AsE>>>>; 2]>((
                    n - 1,
                    node_args,
                    extra_args,
                ))?,
        ))
    }
}

/// [`HashmapAugNode n X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmapauge)  
/// When `E = ()` it is equivalent to [`HashmapNode n X`](https://docs.ton.org/develop/data-formats/tl-b-types#hashmap)
/// ```tlb
/// ahmn_leaf#_ {X:Type} {Y:Type} extra:Y value:X = HashmapAugNode 0 X Y;
/// ahmn_fork#_ {n:#} {X:Type} {Y:Type} left:^(HashmapAug n X Y)
/// right:^(HashmapAug n X Y) extra:Y = HashmapAugNode (n + 1) X Y;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Deref using self.node)]
#[autoimpl(DerefMut using self.node)]
pub struct HashmapAugNode<T, E = ()> {
    pub node: HashmapNode<T, E>,
    pub extra: E,
}

impl<T, E> HashmapAugNode<T, E> {
    #[inline]
    pub fn new(node: HashmapNode<T, E>, extra: E) -> Self {
        Self { node, extra }
    }
}

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<HashmapAugNode<T, E>> for HashmapAugNode<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    /// (n + 1, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn store_as_with(
        source: &HashmapAugNode<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            // extra:Y
            .store_as_with::<_, &AsE>(&source.extra, extra_args.clone())?
            .store_as_with::<_, &HashmapNode<AsT, AsE>>(&source.node, (n, node_args, extra_args))?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, HashmapAugNode<T, E>>
    for HashmapAugNode<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    /// (n + 1, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<HashmapAugNode<T, E>, CellParserError<'de>> {
        Ok(HashmapAugNode {
            // extra:Y
            extra: parser.parse_as_with::<_, AsE>(extra_args.clone())?,
            node: parser.parse_as_with::<_, HashmapNode<AsT, AsE>>((n, node_args, extra_args))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashMap};
    use tlb::{
        bits::bitvec::{bits, order::Msb0, view::AsBits},
        r#as::{Data, NoArgs},
        ser::{r#as::CellSerializeWrapAsExt, CellSerializeExt},
        Cell,
    };

    use super::*;

    #[test]
    fn parse() {
        let cell = given_cell_from_example();

        let hm: HashmapE<u16> = cell
            .parse_fully_as_with::<_, HashmapE<Data<NoArgs<_>>, NoArgs<_>>>((8, (), ()))
            .unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(1u8.to_be_bytes().as_bits()), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(17u8.to_be_bytes().as_bits()), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(128u8.to_be_bytes().as_bits()), Some(&777));

        let mut builder = Cell::builder();
        builder
            .store_as_with::<_, HashmapE<Data<NoArgs<_>>, NoArgs<_>>>(hm, (8, (), ()))
            .unwrap();
        let got = builder.into_cell();
        assert_eq!(got, cell);
    }

    #[test]
    fn hashmape_parse_as_std_hashmap() {
        let cell = given_cell_from_example();

        let hm: HashMap<Key, u16> = cell
            .parse_fully_as_with::<_, HashmapE<Data<NoArgs<_>>>>((8, ()))
            .unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(1u8.to_be_bytes().as_bits()), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(17u8.to_be_bytes().as_bits()), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(128u8.to_be_bytes().as_bits()), Some(&777));
    }

    #[test]
    fn hashmape_parse_as_std_btreemap() {
        let cell = given_cell_from_example();

        let hm: BTreeMap<Key, u16> = cell
            .parse_fully_as_with::<_, HashmapE<Data<NoArgs<_>>>>((8, ()))
            .unwrap();

        assert_eq!(hm.len(), 3);
        // 1 -> 777
        assert_eq!(hm.get(1u8.to_be_bytes().as_bits()), Some(&777));
        // 17 -> 111
        assert_eq!(hm.get(17u8.to_be_bytes().as_bits()), Some(&111));
        // 128 -> 777
        assert_eq!(hm.get(128u8.to_be_bytes().as_bits()), Some(&777));
    }

    /// See <https://docs.ton.org/develop/data-formats/tl-b-types#hashmap-parsing-example>
    fn given_cell_from_example() -> Cell {
        (
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
            .unwrap()
    }
}
