use std::{rc::Rc, sync::Arc};

use crate::{
    either::Either,
    r#as::{AsWrap, NoArgs},
};

use super::{
    super::{CellBuilder, CellBuilderError},
    CellSerializeWithArgs,
};

/// Adapter to **ser**ialize `T` with args.  
/// See [`as`](crate::as) module-level documentation for more.
///
/// For version without arguments, see [`CellSerializeAs`](super::super::as::CellSerializeAs).
pub trait CellSerializeAsWithArgs<T: ?Sized> {
    type Args;

    /// Stores the value with args using an adapter
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError>;
}

impl<'a, T, As> CellSerializeAsWithArgs<&'a T> for &'a As
where
    T: ?Sized,
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &&'a T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        AsWrap::<&T, As>::new(source).store_with(builder, args)
    }
}

impl<'a, T, As> CellSerializeAsWithArgs<&'a mut T> for &'a mut As
where
    T: ?Sized,
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &&'a mut T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        AsWrap::<&T, As>::new(source).store_with(builder, args)
    }
}

impl<T, As> CellSerializeAsWithArgs<[T]> for [As]
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &[T],
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_many_as_with::<_, &As>(source, args)?;
        Ok(())
    }
}

impl<T, As, const N: usize> CellSerializeAsWithArgs<[T; N]> for [As; N]
where
    As: CellSerializeAsWithArgs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &[T; N],
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        <[As]>::store_as_with(source, builder, args)
    }
}

macro_rules! impl_cell_serialize_as_with_args_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> CellSerializeAsWithArgs<($($t,)+)> for ($($a,)+)
        where $(
            $a: CellSerializeAsWithArgs<$t>,
        )+
        {
            type Args = ($($a::Args,)+);

            #[inline]
            fn store_as_with(source: &($($t,)+), builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError> {
                builder$(
                    .store_as_with::<&$t, &$a>(&source.$n, args.$n)?)+;
                Ok(())
            }
        }
    };
}
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_serialize_as_with_args_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As> CellSerializeAsWithArgs<Box<T>> for Box<As>
where
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &Box<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        AsWrap::<&T, As>::new(source).store_with(builder, args)
    }
}

impl<T, As> CellSerializeAsWithArgs<Rc<T>> for Rc<As>
where
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &Rc<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        AsWrap::<&T, As>::new(source).store_with(builder, args)
    }
}

impl<T, As> CellSerializeAsWithArgs<Arc<T>> for Arc<As>
where
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &Arc<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        AsWrap::<&T, As>::new(source).store_with(builder, args)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<Left, Right, AsLeft, AsRight> CellSerializeAsWithArgs<Either<Left, Right>>
    for Either<AsLeft, AsRight>
where
    AsLeft: CellSerializeAsWithArgs<Left>,
    AsRight: CellSerializeAsWithArgs<Right, Args = AsLeft::Args>,
{
    type Args = AsLeft::Args;

    #[inline]
    fn store_as_with(
        source: &Either<Left, Right>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .as_ref()
            .map_either(AsWrap::<&Left, AsLeft>::new, AsWrap::<&Right, AsRight>::new)
            .store_with(builder, args)
    }
}

impl<T, As> CellSerializeAsWithArgs<Option<T>> for Either<(), As>
where
    As: CellSerializeAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &Option<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        match source.as_ref() {
            None => Either::Left(AsWrap::<_, NoArgs<_>>::new(&())),
            Some(v) => Either::Right(AsWrap::<&T, As>::new(v)),
        }
        .store_with(builder, args)
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<T, As> CellSerializeAsWithArgs<Option<T>> for Option<As>
where
    As: CellSerializeAsWithArgs<T>,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &Option<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        source
            .as_ref()
            .map(AsWrap::<_, As>::new)
            .store_with(builder, args)
    }
}
