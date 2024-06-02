use core::marker::PhantomData;
use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, vec::BitVec};
use either::Either;

use crate::{ResultExt, StringError};

use super::{BitPack, BitWriter, BitWriterExt};

pub trait BitPackAs<T: ?Sized> {
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter;
}

#[inline]
pub fn pack_as<T, As>(value: T) -> Result<BitVec<u8, Msb0>, StringError>
where
    As: BitPackAs<T> + ?Sized,
{
    let mut writer = BitVec::new();
    writer.pack_as::<_, As>(value)?;
    Ok(writer)
}

pub struct PackAsWrap<'a, T, As>
where
    As: ?Sized,
    T: ?Sized,
{
    value: &'a T,
    _phantom: PhantomData<As>,
}

impl<'a, T, As> PackAsWrap<'a, T, As>
where
    T: ?Sized,
    As: ?Sized,
{
    #[inline]
    pub const fn new(value: &'a T) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }

    #[inline]
    pub const fn into_inner(&'a self) -> &'a T {
        self.value
    }
}

impl<'a, T, As> BitPack for PackAsWrap<'a, T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: BitPackAs<T>,
{
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as(self.value, writer)
    }
}

impl<'a, T, As> BitPackAs<&'a T> for &'a As
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &&T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        PackAsWrap::<T, As>::new(source).pack(writer)
    }
}

impl<'a, T, As> BitPackAs<&'a mut T> for &'a mut As
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &&mut T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        PackAsWrap::<T, As>::new(source).pack(writer)
    }
}

impl<T, As> BitPackAs<[T]> for [As]
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &[T], mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        for (i, v) in source.iter().enumerate() {
            writer
                .pack_as::<&T, &As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(())
    }
}

impl<T, As, const N: usize> BitPackAs<[T; N]> for [As; N]
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &[T; N], mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<&[T], &[As]>(source)?;
        Ok(())
    }
}

macro_rules! impl_bit_pack_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> BitPackAs<($($t,)+)> for ($($a,)+)
        where $(
            $a: BitPackAs<$t>,
        )+
        {
            #[inline]
            fn pack_as<W>(source: &($($t,)+), mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                writer$(
                    .pack_as::<&$t, &$a>(&source.$n)?)+;
                Ok(())
            }
        }
    };
}
impl_bit_pack_as_for_tuple!(0:T0 as As0);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_bit_pack_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As> BitPackAs<Box<T>> for Box<As>
where
    As: BitPackAs<T> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &Box<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        PackAsWrap::<T, As>::new(source).pack(writer)
    }
}

impl<T, As> BitPackAs<Rc<T>> for Rc<As>
where
    As: BitPackAs<T> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &Rc<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        PackAsWrap::<T, As>::new(source).pack(writer)
    }
}

impl<T, As> BitPackAs<Arc<T>> for Arc<As>
where
    As: BitPackAs<T> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &Arc<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        PackAsWrap::<T, As>::new(source).pack(writer)
    }
}

impl<Left, Right, AsLeft, AsRight> BitPackAs<Either<Left, Right>> for Either<AsLeft, AsRight>
where
    AsLeft: BitPackAs<Left>,
    AsRight: BitPackAs<Right>,
{
    #[inline]
    fn pack_as<W>(source: &Either<Left, Right>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .as_ref()
            .map_either(
                PackAsWrap::<Left, AsLeft>::new,
                PackAsWrap::<Right, AsRight>::new,
            )
            .pack(writer)
    }
}

impl<T, As> BitPackAs<Option<T>> for Either<(), As>
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match source.as_ref() {
            None => Either::Left(()),
            Some(v) => Either::Right(PackAsWrap::<T, As>::new(v)),
        }
        .pack(writer)
    }
}

impl<T, As> BitPackAs<Option<T>> for Option<As>
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.as_ref().map(PackAsWrap::<T, As>::new).pack(writer)
    }
}

pub trait BitPackWrapAsExt {
    #[inline]
    fn wrap_as<As>(&self) -> PackAsWrap<'_, Self, As>
    where
        As: BitPackAs<Self> + ?Sized,
    {
        PackAsWrap::new(self)
    }
}
impl<T> BitPackWrapAsExt for T {}
