use chrono::{DateTime, Utc};
use tlb::{
    Error,
    bits::{
        de::{BitReader, BitReaderExt, r#as::BitUnpackAs},
        ser::{BitWriter, BitWriterExt, r#as::BitPackAs},
    },
};

/// Adapter to **de**/**ser**ialize UNIX timestamp as `u32` from [`DateTime`]
pub struct UnixTimestamp;

impl BitPackAs<DateTime<Utc>> for UnixTimestamp {
    #[inline]
    fn pack_as<W>(source: &DateTime<Utc>, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let timestamp: u32 = source.timestamp().try_into().map_err(Error::custom)?;
        writer.pack(timestamp)?;
        Ok(())
    }
}

impl BitUnpackAs<DateTime<Utc>> for UnixTimestamp {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<DateTime<Utc>, R::Error>
    where
        R: BitReader,
    {
        let timestamp: u32 = reader.unpack()?;
        Ok(DateTime::from_timestamp(timestamp as i64, 0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use tlb::bits::{de::r#as::unpack_fully_as, ser::r#as::pack_as};

    use super::*;

    #[test]
    fn unix_timestamp_serde() {
        let ts = DateTime::UNIX_EPOCH;

        let packed = pack_as::<_, UnixTimestamp>(ts).unwrap();
        let got: DateTime<Utc> = unpack_fully_as::<_, UnixTimestamp>(packed).unwrap();

        assert_eq!(got, ts);
    }
}
