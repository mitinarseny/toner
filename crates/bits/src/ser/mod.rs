mod args;
mod r#as;
mod writer;

pub use self::{args::*, r#as::*, writer::*};

use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use impl_tools::autoimpl;

use crate::{AsBytes, ResultExt, StringError};

#[autoimpl(for<S: trait + ?Sized> &S, &mut S, Box<S>, Rc<S>, Arc<S>)]
pub trait BitPack {
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter;
}

#[inline]
pub fn pack<T>(value: T) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPack,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack(&mut writer, value)?;
    Ok(writer)
}

#[inline]
pub fn pack_with<T>(value: T, args: T::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    T: BitPackWithArgs,
{
    let mut writer = BitVec::new();
    BitWriterExt::pack_with(&mut writer, value, args)?;
    Ok(writer)
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
        writer.write_bit(*self).map_err(Into::into)
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
    fn pack<W>(&self, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        self.as_slice().pack(writer)
    }
}

macro_rules! impl_bit_serialize_for_tuple {
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
impl_bit_serialize_for_tuple!(0:T0);
impl_bit_serialize_for_tuple!(0:T0,1:T1);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_bit_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'a> BitPack for &'a BitSlice<u8, Msb0> {
    #[inline]
    fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.write_bitslice(self)
    }
}

impl BitPack for BitVec<u8, Msb0> {
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
