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

#[cfg(feature = "arbitrary")]
impl UnixTimestamp {
    #[inline]
    pub fn arbitrary(u: &mut ::arbitrary::Unstructured) -> ::arbitrary::Result<DateTime<Utc>> {
        Ok(DateTime::from_timestamp(
            u.int_in_range(
                DateTime::UNIX_EPOCH.timestamp()..=DateTime::<Utc>::MAX_UTC.timestamp(),
            )?,
            0,
        )
        .unwrap_or_else(|| unreachable!()))
    }

    #[inline]
    pub fn arbitrary_option(
        u: &mut ::arbitrary::Unstructured,
    ) -> ::arbitrary::Result<Option<DateTime<Utc>>> {
        use arbitrary::Arbitrary;

        Option::<()>::arbitrary(u)?
            .map(|()| Self::arbitrary(u))
            .transpose()
    }
}

impl BitPackAs<DateTime<Utc>> for UnixTimestamp {
    #[inline]
    fn pack_as<W>(source: &DateTime<Utc>, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let timestamp: u32 = source
            .timestamp()
            .try_into()
            .map_err(|_| Error::custom("timestamp: overflow"))?;
        writer.pack(timestamp)?;
        Ok(())
    }
}

impl<'de> BitUnpackAs<'de, DateTime<Utc>> for UnixTimestamp {
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<DateTime<Utc>, R::Error>
    where
        R: BitReader<'de>,
    {
        let timestamp: u32 = reader.unpack()?;
        DateTime::from_timestamp(timestamp as i64, 0)
            .ok_or_else(|| Error::custom("timestamp: overflow"))
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
        let got: DateTime<Utc> = unpack_fully_as::<_, UnixTimestamp>(&packed).unwrap();

        assert_eq!(got, ts);
    }
}
