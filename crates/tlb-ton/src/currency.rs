use num_bigint::BigUint;
use tlb::bits::{
    de::{BitReader, BitReaderExt, BitUnpack},
    r#as::VarUint,
    ser::{BitPack, BitWriter, BitWriterExt},
};

pub type Coins = VarUint<4>;
pub type Grams = Coins;

/// ```tlb
/// currencies$_ grams:Grams other:ExtraCurrencyCollection = CurrencyCollection;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurrencyCollection {
    pub grams: BigUint,
    pub other: ExtraCurrencyCollection,
}

impl BitPack for CurrencyCollection {
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer
            .pack_as::<_, &Grams>(&self.grams)?
            .pack(&self.other)?;
        Ok(())
    }
}

impl BitUnpack for CurrencyCollection {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        Ok(Self {
            grams: reader.unpack_as::<_, Grams>()?,
            other: reader.unpack()?,
        })
    }
}

/// ```tlb
/// extra_currencies$_ dict:(HashmapE 32 (VarUInteger 32)) = ExtraCurrencyCollection;
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraCurrencyCollection;

impl BitPack for ExtraCurrencyCollection {
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        // TODO
        false.pack(writer)
    }
}

impl BitUnpack for ExtraCurrencyCollection {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        // TODO
        let _: bool = reader.unpack()?;
        Ok(Self)
    }
}

#[cfg(test)]
mod tests {
    use tlb::bits::{de::unpack_fully, ser::pack};

    use super::*;

    #[test]
    fn currency_collection_serde() {
        let v = CurrencyCollection {
            grams: BigUint::ZERO,
            other: ExtraCurrencyCollection,
        };

        let packed = pack(v.clone()).unwrap();
        let got: CurrencyCollection = unpack_fully(packed).unwrap();

        assert_eq!(got, v);
    }
}
