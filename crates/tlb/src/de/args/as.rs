use std::{borrow::Cow, rc::Rc, sync::Arc};

use crate::{
    Context,
    r#as::{AsWrap, NoArgs},
    either::Either,
};

use super::{
    super::{CellParser, CellParserError},
    CellDeserializeWithArgs,
};

/// Adaper to **de**serialize `T` with args.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For version without arguments, see
/// [`CellDeserializeAs`](super::super::as::CellDeserializeAs).
pub trait CellDeserializeAsWithArgs<'de, T> {
    type Args;

    /// Parse value with args using an adapter
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>>;
}

/// Owned version of [`CellDeserializeAsWithArgs`]
pub trait CellDeserializeAsWithArgsOwned<T>: for<'de> CellDeserializeAsWithArgs<'de, T> {}
impl<T, As> CellDeserializeAsWithArgsOwned<As> for T where
    T: for<'de> CellDeserializeAsWithArgs<'de, As> + ?Sized
{
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
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| {
            As::parse_as_with(parser, args.clone()).with_context(|| format!("[{i}]"))
        })
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, Vec<T>> for Vec<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
    As::Args: Clone + 'de,
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
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
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
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
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
        AsWrap::<T, As>::parse_with(parser, args)
            .map(AsWrap::into_inner)
            .map(Into::into)
    }
}

/// Always deserializes as [`Cow::Owned`]
impl<'de, 'a, T, As> CellDeserializeAsWithArgs<'de, Cow<'a, T>> for Cow<'a, As>
where
    T: ToOwned + ?Sized,
    As: ToOwned + ?Sized,
    As::Owned: CellDeserializeAsWithArgs<'de, T::Owned>,
{
    type Args = <As::Owned as CellDeserializeAsWithArgs<'de, T::Owned>>::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Cow<'a, T>, CellParserError<'de>> {
        AsWrap::<T::Owned, As::Owned>::parse_with(parser, args)
            .map(AsWrap::into_inner)
            .map(Cow::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
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
            Either::<AsWrap<Left, AsLeft>, AsWrap<Right, AsRight>>::parse_with(parser, args)?
                .map_either(AsWrap::into_inner, AsWrap::into_inner),
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

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
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
        Ok(Option::<AsWrap<T, As>>::parse_with(parser, args)?.map(AsWrap::into_inner))
    }
}
