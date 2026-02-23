//! **Ser**ialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
mod r#as;
mod builder;

pub use self::{r#as::*, builder::*};

use std::{borrow::Cow, rc::Rc, sync::Arc};

use impl_tools::autoimpl;
use tlbits::ser::BitWriter;

use crate::{Cell, Context, Ref, Same, bits::ser::BitWriterExt, either::Either};

/// A type that can be **ser**ialized.  
/// In contrast with [`CellSerialize`](super::CellSerialize) it allows to pass
/// [`Args`](CellSerializeWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
#[autoimpl(for<T: trait + ToOwned + ?Sized> Cow<'_, T>)]
#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerialize {
    type Args;

    /// Stores the value with args
    fn store(&self, builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError>;
}

impl CellSerialize for () {
    type Args = ();

    #[inline]
    fn store(&self, _builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        Ok(())
    }
}

impl<T> CellSerialize for [T]
where
    T: CellSerialize,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn store(&self, builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError> {
        builder.store_many(self, args)?;
        Ok(())
    }
}

impl<T, const N: usize> CellSerialize for [T; N]
where
    T: CellSerialize,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn store(&self, builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError> {
        builder.store_many(self, args)?;
        Ok(())
    }
}

macro_rules! impl_cell_serialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> CellSerialize for ($($t,)+)
        where $(
            $t: CellSerialize,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn store(&self, builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError>
            {
                $(self.$n.store(builder, args.$n).context(concat!(".", stringify!($n)))?;)+
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
    /// `(left_args, right_args)`
    type Args = (L::Args, R::Args);

    #[inline]
    fn store(
        &self,
        builder: &mut CellBuilder,
        (la, ra): Self::Args,
    ) -> Result<(), CellBuilderError> {
        match self {
            Self::Left(l) => builder
                .pack(false, ())
                .context("tag")?
                .store(l, la)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true, ())
                .context("tag")?
                .store(r, ra)
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
    type Args = T::Args;

    #[inline]
    fn store(&self, builder: &mut CellBuilder, args: Self::Args) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}

impl CellSerialize for Cell {
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder.write_bitslice(&self.data)?;
        builder.store_many_as::<_, Ref>(&self.references, ())?;

        Ok(())
    }
}

pub trait CellSerializeExt: CellSerialize {
    #[inline]
    fn to_cell(&self, args: Self::Args) -> Result<Cell, CellBuilderError> {
        let mut builder = Cell::builder();
        self.store(&mut builder, args)?;
        Ok(builder.into_cell())
    }
}
impl<T> CellSerializeExt for T where T: CellSerialize {}
