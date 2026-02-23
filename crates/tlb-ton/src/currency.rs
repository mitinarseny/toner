//! Collection of types to work with currencies
use lazy_static::lazy_static;
use num_bigint::BigUint;
use num_traits::One;
use tlb::{
    Data, Same,
    bits::{VarInt, de::BitReaderExt, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    hashmap::HashmapE,
    ser::{CellBuilder, CellBuilderError, CellSerialize},
};

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
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CurrencyCollection {
    pub grams: BigUint,
    #[cfg_attr(feature = "arbitrary", arbitrary(default))]
    pub other: ExtraCurrencyCollection,
}

impl CellSerialize for CurrencyCollection {
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder
            .pack_as::<_, &Grams>(&self.grams, ())?
            .store(&self.other, ())?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for CurrencyCollection {
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            grams: parser.unpack_as::<_, Grams>(())?,
            other: parser.parse(())?,
        })
    }
}

/// ```tlb
/// extra_currencies$_ dict:(HashmapE 32 (VarUInteger 32)) = ExtraCurrencyCollection;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtraCurrencyCollection(pub HashmapE<BigUint>);

impl CellSerialize for ExtraCurrencyCollection {
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder.store_as::<_, &HashmapE<Data<VarInt<32>>, Same>>(&self.0, (32, (), ()))?;
        Ok(())
    }
}

impl<'de> CellDeserialize<'de> for ExtraCurrencyCollection {
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(Self(
            parser.parse_as::<_, HashmapE<Data<VarInt<32>>, Same>>((32, (), ()))?,
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

        let cell = v.to_cell(()).unwrap();
        let got: CurrencyCollection = cell.parse_fully(()).unwrap();

        assert_eq!(got, v);
    }
}
