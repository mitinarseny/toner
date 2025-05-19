pub mod r#as;

use std::{rc::Rc, sync::Arc};

use impl_tools::autoimpl;

use crate::{ResultExt, r#as::Same, bits::ser::BitWriterExt, either::Either};

use super::{CellBuilder, CellBuilderError};

/// A type that can be **ser**ialized.  
/// In contrast with [`CellSerialize`](super::CellSerialize) it allows to pass
/// [`Args`](CellSerializeWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerializeWithArgs {
    type Args;

    /// Stores the value with args
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

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
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

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
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
