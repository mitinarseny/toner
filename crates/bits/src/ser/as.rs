use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{order::Msb0, vec::BitVec};
use either::Either;

use crate::{Context, StringError, r#as::AsWrap};

use super::{BitPack, BitWriter, BitWriterExt};

/// Adapter to **ser**ialize `T`.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For dynamic arguments, see
/// [`BitPackAsWithArgs`](super::args::as::BitPackAsWithArgs).
pub trait BitPackAs<T: ?Sized> {
    /// Packs given value using an adapter.
    fn pack_as<W>(source: &T, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter;
}

/// **Ser**ialize given value into [`BitVec`] using an adapter
#[inline]
pub fn pack_as<T, As>(value: T) -> Result<BitVec<u8, Msb0>, StringError>
where
    As: BitPackAs<T> + ?Sized,
{
    let mut writer = BitVec::new();
    writer.pack_as::<_, As>(value)?;
    Ok(writer)
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
        AsWrap::<&T, As>::new(source).pack(writer)
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
        AsWrap::<&T, As>::new(source).pack(writer)
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
        AsWrap::<&T, As>::new(source).pack(writer)
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
        AsWrap::<&T, As>::new(source).pack(writer)
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
        AsWrap::<&T, As>::new(source).pack(writer)
    }
}

impl<'a, T, As> BitPackAs<Cow<'a, T>> for Cow<'a, As>
where
    T: ToOwned + ?Sized,
    As: ToOwned + BitPackAs<T> + ?Sized,
{
    #[inline]
    fn pack_as<W>(source: &Cow<'a, T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        AsWrap::<&T, As>::new(source).pack(writer)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
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
            .map_either(AsWrap::<&Left, AsLeft>::new, AsWrap::<&Right, AsRight>::new)
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
            Some(v) => Either::Right(AsWrap::<&T, As>::new(v)),
        }
        .pack(writer)
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, As> BitPackAs<Option<T>> for Option<As>
where
    As: BitPackAs<T>,
{
    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: W) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source.as_ref().map(AsWrap::<&T, As>::new).pack(writer)
    }
}

pub trait BitPackWrapAsExt {
    #[inline]
    fn wrap_as<As>(&self) -> AsWrap<&'_ Self, As>
    where
        As: BitPackAs<Self> + ?Sized,
    {
        AsWrap::new(self)
    }
}
impl<T> BitPackWrapAsExt for T {}
