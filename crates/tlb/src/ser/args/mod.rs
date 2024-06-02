pub mod r#as;

use std::{rc::Rc, sync::Arc};

use impl_tools::autoimpl;

use crate::{bits::ser::BitWriterExt, either::Either, r#as::Same, ResultExt};

use super::{CellBuilder, CellBuilderError};

#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerializeWithArgs {
    type Args;

    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError>;
}

impl<T> CellSerializeWithArgs for [T]
where
    T: CellSerializeWithArgs,
    T::Args: Clone,
{
    type Args = T::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_many_with(self, args)?;
        Ok(())
    }
}

impl<L, R> CellSerializeWithArgs for Either<L, R>
where
    L: CellSerializeWithArgs,
    R: CellSerializeWithArgs<Args = L::Args>,
{
    type Args = L::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        match self {
            Self::Left(l) => builder
                .pack(false)
                .context("tag")?
                .store_with(l, args)
                .context("left")?,
            Self::Right(r) => builder
                .pack(true)
                .context("tag")?
                .store_with(r, args)
                .context("right")?,
        };
        Ok(())
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<T> CellSerializeWithArgs for Option<T>
where
    T: CellSerializeWithArgs,
{
    type Args = T::Args;

    #[inline]
    fn store_with(
        &self,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store_as_with::<_, Either<(), Same>>(self.as_ref(), args)?;
        Ok(())
    }
}
