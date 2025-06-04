//! Binary **ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
pub mod args;
pub mod r#as;
mod writer;

pub use self::writer::*;

use std::{borrow::Cow, rc::Rc, sync::Arc};

use args::r#as::BitPackAsWithArgs;
use r#as::BitPackAs;
use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use either::Either;
use impl_tools::autoimpl;

use crate::{
    Context, StringError,
    r#as::{AsBytes, Same, args::NoArgs},
};

use self::args::BitPackWithArgs;

/// A type that can be bitwise-**ser**ilalized into any [`BitWriter`].
#[autoimpl(for<T: trait + ToOwned + ?Sized> Cow<'_, T>)]
#[autoimpl(for<T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait BitPack {
    /// Pack value into the writer.
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter;
}

/// **Ser**ialize given value into [`BitVec`]
#[inline]
pub fn pack<T>(value: T) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPack,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack(&mut writer, value)?;
    Ok(writer)
}

/// Serialize given value with args into [`BitVec`]
#[inline]
pub fn pack_with<T>(value: T, args: T::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPackWithArgs,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack_with(&mut writer, value, args)?;
    Ok(writer)
}

#[inline]
pub fn bits_for<T>(value: T) -> Result<usize, StringError>
where
    T: BitPack,
{
    bits_for_as::<_, Same>(value)
}

#[inline]
pub fn bits_for_as<T, As>(value: T) -> Result<usize, StringError>
where
    As: BitPackAs<T>,
{
    bits_for_as_with::<_, NoArgs<_, As>>(value, ())
}

#[inline]
pub fn bits_for_as_with<T, As>(value: T, args: As::Args) -> Result<usize, StringError>
where
    As: BitPackAsWithArgs<T>,
{
    let mut writer = NoopBitWriter.counted();
    BitWriterExt::pack_as_with::<_, As>(&mut writer, value, args)?;
    Ok(writer.bit_count())
}

impl BitPack for () {
    #[inline]
    fn pack<W>(&self, _writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        Ok(())
    }
}

impl BitPack for bool {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.write_bit(*self)
    }
}

impl<T> BitPack for [T]
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_many(self)?;
        Ok(())
    }
}

impl<T, const N: usize> BitPack for [T; N]
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack(writer)
    }
}

impl<T> BitPack for Vec<T>
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack(writer)
    }
}

macro_rules! impl_bit_pack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitPack for ($($t,)+)
        where $(
            $t: BitPack,
        )+
        {
            #[inline]
            fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                $(self.$n.pack(&mut writer).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_bit_pack_for_tuple!(0:T0);
impl_bit_pack_for_tuple!(0:T0,1:T1);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_pack_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<L, R> BitPack for Either<L, R>
where
    L: BitPack,
    R: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        match self {
            Self::Left(l) => writer.pack(false).context("tag")?.pack(l).context("left")?,
            Self::Right(r) => writer.pack(true).context("tag")?.pack(r).context("right")?,
        };
        Ok(())
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T> BitPack for Option<T>
where
    T: BitPack,
{
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

impl BitPack for BitSlice<u8, Msb0> {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.write_bitslice(self)
    }
}

impl BitPack for BitVec<u8, Msb0> {
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_bitslice().pack(writer)
    }
}

impl BitPack for str {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_as::<_, AsBytes>(self)?;
        Ok(())
    }
}

impl BitPack for String {
    #[inline]
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_str().pack(writer)
    }
}
