use std::{rc::Rc, sync::Arc};

use either::Either;

use crate::{
    r#as::args::NoArgs,
    ser::{r#as::PackAsWrap, BitWriter, BitWriterExt},
};

use super::BitPackWithArgs;

pub trait BitPackAsWithArgs<T: ?Sized> {
    type Args;

    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter;
}

impl<'a, T, As> BitPackWithArgs for PackAsWrap<'a, T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: BitPackAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as_with(self.into_inner(), writer, args)
    }
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
        PackAsWrap::<T, As>::new(source).pack_with(writer, args)
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
        PackAsWrap::<T, As>::new(source).pack_with(writer, args)
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
        PackAsWrap::<T, As>::new(source).pack_with(writer, args)
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
        PackAsWrap::<T, As>::new(source).pack_with(writer, args)
    }
}

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
            .map_either(
                PackAsWrap::<Left, AsLeft>::new,
                PackAsWrap::<Right, AsRight>::new,
            )
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
                None => Either::Left(PackAsWrap::<_, NoArgs<_>>::new(&())),
                Some(v) => Either::Right(PackAsWrap::<T, As>::new(v)),
            },
            writer,
            args,
        )
    }
}

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
            .map(PackAsWrap::<T, As>::new)
            .pack_with(writer, args)
    }
}
