use time::UtcDateTime;
use tlb::{
    Error,
    bits::{
        de::{BitReader, BitReaderExt, BitUnpackAs},
        ser::{BitPackAs, BitWriter, BitWriterExt},
    },
};

/// Adapter to **de**/**ser**ialize UNIX timestamp as `u32` from [`DateTime`]
pub struct UnixTimestamp;

impl BitPackAs<UtcDateTime> for UnixTimestamp {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &UtcDateTime, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let timestamp: u32 = source
            .unix_timestamp()
            .try_into()
            .map_err(|_| Error::custom("timestamp: overflow"))?;
        writer.pack(timestamp, ())?;
        Ok(())
    }
}

impl<'de> BitUnpackAs<'de, UtcDateTime> for UnixTimestamp {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<UtcDateTime, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let timestamp: u32 = reader.unpack(())?;
        UtcDateTime::from_unix_timestamp(timestamp as i64).map_err(Error::custom)
    }
}

#[cfg(feature = "arbitrary")]
const _: () = {
    use arbitrary::{Result, Unstructured};
    use arbitrary_with::ArbitraryAs;

    impl<'a> ArbitraryAs<'a, UtcDateTime> for UnixTimestamp {
        fn arbitrary_as(u: &mut Unstructured<'a>) -> Result<UtcDateTime> {
            Ok(UtcDateTime::from_unix_timestamp(u.int_in_range(
                UtcDateTime::UNIX_EPOCH.unix_timestamp()..=UtcDateTime::MAX.unix_timestamp(),
            )?)
            .unwrap_or_else(|_| unreachable!()))
        }
    }
};

#[cfg(test)]
mod tests {
    use time::UtcDateTime;
    use tlb::bits::{de::unpack_fully_as, ser::pack_as};

    use super::*;

    #[test]
    fn unix_timestamp_serde() {
        let ts = UtcDateTime::UNIX_EPOCH;

        let packed = pack_as::<_, UnixTimestamp>(ts, ()).unwrap();
        let got: UtcDateTime = unpack_fully_as::<_, UnixTimestamp>(&packed, ()).unwrap();

        assert_eq!(got, ts);
    }
}
