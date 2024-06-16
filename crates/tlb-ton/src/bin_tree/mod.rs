//! Collection of bintree-like **de**/**ser**ializable data structures
pub mod aug;

use std::ops::Deref;

use tlb::bits::de::BitReaderExt;
use tlb::de::args::r#as::CellDeserializeAsWithArgs;
use tlb::de::{CellParser, CellParserError};
use tlb::r#as::Ref;

/// [`BinTree X`](https://docs.ton.org/develop/data-formats/tl-b-types#bintree)
/// ```tlb
/// bt_leaf$0 {X:Type} leaf:X = BinTree X;
/// bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X) = BinTree X;
/// ```
#[derive(Debug, Clone)]
pub enum BinTree<X> {
    Leaf(X),
    Fork([Box<BinTree<X>>; 2]),
}

impl<X> BinTree<X> {
    #[inline]
    pub fn as_leaf(&self) -> Option<&X> {
        match self {
            Self::Leaf(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn as_fork(&self) -> Option<[&BinTree<X>; 2]> {
        match self {
            Self::Fork(v) => Some(v.each_ref().map(Deref::deref)),
            _ => None,
        }
    }

    #[inline]
    pub fn into_leaf(self) -> Option<X> {
        match self {
            Self::Leaf(v) => Some(v),
            _ => None,
        }
    }

    #[inline]
    pub fn into_fork(self) -> Option<[BinTree<X>; 2]> {
        match self {
            Self::Fork(v) => Some(v.map(|b| *b)),
            _ => None,
        }
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, BinTree<T>> for BinTree<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<BinTree<T>, CellParserError<'de>> {
        Ok(match parser.unpack()? {
            // bt_leaf$0
            false => BinTree::Leaf(parser.parse_as_with::<T, As>(args)?),
            // bt_fork$1
            true => BinTree::Fork(parser.parse_as_with::<_, [Box<Ref<BinTree<As>>>; 2]>(args)?),
        })
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Vec<T>> for BinTree<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Vec<T>, CellParserError<'de>> {
        let mut output = Vec::new();
        let mut stack: Vec<CellParser<'de>> = Vec::new();

        #[inline]
        fn parse<'de, T, As>(
            parser: &mut CellParser<'de>,
            stack: &mut Vec<CellParser<'de>>,
            output: &mut Vec<T>,
            args: As::Args,
        ) -> Result<(), CellParserError<'de>>
        where
            As: CellDeserializeAsWithArgs<'de, T>,
        {
            match parser.unpack()? {
                // bt_leaf$0
                false => output.push(parser.parse_as_with::<_, As>(args)?),
                // bt_fork$1
                true => stack.extend(
                    parser
                        .parse_as::<_, [Ref; 2]>()?
                        .into_iter()
                        // inverse ordering
                        .rev(),
                ),
            }
            Ok(())
        }

        parse::<_, As>(parser, &mut stack, &mut output, args.clone())?;

        while let Some(mut parser) = stack.pop() {
            parse::<_, As>(&mut parser, &mut stack, &mut output, args.clone())?;
        }

        output.shrink_to_fit();
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::BinTree;
    use tlb::bits::bitvec::bits;
    use tlb::bits::bitvec::order::Msb0;
    use tlb::r#as::{Data, NoArgs, Ref, Same};
    use tlb::ser::r#as::CellSerializeWrapAsExt;
    use tlb::ser::CellSerializeExt;

    #[test]
    fn bin_tree_leaf() {
        let data = bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1]
            .wrap_as::<Data>()
            .to_cell()
            .unwrap();

        let got: BinTree<u8> = data
            .parse_fully_as_with::<_, BinTree<Data<NoArgs<_>>>>(())
            .unwrap();

        assert_eq!(got.into_leaf(), Some(5));
    }

    #[test]
    fn bin_tree_fork() {
        let data = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();

        let [left, right] = data
            .parse_fully_as_with::<BinTree<u8>, BinTree<Data<NoArgs<_>>>>(())
            .unwrap()
            .into_fork()
            .unwrap();

        assert_eq!(left.into_leaf(), Some(5));
        assert_eq!(right.into_leaf(), Some(3));
    }

    #[test]
    fn bin_tree_as_vector_leaf() {
        let data = bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1]
            .wrap_as::<Data>()
            .to_cell()
            .unwrap();

        let got: Vec<u8> = data
            .parse_fully_as_with::<_, BinTree<Data<NoArgs<_>>>>(())
            .unwrap();

        assert_eq!(got, vec![5]);
    }

    #[test]
    fn bin_tree_as_vector_fork() {
        let data = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();

        let got: Vec<u8> = data
            .parse_fully_as_with::<_, BinTree<Data<NoArgs<_>>>>(())
            .unwrap();

        assert_eq!(got, vec![5, 3]);
    }

    #[test]
    fn bin_tree_as_vector_ordering() {
        let left_left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 0, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 0, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();
        let left_right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();
        let right_left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();
        let rigth_right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 1, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 1, 1].wrap_as::<Ref<Data>>(),
        )
            .to_cell()
            .unwrap();
        let left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            left_left_branch.wrap_as::<Ref<Same>>(),
            left_right_branch.wrap_as::<Ref<Same>>(),
        )
            .to_cell()
            .unwrap();
        let right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            right_left_branch.wrap_as::<Ref<Same>>(),
            rigth_right_branch.wrap_as::<Ref<Same>>(),
        )
            .to_cell()
            .unwrap();
        let root = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            left_branch.wrap_as::<Ref<Same>>(),
            right_branch.wrap_as::<Ref<Same>>(),
        )
            .to_cell()
            .unwrap();

        let got: Vec<u8> = root
            .parse_fully_as_with::<_, BinTree<Data<NoArgs<_>>>>(())
            .unwrap();

        assert_eq!(got, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}
