pub mod r#as;

use std::{borrow::Cow, mem::MaybeUninit, rc::Rc, sync::Arc};

use crate::{
    Context,
    r#as::{FromInto, Same},
    bits::de::BitReaderExt,
    either::Either,
};

use super::{CellParser, CellParserError};

/// A type that can be **de**serialized.  
/// In contrast with [`CellDeserialize`](super::CellDeserialize) it allows to
/// pass [`Args`](CellDeserializeWithArgs::Args) and these arguments can be
/// calculated dynamically in runtime.
pub trait CellDeserializeWithArgs<'de>: Sized {
    type Args;

    /// Parses the value with args
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>>;
}

/// Owned version of [`CellDeserializeWithArgs`]
pub trait CellDeserializeWithArgsOwned: for<'de> CellDeserializeWithArgs<'de> {}
impl<T> CellDeserializeWithArgsOwned for T where T: for<'de> CellDeserializeWithArgs<'de> {}

impl<'de, T, const N: usize> CellDeserializeWithArgs<'de> for [T; N]
where
    T: CellDeserializeWithArgs<'de>,
    T::Args: Clone,
{
    type Args = T::Args;

    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for (i, a) in arr.iter_mut().enumerate() {
            a.write(T::parse_with(parser, args.clone()).with_context(|| format!("[{i}]"))?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_with_args_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> CellDeserializeWithArgs<'de> for ($($t,)+)
        where $(
            $t: CellDeserializeWithArgs<'de>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn parse_with(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>>
            {
                Ok(($(
                    $t::parse_with(parser, args.$n).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_with_args_for_tuple!(0:T0);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_deserialize_with_args_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'de, T> CellDeserializeWithArgs<'de> for Vec<T>
where
    T: CellDeserializeWithArgs<'de>,
    T::Args: Clone + 'de,
{
    type Args = (usize, T::Args);

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        (len, args): Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_iter_with(args).take(len).collect()
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for Box<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for Rc<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserializeWithArgs<'de> for Arc<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, FromInto<T>>(args)
    }
}

/// Always deserializes as [`Cow::Owned`]
impl<'de, 'a, T> CellDeserializeWithArgs<'de> for Cow<'a, T>
where
    T: ToOwned + ?Sized,
    T::Owned: CellDeserializeWithArgs<'de>,
{
    type Args = <T::Owned as CellDeserializeWithArgs<'de>>::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_with::<T::Owned>(args).map(Self::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right> CellDeserializeWithArgs<'de> for Either<Left, Right>
where
    Left: CellDeserializeWithArgs<'de>,
    Right: CellDeserializeWithArgs<'de, Args = Left::Args>,
{
    type Args = Left::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        match parser.unpack().context("tag")? {
            false => parser.parse_with(args).map(Either::Left).context("left"),
            true => parser.parse_with(args).map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T> CellDeserializeWithArgs<'de> for Option<T>
where
    T: CellDeserializeWithArgs<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        parser.parse_as_with::<_, Either<(), Same>>(args)
    }
}
