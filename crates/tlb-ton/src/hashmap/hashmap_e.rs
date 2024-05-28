use bitvec::{order::Msb0, vec::BitVec};
use tlb::{
    BitReader, BitReaderExt, Cell, CellDeserializeOwned, CellParser, CellParserError, Error, Ref,
};

use crate::Unary;

/// ```tlb
///  hm_edge#_ {n:#} {X:Type} {l:#} {m:#} label:(HmLabel ~l n)
/// {n = (~m) + l} node:(HashmapNode m X) = Hashmap n X;
/// ```
#[derive(Debug)]
struct Hashmap<T> {
    label: HmLabel,
    node: HashmapNode<T>,
}

impl<T> Hashmap<T> {
    fn parse_n<'de>(parser: &mut CellParser<'de>, n: u32) -> Result<Self, CellParserError<'de>>
    where
        T: CellDeserializeOwned,
    {
        let label = HmLabel::unpack_m(&mut *parser, n)?;
        let m = n - label.n;
        Ok(Self {
            label,
            node: HashmapNode::parse_n(parser, m)?,
        })
    }
}

/// ```tlb
/// hml_short$0 {m:#} {n:#} len:(Unary ~n) {n <= m} s:(n * Bit) = HmLabel ~n m;
/// hml_long$10 {m:#} n:(#<= m) s:(n * Bit) = HmLabel ~n m;
/// hml_same$11 {m:#} v:Bit n:(#<= m) = HmLabel ~n m;
/// ```
#[derive(Debug)]
struct HmLabel {
    n: u32,
    s: BitVec<u8, Msb0>,
}

impl HmLabel {
    fn unpack_m<R>(mut reader: R, m: u32) -> Result<Self, R::Error>
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
                    let n: u32 = reader.unpack_as_n_bits(m.ilog2() + 1)?;
                    Self {
                        n,
                        // s:(n * Bit)
                        s: reader.read_bitvec(n as usize)?,
                    }
                }
                // hml_same$11
                true => Self {
                    // v:Bit
                    s: reader.read_bitvec(1)?,
                    // n:(#<= m)
                    n: reader.unpack_as_n_bits(m.ilog2() + 1)?,
                },
            },
        })
    }
}

/// ```tlb
/// hmn_leaf#_ {X:Type} value:X = HashmapNode 0 X;
/// hmn_fork#_ {n:#} {X:Type} left:^(Hashmap n X)
///            right:^(Hashmap n X) = HashmapNode (n + 1) X;
/// ```
#[derive(Debug)]
enum HashmapNode<T> {
    Leaf(T),
    Fork([Box<Hashmap<T>>; 2]),
}

impl<T> HashmapNode<T> {
    pub fn parse_n<'de>(parser: &mut CellParser<'de>, n: u32) -> Result<Self, CellParserError<'de>>
    where
        T: CellDeserializeOwned,
    {
        if n == 0 {
            return parser.parse().map(Self::Leaf);
        }
        let [lc, rc]: [Cell; 2] = parser.parse_as::<_, [Ref; 2]>()?;
        let [mut lp, mut rp] = [&lc, &rc].map(Cell::parser);
        let [l, r] = [
            Hashmap::parse_n(&mut lp, n - 1)?,
            Hashmap::parse_n(&mut rp, n - 1)?,
        ];

        for p in [lp, rp] {
            p.ensure_empty()?;
        }
        Ok(Self::Fork([l, r].map(Into::into)))
    }
}

/// ```tlb
/// hme_empty$0 {n:#} {X:Type} = HashmapE n X;
/// hme_root$1 {n:#} {X:Type} root:^(Hashmap n X) = HashmapE n X;
/// ```
pub enum HashmapE<T> {
    Empty,
    Root(Hashmap<T>),
}

impl<T> HashmapE<T> {
    pub fn parse_n<'de>(parser: &mut CellParser<'de>, n: u32) -> Result<Self, CellParserError<'de>>
    where
        T: CellDeserializeOwned,
    {
        Ok(match parser.unpack()? {
            false => Self::Empty,
            true => {
                let cell: Cell = parser.parse_as::<_, Ref>()?;
                let mut parser = cell.parser();
                let root = Hashmap::parse_n(&mut parser, n)?;
                parser.ensure_empty()?;
                Self::Root(root)
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use bitvec::{bits, order::Msb0, vec::BitVec};
    use tlb::{
        Cell, CellDeserialize, CellDeserializeAsWrap, CellSerializeAsWrap, CellSerializeExt,
        CellSerializeWrapAsExt, Data, Ref,
    };

    use super::*;

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

        println!("{cell:#?}");

        let hm: HashmapE<CellDeserializeAsWrap<u16, Data>> =
            HashmapE::parse_n(&mut cell.parser(), 8).unwrap();

        // println!("{hm:#?}");
    }
}
