use tlb::{Cell, CellDeserializeAs, CellDeserializeAsOwned, CellParser, CellParserError, Ref};
use tlbits::BitReaderExt;

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

#[cfg(test)]
mod tests {
    use bitvec::bits;
    use bitvec::order::Msb0;
    use tlb::{CellSerializeExt, CellSerializeWrapAsExt, Data};
    use crate::BinTree;

    impl<I> BinTree<I> {
        pub fn unwrap_leaf(self) -> I {
            match self {
                BinTree::Leaf(x) => x,
                _ => panic!("expected leaf, got fork"),
            }
        }
    }

    #[test]
    fn bin_tree_leaf() {
        let data = (bits![u8, Msb0; 0, 0, 0, 0, 0, 0, 1, 0, 1].wrap_as::<Data>()).to_cell().unwrap();

        let got: BinTree<u8> = data.parse_fully_as::<_, BinTree<Data>>().unwrap();

        assert_eq!(got.unwrap_leaf(), 5);
    }
}


