use num_traits::{ConstZero, One, ToPrimitive, Unsigned};

use crate::{
    Error,
    de::{BitReader, BitUnpackAs},
    ser::{BitPackAs, BitWriter, BitWriterExt},
};

/// [`Unary ~n`](https://docs.ton.org/develop/data-formats/tl-b-types#unary)
/// adapter
/// ```tlb
/// unary_zero$0 = Unary ~0;
/// unary_succ$1 {n:#} x:(Unary ~n) = Unary ~(n + 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Unary;

impl<T> BitPackAs<T> for Unary
where
    T: ToPrimitive + Unsigned,
{
    type Args = ();

    #[inline]
    fn pack_as<W>(num: &T, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer
            // unary_succ$1 {n:#} x:(Unary ~n) = Unary ~(n + 1);
            .with_repeat_bit(
                num.to_usize()
                    .ok_or_else(|| Error::custom("cannot be represented as usize"))?,
                true,
            )?
            // unary_zero$0 = Unary ~0;
            .pack(false, ())?;
        Ok(())
    }
}

impl<'de, T> BitUnpackAs<'de, T> for Unary
where
    T: Unsigned + ConstZero + One,
{
    type Args = ();

    #[inline]
    fn unpack_as<R>(reader: &mut R, _: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let mut n = T::ZERO;
        while reader.read_bit()?.ok_or_else(|| Error::custom("EOF"))? {
            n = n + T::one();
        }
        Ok(n)
    }
}
