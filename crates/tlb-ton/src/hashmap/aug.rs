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

// pub struct HashmapEN<const N: u32, AsT: ?Sized = Same, AsE: ?Sized = Same>(PhantomData<(AsT, AsE)>);

// impl<const N: u32, T, AsT, E, AsE> CellSerializeAsWithArgs<HashmapE<T, E>>
//     for HashmapEN<N, AsT, AsE>
// where
//     AsT: CellSerializeAsWithArgs<T>,
//     AsT::Args: Clone,
//     AsE: CellSerializeAsWithArgs<E>,
//     AsE::Args: Clone,
// {
//     type Args = (AsT::Args, AsE::Args);

//     fn store_as_with(
//         source: &HashmapE<T, E>,
//         builder: &mut CellBuilder,
//         (node_args, extra_args): Self::Args,
//     ) -> Result<(), CellBuilderError> {
//         builder.store_as_with::<_, &HashmapE<AsT, AsE>>(source, (N, node_args, extra_args))?;
//         Ok(())
//     }
// }

// impl<const N: u32, T, AsT, E, AsE> CellSerializeAs<HashmapE<T, E>> for HashmapEN<N, AsT, AsE>
// where
//     AsT: CellSerializeAs<T>,
//     AsE: CellSerializeAs<E>,
// {
//     fn store_as(
//         source: &HashmapE<T, E>,
//         builder: &mut CellBuilder,
//     ) -> Result<(), CellBuilderError> {
//         builder
//             .store_as_with::<_, &HashmapE<NoArgs<_, AsT>, NoArgs<_, AsE>>>(source, (N, (), ()))?;
//         Ok(())
//     }
// }
//
// impl<'de, const N: u32, T, As> CellDeserializeAsWithArgs<'de, HashmapE<T>> for HashmapEN<N, As>
// where
//     As: CellDeserializeAsWithArgs<'de, T>,
//     As::Args: Clone,
// {
//     type Args = As::Args;

//     #[inline]
//     fn parse_as_with(
//         parser: &mut CellParser<'de>,
//         args: Self::Args,
//     ) -> Result<HashmapE<T>, CellParserError<'de>> {
//         parser.parse_as_with::<_, HashmapE<As>>((N, args))
//     }
// }

// impl<'de, const N: u32, T, As> CellDeserializeAs<'de, HashmapE<T>> for HashmapEN<N, As>
// where
//     As: CellDeserializeAs<'de, T>,
// {
//     #[inline]
//     fn parse_as(parser: &mut CellParser<'de>) -> Result<HashmapE<T>, CellParserError<'de>> {
//         parser.parse_as_with::<_, HashmapE<NoArgs<_, As>>>((N, ()))
//     }
// }

/// ```tlb
/// ahm_edge#_ {n:#} {X:Type} {Y:Type} {l:#} {m:#}
/// label:(HmLabel ~l n) {n = (~m) + l}
/// node:(HashmapAugNode m X Y) = HashmapAug n X Y;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hashmap<T, E = ()> {
    pub(super) prefix: BitVec<u8, Msb0>,
    pub(super) node: AugNode<T, E>,
}

impl<T, E> Hashmap<T, E> {
    #[inline]
    pub fn new(prefix: impl Into<BitVec<u8, Msb0>>, node: AugNode<T, E>) -> Self {
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
            .store_as_with::<_, &AugNode<AsT, AsE>>(
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
                .parse_as_with::<_, AugNode<AsT, AsE>>((m, node_args, extra_args))
                .context("node")?,
        })
    }
}

/// ```tlb
/// hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
/// hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
///            right:^(Hashmap n X) = HashmapNode (n + 1) X;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node<T, E = ()> {
    Leaf(T),
    /// [left, right]
    Fork([Box<Hashmap<T, E>>; 2]),
}

impl<T, E> Node<T, E> {
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

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<Node<T, E>> for Node<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    // (n, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn store_as_with(
        source: &Node<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            Node::Leaf(value) => {
                if n != 0 {
                    return Err(CellBuilderError::custom(format!(
                        "key is too small, {n} more bits required"
                    )));
                }
                // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
                builder.store_as_with::<_, &AsT>(value, node_args)?
            }
            Node::Fork(fork) => {
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

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, Node<T, E>> for Node<AsT, AsE>
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
    ) -> Result<Node<T, E>, CellParserError<'de>> {
        if n == 0 {
            // hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
            return parser.parse_as_with::<_, AsT>(node_args).map(Node::Leaf);
        }

        Ok(Node::Fork(
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

/// ```tlb
/// ahmn_leaf#_ {X:Type} {Y:Type} extra:Y value:X = HashmapAugNode 0 X Y;
/// ahmn_fork#_ {n:#} {X:Type} {Y:Type} left:^(HashmapAug n X Y)
/// right:^(HashmapAug n X Y) extra:Y = HashmapAugNode (n + 1) X Y;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[autoimpl(Deref using self.node)]
#[autoimpl(DerefMut using self.node)]
pub struct AugNode<T, E = ()> {
    pub node: Node<T, E>,
    pub extra: E,
}

impl<T, E> AugNode<T, E> {
    #[inline]
    pub fn new(node: Node<T, E>, extra: E) -> Self {
        Self { node, extra }
    }
}

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<AugNode<T, E>> for AugNode<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    /// (n + 1, AsT::Args, AsE::Args)
    type Args = (u32, AsT::Args, AsE::Args);

    fn store_as_with(
        source: &AugNode<T, E>,
        builder: &mut CellBuilder,
        (n, node_args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            // extra:Y
            .store_as_with::<_, &AsE>(&source.extra, extra_args.clone())?
            .store_as_with::<_, &Node<AsT, AsE>>(&source.node, (n, node_args, extra_args))?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, AugNode<T, E>> for AugNode<AsT, AsE>
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
    ) -> Result<AugNode<T, E>, CellParserError<'de>> {
        Ok(AugNode {
            // extra:Y
            extra: parser.parse_as_with::<_, AsE>(extra_args.clone())?,
            node: parser.parse_as_with::<_, Node<AsT, AsE>>((n, node_args, extra_args))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use tlb::{
        bits::bitvec::{bits, order::Msb0, view::AsBits},
        r#as::{Data, NoArgs},
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
}
