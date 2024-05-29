use std::marker::PhantomData;

use crate::BitWriter;

pub trait BitPackWithArgs {
    type Args;

    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter;
}

pub trait BitPackAsWithArgs<T: ?Sized> {
    type Args;

    fn pack_as_with<W>(source: &T, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter;
}

pub struct BitPackAsWithArgsWrap<'a, T, As>
where
    As: BitPackAsWithArgs<T> + ?Sized,
    T: ?Sized,
{
    value: &'a T,
    _phantom: PhantomData<As>,
}

impl<'a, T, As> BitPackAsWithArgsWrap<'a, T, As>
where
    T: ?Sized,
    As: BitPackAsWithArgs<T> + ?Sized,
{
    #[inline]
    pub const fn new(value: &'a T) -> Self {
        Self {
            value,
            _phantom: PhantomData,
        }
    }
}

impl<'a, T, As> BitPackWithArgs for BitPackAsWithArgsWrap<'a, T, As>
where
    T: ?Sized,
    As: ?Sized,
    As: BitPackAsWithArgs<T>,
{
    type Args = As::Args;

    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        As::pack_as_with(self.value, writer, args)
    }
}
