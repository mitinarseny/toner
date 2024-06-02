use core::mem::MaybeUninit;
use std::{rc::Rc, sync::Arc};

use crate::{bits::de::r#as::UnpackAsWrap, either::Either, r#as::NoArgs, ResultExt};

use super::{
    super::{CellParser, CellParserError},
    CellDeserializeWithArgs,
};

pub trait CellDeserializeAsWithArgs<'de, T> {
    type Args;

    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>>;
}

impl<'de, T, As> CellDeserializeWithArgs<'de> for UnpackAsWrap<T, As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        As::parse_as_with(parser, args).map(Self::new)
    }
}

impl<'de, T, As, const N: usize> CellDeserializeAsWithArgs<'de, [T; N]> for [As; N]
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<[T; N], CellParserError<'de>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(parser.parse_as_with::<T, As>(args.clone())?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

impl<'de, 'a: 'de, T, As> CellDeserializeAsWithArgs<'de, Vec<T>> for Vec<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone + 'a,
{
    type Args = (usize, As::Args);

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        (len, args): Self::Args,
    ) -> Result<Vec<T>, CellParserError<'de>> {
        parser.parse_iter_as_with::<_, As>(args).take(len).collect()
    }
}

macro_rules! impl_cell_deserialize_as_with_args_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<'de, $($t, $a),+> CellDeserializeAsWithArgs<'de, ($($t,)+)> for ($($a,)+)
        where $(
            $a: CellDeserializeAsWithArgs<'de, $t>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn parse_as_with(parser: &mut CellParser<'de>, args: Self::Args) -> Result<($($t,)+), CellParserError<'de>>
            {
                Ok(($(
                    $a::parse_as_with(parser, args.$n)
                        .context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_deserialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Box<T>> for Box<As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Box<T>, CellParserError<'de>> {
        UnpackAsWrap::<T, As>::parse_with(parser, args)
            .map(UnpackAsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Rc<T>> for Rc<As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Rc<T>, CellParserError<'de>> {
        UnpackAsWrap::<T, As>::parse_with(parser, args)
            .map(UnpackAsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Arc<T>> for Arc<As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Arc<T>, CellParserError<'de>> {
        UnpackAsWrap::<T, As>::parse_with(parser, args)
            .map(UnpackAsWrap::into_inner)
            .map(Into::into)
    }
}

impl<'de, Left, Right, AsLeft, AsRight> CellDeserializeAsWithArgs<'de, Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellDeserializeAsWithArgs<'de, Left>,
    AsRight: CellDeserializeAsWithArgs<'de, Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Either<Left, Right>, CellParserError<'de>> {
        Ok(
            Either::<UnpackAsWrap<Left, AsLeft>, UnpackAsWrap<Right, AsRight>>::parse_with(
                parser, args,
            )?
            .map_either(UnpackAsWrap::into_inner, UnpackAsWrap::into_inner),
        )
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Option<T>> for Either<(), As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Option<T>, CellParserError<'de>> {
        Ok(parser
            .parse_as_with::<Either<(), T>, Either<NoArgs<_>, As>>(args)?
            .right())
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Option<T>> for Option<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Option<T>, CellParserError<'de>> {
        Ok(Option::<UnpackAsWrap<T, As>>::parse_with(parser, args)?.map(UnpackAsWrap::into_inner))
    }
}
