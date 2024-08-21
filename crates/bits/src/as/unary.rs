use num_traits::{ConstZero, One, ToPrimitive, Unsigned};

use crate::{
    de::{r#as::BitUnpackAs, BitReader},
    ser::{r#as::BitPackAs, BitWriter, BitWriterExt},
    Error,
};

/// [`Unary ~n`](https://docs.ton.org/develop/data-formats/tl-b-types#unary)
/// adapter
/// ```tlb
/// unary_zero$0 = Unary ~0;
/// unary_succ$1 {n:#} x:(Unary ~n) = Unary ~(n + 1);
/// ```
pub struct Unary;

impl<T> BitPackAs<T> for Unary
where
    T: ToPrimitive + Unsigned,
{
    #[inline]
    fn pack_as<W>(num: &T, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer
            // unary_succ$1 {n:#} x:(Unary ~n) = Unary ~(n + 1);
            .with_repeat_bit(
                num.to_usize()
                    .ok_or_else(|| Error::custom("cannot be represented as usize"))?,
                true,
            )?
            // unary_zero$0 = Unary ~0;
            .pack(false)?;
        Ok(())
    }
}

impl<T> BitUnpackAs<T> for Unary
where
    T: Unsigned + ConstZero + One,
{
    #[inline]
    fn unpack_as<R>(mut reader: R) -> Result<T, R::Error>
    where
        R: BitReader,
    {
        let mut n = T::ZERO;
        while reader.read_bit()?.ok_or_else(|| Error::custom("EOF"))? {
            n = n + T::one();
        }
        Ok(n)
    }
}
