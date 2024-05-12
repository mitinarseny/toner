use chrono::{DateTime, Utc};
use tlb::{BitPack, BitPackAs, BitReaderExt, BitUnpackAs, BitWriter, Error};

pub struct UnixTimestamp;

impl BitPackAs<DateTime<Utc>> for UnixTimestamp {
    fn pack_as<W>(source: &DateTime<Utc>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let timestamp: u32 = source.timestamp().try_into().map_err(Error::custom)?;
        timestamp.pack(writer)
    }
}

impl BitUnpackAs<DateTime<Utc>> for UnixTimestamp {
    fn unpack_as<R>(mut reader: R) -> Result<DateTime<Utc>, R::Error>
    where
        R: tlb::BitReader,
    {
        let timestamp: u32 = reader.unpack()?;
        Ok(DateTime::from_timestamp(timestamp as i64, 0).unwrap())
    }
}
