use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, vec::BitVec};
use either::Either;

use crate::{
    r#as::{args::NoArgs, AsWrap},
    ser::{BitWriter, BitWriterExt},
    StringError,
};

use super::BitPackWithArgs;

/// Adapter to **ser**ialize `T` with args.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For version without arguments, see [`BitPackAs`](super::super::as::BitPackAs).
pub trait BitPackAsWithArgs<T: ?Sized> {
    type Args;

    /// Packs the value with args using an adapter
    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter;
}

/// **Ser**ialize given value into [`BitVec`] with argmuments using an adapter
#[inline]
pub fn pack_as_with<T, As>(value: T, args: As::Args) -> Result<BitVec<u8, Msb0>, StringError>
where
    As: BitPackAsWithArgs<T> + ?Sized,
{
    let mut writer = BitVec::new();
    writer.pack_as_with::<_, As>(value, args)?;
    Ok(writer)
}

impl<'a, T, As> BitPackAsWithArgs<&'a T> for &'a As
where
    As: BitPackAsWithArgs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &&'a T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        AsWrap::<&T, As>::new(source).pack_with(writer, args)
    }
}

impl<'a, T, As> BitPackAsWithArgs<&'a mut T> for &'a mut As
where
    As: BitPackAsWithArgs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &&'a mut T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        AsWrap::<&T, As>::new(source).pack_with(writer, args)
    }
}

impl<T, As> BitPackAsWithArgs<[T]> for [As]
where
    As: BitPackAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &[T], mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        writer.pack_many_as_with::<_, &As>(source, args)?;
        Ok(())
    }
}

impl<T, As, const N: usize> BitPackAsWithArgs<[T; N]> for [As; N]
where
    As: BitPackAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &[T; N], writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        <[As]>::pack_as_with(source.as_slice(), writer, args)
    }
}

macro_rules! impl_bit_pack_as_with_args_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> BitPackAsWithArgs<($($t,)+)> for ($($a,)+)
        where $(
            $a: BitPackAsWithArgs<$t>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn pack_as_with<W>(source: &($($t,)+), mut writer: W, args: Self::Args) -> Result<(), W::Error>
            where
                W: BitWriter,
            {
                writer$(
                    .pack_as_with::<&$t, &$a>(&source.$n, args.$n)?)+;
                Ok(())
            }
        }
    };
}
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_bit_pack_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As> BitPackAsWithArgs<Rc<T>> for Rc<As>
where
    As: BitPackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &Rc<T>, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        AsWrap::<&T, As>::new(source).pack_with(writer, args)
    }
}

impl<T, As> BitPackAsWithArgs<Arc<T>> for Arc<As>
where
    As: BitPackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &Arc<T>, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        AsWrap::<&T, As>::new(source).pack_with(writer, args)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<Left, Right, AsLeft, AsRight> BitPackAsWithArgs<Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: BitPackAsWithArgs<Left>,
    AsRight: BitPackAsWithArgs<Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn pack_as_with<W>(
        source: &Either<Left, Right>,
        writer: W,
        args: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .as_ref()
            .map_either(AsWrap::<&Left, AsLeft>::new, AsWrap::<&Right, AsRight>::new)
            .pack_with(writer, args)
    }
}

impl<T, As> BitPackAsWithArgs<Option<T>> for Either<(), As>
where
    As: BitPackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &Option<T>, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        BitPackWithArgs::pack_with(
            &match source.as_ref() {
                None => Either::Left(AsWrap::<_, NoArgs<_>>::new(&())),
                Some(v) => Either::Right(AsWrap::<&T, As>::new(v)),
            },
            writer,
            args,
        )
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, As> BitPackAsWithArgs<Option<T>> for Option<As>
where
    As: BitPackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn pack_as_with<W>(source: &Option<T>, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        source
            .as_ref()
            .map(AsWrap::<&T, As>::new)
            .pack_with(writer, args)
    }
}
