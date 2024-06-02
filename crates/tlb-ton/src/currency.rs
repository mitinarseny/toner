use num_bigint::BigUint;
use tlb::{
    bits::{de::BitReaderExt, r#as::VarInt, ser::BitWriterExt},
    de::{CellDeserialize, CellParser, CellParserError},
    r#as::{Data, NoArgs},
    ser::{CellBuilder, CellBuilderError, CellSerialize},
};

use crate::hashmap::HashmapE;

pub type Coins = VarInt<4>;
pub type Grams = Coins;

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
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
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
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
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
