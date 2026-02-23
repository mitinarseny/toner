use chrono::{DateTime, Utc};
use tlb::{
    Error,
    bits::{
        de::{BitReader, BitReaderExt, BitUnpackAs},
        ser::{BitPackAs, BitWriter, BitWriterExt},
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
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &DateTime<Utc>, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let timestamp: u32 = source
            .timestamp()
            .try_into()
            .map_err(|_| Error::custom("timestamp: overflow"))?;
        writer.pack(timestamp, ())?;
        Ok(())
    }
}

impl<'de> BitUnpackAs<'de, DateTime<Utc>> for UnixTimestamp {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<DateTime<Utc>, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let timestamp: u32 = reader.unpack(())?;
        DateTime::from_timestamp(timestamp as i64, 0)
            .ok_or_else(|| Error::custom("timestamp: overflow"))
    }
}

#[cfg(test)]
mod tests {
    use tlb::bits::{de::unpack_fully_as, ser::pack_as};

    use super::*;

    #[test]
    fn unix_timestamp_serde() {
        let ts = DateTime::UNIX_EPOCH;

        let packed = pack_as::<_, UnixTimestamp>(ts, ()).unwrap();
        let got: DateTime<Utc> = unpack_fully_as::<_, UnixTimestamp>(&packed, ()).unwrap();

        assert_eq!(got, ts);
    }
}
