//! Collection of types to work with currencies
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::One;
use tlb::{
    bits::{de::BitReaderExt, r#as::VarInt, ser::BitWriterExt},
    de::{CellDeserialize, OrdinaryCellParser, OrdinaryCellParserError},
    r#as::{Data, NoArgs},
    ser::{CellBuilder, CellBuilderError, CellSerialize},
};

use crate::hashmap::HashmapE;

lazy_static! {
    /// 1 gram (nano-TON)
    pub static ref ONE_GRAM: BigUint = BigUint::one();
    /// 1 TON
    pub static ref ONE_TON: BigUint = &*ONE_GRAM * 1_000_000_000u64;
}

/// Alias for `VarUInteger 16`
/// ```tlb
/// nanograms$_ amount:(VarUInteger 16) = Grams;
/// ```
pub type Coins = VarInt<4>;

/// Alias for `VarUInteger 16`
/// ```tlb
/// nanograms$_ amount:(VarUInteger 16) = Grams;
/// ```
pub type Grams = Coins;

/// [`CurrencyCollection`](https://docs.ton.org/develop/data-formats/msg-tlb#currencycollection)
/// ```tlb
/// currencies$_ grams:Grams other:ExtraCurrencyCollection = CurrencyCollection;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurrencyCollection {
    pub grams: BigUint,
    pub other: ExtraCurrencyCollection,
}

impl CellSerialize for CurrencyCollection {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack_as::<_, &Grams>(&self.grams)?
            .store(&self.other)?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for CurrencyCollection {
    #[inline]
    fn parse(parser: &mut OrdinaryCellParser<'de>) -> Result<Self, OrdinaryCellParserError<'de>> {
        Ok(Self {
            grams: parser.unpack_as::<_, Grams>()?,
            other: parser.parse()?,
        })
    }
}

/// ```tlb
/// extra_currencies$_ dict:(HashmapE 32 (VarUInteger 32)) = ExtraCurrencyCollection;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtraCurrencyCollection(pub HashmapE<BigUint>);

impl CellSerialize for ExtraCurrencyCollection {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, &HashmapE<NoArgs<_, Data<VarInt<32>>>, NoArgs<_>>>(
            &self.0,
            (32, (), ()),
        )?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for ExtraCurrencyCollection {
    #[inline]
    fn parse(parser: &mut OrdinaryCellParser<'de>) -> Result<Self, OrdinaryCellParserError<'de>> {
        Ok(Self(
            parser.parse_as_with::<_, HashmapE<NoArgs<_, Data<VarInt<32>>>, NoArgs<_>>>((
                32,
                (),
                (),
            ))?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use tlb::ser::CellSerializeExt;

    use super::*;

    #[test]
    fn currency_collection_serde() {
        let v = CurrencyCollection::default();

        let cell = v.to_cell().unwrap();
        let got: CurrencyCollection = cell.parse_fully().unwrap();

        assert_eq!(got, v);
    }
}
