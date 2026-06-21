use jiff::Timestamp;
use tlb::{
    Error,
    bits::{
        de::{BitReader, BitReaderExt, BitUnpackAs},
        ser::{BitPackAs, BitWriter, BitWriterExt},
    },
};

/// Adapter to **de**/**ser**ialize UNIX timestamp as `u32` from [`DateTime`]
pub struct UnixTimestampSeconds;

impl BitPackAs<Timestamp> for UnixTimestampSeconds {
    type Args = ();

    #[inline]
    fn pack_as<W>(source: &Timestamp, writer: &mut W, (): Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let timestamp: u32 = source
            .as_second()
            .try_into()
            .map_err(|_| Error::custom("timestamp: overflow"))?;
        writer.pack(timestamp, ())?;
        Ok(())
    }
}

impl<'de> BitUnpackAs<'de, Timestamp> for UnixTimestampSeconds {
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, (): Self::Args) -> Result<Timestamp, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let timestamp: u32 = reader.unpack(())?;
        Timestamp::from_second(timestamp as i64).map_err(Error::custom)
    }
}

#[cfg(feature = "arbitrary")]
const _: () = {
    use arbitrary::{Result, Unstructured};
    use arbitrary_with::ArbitraryAs;

    impl<'a> ArbitraryAs<'a, Timestamp> for UnixTimestampSeconds {
        fn arbitrary_as(u: &mut Unstructured<'a>) -> Result<Timestamp> {
            Ok(Timestamp::from_second(
                u.int_in_range(Timestamp::UNIX_EPOCH.as_second()..=Timestamp::MAX.as_second())?,
            )
            .unwrap_or_else(|_| unreachable!()))
        }
    }
};

#[cfg(test)]
mod tests {

    use tlb::bits::{de::unpack_fully_as, ser::pack_as};

    use super::*;

    #[test]
    fn unix_timestamp_serde() {
        let ts = Timestamp::UNIX_EPOCH;

        let packed = pack_as::<_, UnixTimestampSeconds>(ts, ()).unwrap();
        let got: Timestamp = unpack_fully_as::<_, UnixTimestampSeconds>(&packed, ()).unwrap();

        assert_eq!(got, ts);
    }
}
