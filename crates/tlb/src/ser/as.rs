use std::{rc::Rc, sync::Arc};

pub use crate::bits::ser::r#as::PackAsWrap;
use crate::ResultExt;

use super::{CellBuilder, CellBuilderError, CellSerialize};

pub trait CellSerializeAs<T: ?Sized> {
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError>;
}

impl<'a, T, As> CellSerialize for PackAsWrap<'a, T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::store_as(self.into_inner(), builder)
    }
}

impl<'a, T, As> CellSerializeAs<&'a T> for &'a As
where
    As: CellSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        PackAsWrap::<T, As>::new(source).store(builder)
    }
}

impl<'a, T, As> CellSerializeAs<&'a mut T> for &'a mut As
where
    As: CellSerializeAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &&mut T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        PackAsWrap::<T, As>::new(source).store(builder)
    }
}

impl<T, As> CellSerializeAs<[T]> for [As]
where
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &[T], builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        for (i, v) in source.iter().enumerate() {
            builder
                .store_as::<&T, &As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(())
    }
}

impl<T, As, const N: usize> CellSerializeAs<[T; N]> for [As; N]
where
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &[T; N], builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as::<&[T], &[As]>(source)?;
        Ok(())
    }
}

macro_rules! impl_cell_serialize_as_for_tuple {
    ($($n:tt:$t:ident as $a:ident),+) => {
        impl<$($t, $a),+> CellSerializeAs<($($t,)+)> for ($($a,)+)
        where $(
            $a: CellSerializeAs<$t>,
        )+
        {
            #[inline]
            fn store_as(source: &($($t,)+), builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
                builder$(
                    .store_as::<&$t, &$a>(&source.$n)?)+;
                Ok(())
            }
        }
    };
}
impl_cell_serialize_as_for_tuple!(0:T0 as As0);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8);
impl_cell_serialize_as_for_tuple!(0:T0 as As0,1:T1 as As1,2:T2 as As2,3:T3 as As3,4:T4 as As4,5:T5 as As5,6:T6 as As6,7:T7 as As7,8:T8 as As8,9:T9 as As9);

impl<T, As> CellSerializeAs<Box<T>> for Box<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(source: &Box<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        PackAsWrap::<T, As>::new(source).store(builder)
    }
}

impl<T, As> CellSerializeAs<Rc<T>> for Rc<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(source: &Rc<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        PackAsWrap::<T, As>::new(source).store(builder)
    }
}

impl<T, As> CellSerializeAs<Arc<T>> for Arc<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(source: &Arc<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        PackAsWrap::<T, As>::new(source).store(builder)
    }
}

impl<T, As> CellSerializeAs<Option<T>> for Option<As>
where
    As: CellSerializeAs<T>,
{
    #[inline]
    fn store_as(source: &Option<T>, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        source
            .as_ref()
            .map(CellSerializeWrapAsExt::wrap_as::<As>)
            .store(builder)
    }
}

pub trait CellSerializeWrapAsExt {
    #[inline]
    fn wrap_as<As>(&self) -> PackAsWrap<'_, Self, As>
    where
        As: CellSerializeAs<Self> + ?Sized,
    {
        PackAsWrap::new(self)
    }
}
impl<T> CellSerializeWrapAsExt for T {}
