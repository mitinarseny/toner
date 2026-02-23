//! Binary **ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
mod r#as;
mod writer;

pub use self::{r#as::*, writer::*};

use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{array::BitArray, order::Msb0, vec::BitVec, view::BitViewSized};
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

impl BitPack for () {
    type Args = ();

    #[inline]
    fn pack<W>(&self, _writer: &mut W, _: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Ok(())
    }
}

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

// impl<T> BitPack for [T]
// where
//     T: BitPack,
//     T::Args: Clone,
// {
//     type Args = T::Args;

//     #[inline]
//     fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         // TODO: save bytes
//         // u32::try_from(self.len())
//         //     .map_err(Error::custom)
//         //     .context("length")?
//         //     .pack(writer)?;

//         writer.pack_many(self, args)?;
//         Ok(())
//     }
// }

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

macro_rules! impl_bit_pack_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> BitPack for ($($t,)+)
        where $(
            $t: BitPack,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
            where
                W: BitWriter + ?Sized,
            {
                $(self.$n.pack(writer, args.$n).context(concat!(".", stringify!($n)))?;)+
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

// TODO
// impl BitPack for BitSlice<u8, Msb0> {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         writer.write_bitslice(self)
//     }
// }

// impl BitPack for BitVec<u8, Msb0> {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         self.as_bitslice().pack(writer)
//     }
// }

// impl BitPack for str {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         // TODO: len
//         writer.pack_as::<_, AsBytes>(self)?;
//         Ok(())
//     }
// }

// impl BitPack for String {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         self.as_str().pack(writer)
//     }
// }

// impl<T> BitPack for Vec<T>
// where
//     T: BitPack,
//     T::Args: Clone,
// {
//     type Args = T::Args;

//     #[inline]
//     fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         self.as_slice().pack(writer, args)
//     }
// }

// impl<T> BitPack for VecDeque<T>
// where
//     T: BitPack,
// {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         let (a, b) = self.as_slices();
//         writer.pack_many(a)?.pack_many(b)?;

//         Ok(())
//     }
// }

// impl<T> BitPack for LinkedList<T>
// where
//     T: BitPack,
// {
//     #[inline]
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         writer.pack_many(self)?;

//         Ok(())
//     }
// }

// impl<T> BitPack for BTreeSet<T>
// where
//     T: BitPack,
// {
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         writer.pack_many(self)?;

//         Ok(())
//     }
// }

// impl<K, V> BitPack for BTreeMap<K, V>
// where
//     K: BitPack,
//     V: BitPack,
// {
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         writer.pack_many(self)?;

//         Ok(())
//     }
// }

// impl<T> BitPack for HashSet<T>
// where
//     T: BitPack,
// {
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         writer.pack_many(self)?;

//         Ok(())
//     }
// }

// impl<K, V> BitPack for HashMap<K, V>
// where
//     K: BitPack,
//     V: BitPack,
// {
//     fn pack<W>(&self, writer: &mut W) -> Result<(), W::Error>
//     where
//         W: BitWriter + ?Sized,
//     {
//         u32::try_from(self.len())
//             .map_err(Error::custom)
//             .context("length")?
//             .pack(writer)?;

//         writer.pack_many(self)?;

//         Ok(())
//     }
// }
