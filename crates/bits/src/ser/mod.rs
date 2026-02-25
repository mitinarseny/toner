//! Binary **ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
mod r#as;
mod writer;

pub use self::{r#as::*, writer::*};

use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{array::BitArray, order::Msb0, slice::BitSlice, vec::BitVec, view::BitViewSized};
use either::Either;
use impl_tools::autoimpl;

use crate::{Context, StringError, r#as::Same};

/// A type that can be bitwise-**ser**ilalized into any [`BitWriter`].
#[autoimpl(for<T: trait + ToOwned + ?Sized> Cow<'_, T>)]
#[autoimpl(for<T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait BitPack {
    type Args;

    /// Packs the value into given writer with args
    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized;
}

/// Serialize given value with args into [`BitVec`]
#[inline]
pub fn pack<T>(value: T, args: T::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPack,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack(&mut writer, value, args)?;
    Ok(writer)
}

#[inline]
pub fn bits_for<T>(value: T, args: T::Args) -> Result<usize, StringError>
where
    T: BitPack,
{
    bits_for_as::<_, Same>(value, args)
}

#[inline]
pub fn bits_for_as<T, As>(value: T, args: As::Args) -> Result<usize, StringError>
where
    As: BitPackAs<T> + ?Sized,
{
    let mut writer = NoopBitWriter.counted();
    BitWriterExt::pack_as::<_, As>(&mut writer, value, args)?;
    Ok(writer.bit_count())
}

macro_rules! impl_bit_pack_for_tuple {
    ($($t:ident:$n:tt),*) => {
        impl<$($t),*> BitPack for ($($t,)*)
        where $(
            $t: BitPack,
        )*
        {
            type Args = ($($t::Args,)*);

            #[inline]
            #[allow(unused_variables)]
            fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
            where
                W: BitWriter + ?Sized,
            {
                $(self.$n.pack(writer, args.$n).context(concat!(".", stringify!($n)))?;)*
                Ok(())
            }
        }
    };
}
impl_bit_pack_for_tuple!();
impl_bit_pack_for_tuple!(T0:0);
impl_bit_pack_for_tuple!(T0:0,T1:1);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4,T5:5);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4,T5:5,T6:6);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4,T5:5,T6:6,T7:7);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4,T5:5,T6:6,T7:7,T8:8);
impl_bit_pack_for_tuple!(T0:0,T1:1,T2:2,T3:3,T4:4,T5:5,T6:6,T7:7,T8:8,T9:9);

impl BitPack for bool {
    type Args = ();

    #[inline]
    fn pack<W>(&self, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.write_bit(*self)
    }
}

impl<T, const N: usize> BitPack for [T; N]
where
    T: BitPack,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_many(self, args)?;
        Ok(())
    }
}

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
    type Args = (L::Args, R::Args);

    #[inline]
    fn pack<W>(&self, writer: &mut W, (la, ra): Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        match self {
            Self::Left(l) => writer
                .pack(false, ())
                .context("tag")?
                .pack(l, la)
                .context("left")?,
            Self::Right(r) => writer
                .pack(true, ())
                .context("tag")?
                .pack(r, ra)
                .context("right")?,
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
    type Args = T::Args;

    #[inline]
    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_as::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}

impl BitPack for BitSlice<u8, Msb0> {
    type Args = ();

    fn pack<W>(&self, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.write_bitslice(self)
    }
}

impl<A> BitPack for BitArray<A, Msb0>
where
    A: BitViewSized<Store = u8>,
{
    type Args = ();

    fn pack<W>(&self, writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.write_bitslice(self.as_bitslice())
    }
}
