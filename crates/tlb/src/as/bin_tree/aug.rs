use crate::{
    r#as::{ParseFully, Ref},
    bits::{de::BitReaderExt, ser::BitWriterExt},
    de::{CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerializeAs},
};

/// [`BinTreeAug X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#bintree)  
/// ```tlb
/// bta_leaf$0 {X:Type} {Y:Type} extra:Y leaf:X = BinTreeAug X Y;
/// bta_fork$1 {X:Type} {Y:Type} left:^(BinTreeAug X Y)
/// right:^(BinTreeAug X Y) extra:Y = BinTreeAug X Y;
/// ```
pub struct BinTreeAug<T, E = ()> {
    pub node: BinTreeNode<T, E>,
    pub extra: E,
}

impl<T, AsT, E, AsE> CellSerializeAs<BinTreeAug<T, E>> for BinTreeAug<AsT, AsE>
where
    AsT: CellSerializeAs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAs<E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn store_as(
        source: &BinTreeAug<T, E>,
        builder: &mut CellBuilder,
        (args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_as::<_, &AsE>(&source.extra, extra_args.clone())?
            .store_as::<_, &BinTreeNode<AsT, AsE>>(&source.node, (args, extra_args))?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAs<'de, BinTreeAug<T, E>> for BinTreeAug<AsT, AsE>
where
    AsT: CellDeserializeAs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAs<'de, E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        (args, extra_args): Self::Args,
    ) -> Result<BinTreeAug<T, E>, CellParserError<'de>> {
        Ok(BinTreeAug {
            extra: parser.parse_as::<_, AsE>(extra_args.clone())?,
            node: parser.parse_as::<_, ParseFully<BinTreeNode<AsT, AsE>>>((args, extra_args))?,
        })
    }
}

/// [`BinTreeAugNode X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#bintree)
/// Type parameter `E` is optional and stands for `extra`, so it can be reused
/// for [`BinTree X`](super::BinTree)
/// ```tlb
/// bta_leaf$0 {X:Type} {Y:Type} extra:Y leaf:X = BinTreeAug X Y;
/// bta_fork$1 {X:Type} {Y:Type} left:^(BinTreeAug X Y)
/// right:^(BinTreeAug X Y) extra:Y = BinTreeAug X Y;
/// ```
pub enum BinTreeNode<T, E = ()> {
    Leaf(T),
    Fork([Box<BinTreeAug<T, E>>; 2]),
}

impl<T, AsT, E, AsE> CellSerializeAs<BinTreeNode<T, E>> for BinTreeNode<AsT, AsE>
where
    AsT: CellSerializeAs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAs<E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn store_as(
        source: &BinTreeNode<T, E>,
        builder: &mut CellBuilder,
        (args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            BinTreeNode::Leaf(leaf) => builder.pack(false, ())?.store_as::<_, &AsT>(leaf, args)?,
            BinTreeNode::Fork(fork) => builder
                .pack(true, ())?
                .store_as::<_, &[Box<Ref<BinTreeAug<AsT, AsE>>>; 2]>(fork, (args, extra_args))?,
        };
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAs<'de, BinTreeNode<T, E>> for BinTreeNode<AsT, AsE>
where
    AsT: CellDeserializeAs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAs<'de, E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        (args, extra_args): Self::Args,
    ) -> Result<BinTreeNode<T, E>, CellParserError<'de>> {
        Ok(match parser.unpack(())? {
            false => BinTreeNode::Leaf(parser.parse_as::<_, AsT>(args)?),
            true => BinTreeNode::Fork(
                parser.parse_as::<_, [Box<Ref<ParseFully<BinTreeAug<AsT, AsE>>>>; 2]>((
                    args, extra_args,
                ))?,
            ),
        })
    }
}
