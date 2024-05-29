use core::marker::PhantomData;

use crate::BitReader;

pub trait BitUnpackWithArgs: Sized {
    type Args;

    fn unpack_with<R>(reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader;
}

pub trait BitUnpackAsWithArgs<T> {
    type Args;
    fn unpack_as_with<R>(reader: R, args: Self::Args) -> Result<T, R::Error>
    where
        R: BitReader;
}

pub struct BitUnpackAsWithArgsWrap<T, As>
where
    As: ?Sized,
{
    value: T,
    _phantom: PhantomData<As>,
}

impl<T, As> BitUnpackAsWithArgsWrap<T, As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    /// Return the inner value of type `T`.
    #[inline]
    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T, As> BitUnpackWithArgs for BitUnpackAsWithArgsWrap<T, As>
where
    As: BitUnpackAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    fn unpack_with<R>(reader: R, args: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        As::unpack_as_with(reader, args).map(|value| Self {
            value,
            _phantom: PhantomData,
        })
    }
}
