use tlb::{
    bits::{de::BitReaderExt, ser::BitWriterExt},
    de::{args::r#as::CellDeserializeAsWithArgs, OrdinaryCellParser, OrdinaryCellParserError},
    r#as::{ParseFully, Ref},
    ser::{args::r#as::CellSerializeAsWithArgs, CellBuilder, CellBuilderError},
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

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<BinTreeAug<T, E>> for BinTreeAug<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn store_as_with(
        source: &BinTreeAug<T, E>,
        builder: &mut CellBuilder,
        (args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_as_with::<_, &AsE>(&source.extra, extra_args.clone())?
            .store_as_with::<_, &BinTreeNode<AsT, AsE>>(&source.node, (args, extra_args))?;
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, BinTreeAug<T, E>> for BinTreeAug<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut OrdinaryCellParser<'de>,
        (args, extra_args): Self::Args,
    ) -> Result<BinTreeAug<T, E>, OrdinaryCellParserError<'de>> {
        Ok(BinTreeAug {
            extra: parser.parse_as_with::<_, AsE>(extra_args.clone())?,
            node: parser
                .parse_as_with::<_, ParseFully<BinTreeNode<AsT, AsE>>>((args, extra_args))?,
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

impl<T, AsT, E, AsE> CellSerializeAsWithArgs<BinTreeNode<T, E>> for BinTreeNode<AsT, AsE>
where
    AsT: CellSerializeAsWithArgs<T>,
    AsT::Args: Clone,
    AsE: CellSerializeAsWithArgs<E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn store_as_with(
        source: &BinTreeNode<T, E>,
        builder: &mut CellBuilder,
        (args, extra_args): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source {
            BinTreeNode::Leaf(leaf) => builder.pack(false)?.store_as_with::<_, &AsT>(leaf, args)?,
            BinTreeNode::Fork(fork) => builder
                .pack(true)?
                .store_as_with::<_, &[Box<Ref<BinTreeAug<AsT, AsE>>>; 2]>(
                    fork,
                    (args, extra_args),
                )?,
        };
        Ok(())
    }
}

impl<'de, T, AsT, E, AsE> CellDeserializeAsWithArgs<'de, BinTreeNode<T, E>>
    for BinTreeNode<AsT, AsE>
where
    AsT: CellDeserializeAsWithArgs<'de, T>,
    AsT::Args: Clone,
    AsE: CellDeserializeAsWithArgs<'de, E>,
    AsE::Args: Clone,
{
    type Args = (AsT::Args, AsE::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut OrdinaryCellParser<'de>,
        (args, extra_args): Self::Args,
    ) -> Result<BinTreeNode<T, E>, OrdinaryCellParserError<'de>> {
        Ok(match parser.unpack()? {
            false => BinTreeNode::Leaf(parser.parse_as_with::<_, AsT>(args)?),
            true => BinTreeNode::Fork(
                parser.parse_as_with::<_, [Box<Ref<ParseFully<BinTreeAug<AsT, AsE>>>>; 2]>((
                    args, extra_args,
                ))?,
            ),
        })
    }
}
