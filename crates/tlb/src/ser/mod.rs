//! **Ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
pub mod args;
pub mod r#as;
mod builder;

pub use self::builder::*;

use std::{rc::Rc, sync::Arc};

use impl_tools::autoimpl;

use crate::{
    bits::ser::BitWriterExt,
    either::Either,
    r#as::{Ref, Same},
    Cell, ResultExt,
};

/// A type that can be **ser**ilalized into [`CellBuilder`].
#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerialize {
    /// Store the value into [`CellBuilder`]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError>;
}

impl CellSerialize for () {
    #[inline]
    fn store(&self, _builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        Ok(())
    }
}

impl<T> CellSerialize for [T]
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_many(self)?;
        Ok(())
    }
}

impl<T, const N: usize> CellSerialize for [T; N]
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        self.as_slice().store(builder)
    }
}

macro_rules! impl_cell_serialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> CellSerialize for ($($t,)+)
        where $(
            $t: CellSerialize,
        )+
        {
            #[inline]
            fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError>
            {
                $(self.$n.store(builder).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_cell_serialize_for_tuple!(0:T0);
impl_cell_serialize_for_tuple!(0:T0,1:T1);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<L, R> CellSerialize for Either<L, R>
where
    L: CellSerialize,
    R: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store(l)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store(r)
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
impl<T> CellSerialize for Option<T>
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Either<(), Same>>(self.as_ref())?;
        Ok(())
    }
}

impl CellSerialize for Cell {
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder
            .pack(self.as_bitslice())?
            .store_many_as::<_, Ref>(self.references())?;

        Ok(())
    }
}

pub trait CellSerializeExt: CellSerialize {
    #[inline]
    fn to_cell(&self) -> Result<Cell, CellBuilderError> {
        let mut builder = Cell::builder();
        self.store(&mut builder)?;
        Ok(builder.into_cell())
    }
}
impl<T> CellSerializeExt for T where T: CellSerialize {}
