use tlb::{Cell, CellDeserializeAs, CellDeserializeAsOwned, CellParser, CellParserError, Ref};
use tlbits::{BitReader, BitReaderExt};

/// ```tlb
/// bt_leaf$0 {X:Type} leaf:X = BinTree X;
/// bt_fork$1 {X:Type} left:^(BinTree X) right:^(BinTree X) = BinTree X;
/// ```
#[derive(Debug, Clone)]
pub enum BinTree<X> {
    Leaf(X),
    Fork([Box<BinTree<X>>; 2]),
}

impl<'de, T, As> CellDeserializeAs<'de, BinTree<T>> for BinTree<As> where As: CellDeserializeAsOwned<T> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<BinTree<T>, CellParserError<'de>> {
        match parser.unpack()? {
            false => { Ok(BinTree::Leaf(parser.parse_as::<T, As>()?)) },
            true => {
                let [lc, rc]: [Cell; 2] = parser.parse_as::<_, [Ref; 2]>()?;
                let l = lc.parse_fully_as::<BinTree<T>, BinTree<As>>()?;
                let r = rc.parse_fully_as::<BinTree<T>, BinTree<As>>()?;

                Ok(BinTree::Fork([l, r].map(Into::into)))
            }
        }
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Vec<T>> for BinTree<As> where As: CellDeserializeAsOwned<T> {
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Vec<T>, CellParserError<'de>> {
        #[inline]
        fn unpack<'a, T, As: CellDeserializeAsOwned<T>>(
            output: &'a mut Vec<T>,
            stack: &'a mut Vec<Cell>,
            parser: &'a mut CellParser<'_>
        ) -> Result<(), <CellParser<'a> as BitReader>::Error> {
            match parser.unpack()? {
                false => output.push(parser.parse_as::<T, As>()?),
                true => {
                    let [lc, rc]: [Cell; 2] = parser.parse_as::<_, [Ref; 2]>()?;
                    stack.push(rc);
                    stack.push(lc);
                }
            }
            Ok(())
        }

        let mut output = Vec::new();
        let mut stack = Vec::new();

        unpack::<T, As>(&mut output, &mut stack, parser)?;

        while let Some(cell) = stack.pop() {
            let mut parser = cell.parser();
            unpack::<T, As>(&mut output, &mut stack, &mut parser)?;
        }

        output.shrink_to_fit();

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use bitvec::bits;
    use bitvec::order::Msb0;
    use tlb::{CellSerializeExt, CellSerializeWrapAsExt, Data, Ref};
    use tlbits::Same;
    use crate::BinTree;

    impl<I> BinTree<I> {
        pub fn unwrap_leaf(self) -> I {
            match self {
                BinTree::Leaf(x) => x,
                _ => panic!("expected leaf, got fork"),
            }
        }

        pub fn unwrap_fork(self) -> [BinTree<I>; 2] {
            match self {
                BinTree::Fork(x) => x.map(|x| *x),
                _ => panic!("expected fork, got leaf"),
            }
        }
    }

    #[test]
    fn bin_tree_leaf() {
        let data = bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Data>().to_cell().unwrap();

        let got: BinTree<u8> = data.parse_fully_as::<_, BinTree<Data>>().unwrap();

        assert_eq!(got.unwrap_leaf(), 5);
    }

    #[test]
    fn bin_tree_fork() {
        let data = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();

        let [left, right] = data.parse_fully_as::<BinTree<u8>, BinTree<Data>>()
            .unwrap()
            .unwrap_fork();

        assert_eq!(left.unwrap_leaf(), 5);
        assert_eq!(right.unwrap_leaf(), 3);
    }

    #[test]
    fn bin_tree_as_vector_leaf() {
        let data = bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Data>().to_cell().unwrap();

        let got: Vec<u8> = data.parse_fully_as::<_, BinTree<Data>>().unwrap();

        assert_eq!(got, vec![5]);
    }

    #[test]
    fn bin_tree_as_vector_fork() {
        let data = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();

        let got: Vec<u8> = data.parse_fully_as::<_, BinTree<Data>>().unwrap();

        assert_eq!(got, vec![5, 3]);
    }

    #[test]
    fn bin_tree_as_vector_ordering() {
        let left_left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 0, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 0, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();
        let left_right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 0, 1, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();
        let right_left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();
        let rigth_right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 1, 0].wrap_as::<Ref<Data>>(),
            bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 1, 1].wrap_as::<Ref<Data>>()
        ).to_cell().unwrap();
        let left_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            left_left_branch.wrap_as::<Ref<Same>>(),
            left_right_branch.wrap_as::<Ref<Same>>(),
        ).to_cell().unwrap();
        let right_branch = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            right_left_branch.wrap_as::<Ref<Same>>(),
            rigth_right_branch.wrap_as::<Ref<Same>>(),
        ).to_cell().unwrap();
        let root = (
            bits![u8, Msb0; 1].wrap_as::<Data>(),
            left_branch.wrap_as::<Ref<Same>>(),
            right_branch.wrap_as::<Ref<Same>>(),
        ).to_cell().unwrap();

        let got: Vec<u8> = root.parse_fully_as::<_, BinTree<Data>>().unwrap();

        assert_eq!(got, vec![0, 1, 2, 3, 4, 5, 6, 7]);
    }
}


