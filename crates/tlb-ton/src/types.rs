use num_bigint::BigUint;
use tlb::{BitPack, BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, VarUint};

pub type Coins = VarUint<4>;
pub type Grams = Coins;

/// currencies$_ grams:Grams other:ExtraCurrencyCollection = CurrencyCollection;
#[derive(Debug, Clone)]
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

/// extra_currencies$_ dict:(HashmapE 32 (VarUInteger 32)) = ExtraCurrencyCollection;
#[derive(Debug, Clone)]
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
    fn unpack<R>(_reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        // TODO
        Ok(Self)
    }
}
