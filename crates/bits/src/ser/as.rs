use std::{borrow::Cow, rc::Rc, sync::Arc};

use bitvec::{order::Msb0, vec::BitVec};
use either::Either;

use crate::{
    StringError,
    r#as::{AsWrap, Same},
};

use super::{BitPack, BitWriter, BitWriterExt};

/// Adapter to **ser**ialize `T`.  
///
/// This approach is heavily inspired by
/// [serde_with](https://docs.rs/serde_with/latest/serde_with).
/// Please, read their docs for more usage examples.
pub trait BitPackAs<T: ?Sized> {
    /// Arguments to be passed in runtime
    type Args;

    /// Packs the value with args using an adapter
    fn pack_as<W>(source: &T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized;
}

/// **Ser**ialize given value into [`BitVec`] with argmuments using an adapter
#[inline]
pub fn pack_as<T, As>(value: T, args: As::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    As: BitPackAs<T> + ?Sized,
{
    let mut writer = BitVec::new();
    writer.pack_as::<_, As>(value, args)?;
    Ok(writer)
}

impl<'a, T, As> BitPackAs<&'a T> for &'a As
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &&'a T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
    }
}

impl<'a, T, As> BitPackAs<&'a mut T> for &'a mut As
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &&'a mut T, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
    }
}

impl<T, As> BitPackAs<[T]> for [As]
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &[T], writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        writer.pack_many_as::<_, &As>(source, args)?;
        Ok(())
    }
}

impl<T, As, const N: usize> BitPackAs<[T; N]> for [As; N]
where
    As: BitPackAs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &[T; N], writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        <[As]>::pack_as(source.as_slice(), writer, args)
    }
}

macro_rules! impl_bit_pack_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> BitPackAs<($($t,)+)> for ($($a,)+)
        where $(
            $a: BitPackAs<$t>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn pack_as<W>(source: &($($t,)+), writer: &mut W, args: Self::Args) -> Result<(), W::Error>
            where
                W: BitWriter + ?Sized,
            {
                writer$(
                    .pack_as::<&$t, &$a>(&source.$n, args.$n)?)+;
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

impl<T, As> BitPackAs<Rc<T>> for Box<As>
where
    As: BitPackAs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Rc<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
    }
}

impl<T, As> BitPackAs<Rc<T>> for Rc<As>
where
    As: BitPackAs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Rc<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
    }
}

impl<T, As> BitPackAs<Arc<T>> for Arc<As>
where
    As: BitPackAs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Arc<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
    }
}

impl<'a, T, As> BitPackAs<Cow<'a, T>> for Cow<'a, As>
where
    T: ToOwned + ?Sized,
    As: ToOwned + BitPackAs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Cow<'a, T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        AsWrap::<&T, As>::new(source).pack(writer, args)
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
    type Args = (AsLeft::Args, AsRight::Args);

    #[inline]
    fn pack_as<W>(
        source: &Either<Left, Right>,
        writer: &mut W,
        args: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .as_ref()
            .map_either(AsWrap::<&Left, AsLeft>::new, AsWrap::<&Right, AsRight>::new)
            .pack(writer, args)
    }
}

impl<T, As> BitPackAs<Option<T>> for Either<(), As>
where
    As: BitPackAs<T>,
{
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        Either::<Same, &As>::pack_as(
            &match source.as_ref() {
                None => Either::Left(()),
                Some(v) => Either::Right(v),
            },
            writer,
            ((), args),
        )
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
    type Args = As::Args;

    #[inline]
    fn pack_as<W>(source: &Option<T>, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        source
            .as_ref()
            .map(AsWrap::<&T, As>::new)
            .pack(writer, args)
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
