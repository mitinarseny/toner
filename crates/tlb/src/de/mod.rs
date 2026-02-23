//! **De**serialization for [TL-B](https://docs.ton.org/develop/data-formats/tl-b-language)
mod r#as;
mod parser;

pub use self::{r#as::*, parser::*};

use std::{borrow::Cow, mem, rc::Rc, sync::Arc};

use crate::{Cell, Context, FromInto, Same, bits::de::BitReaderExt, either::Either};

/// A type that can be **de**serialized.  
pub trait CellDeserialize<'de>: Sized {
    /// Arguments to be passed in runtime
    type Args;

    /// Parses the value with args
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>>;
}

/// Owned version of [`CellDeserialize`]
pub trait CellDeserializeOwned: for<'de> CellDeserialize<'de> {}
impl<T> CellDeserializeOwned for T where T: for<'de> CellDeserialize<'de> {}

impl<'de> CellDeserialize<'de> for () {
    type Args = ();

    #[inline]
    fn parse(_parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(())
    }
}

impl<'de, T, const N: usize> CellDeserialize<'de> for [T; N]
where
    T: CellDeserialize<'de>,
    T::Args: Clone,
{
    type Args = T::Args;

    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        // TODO: replace with [`core::array::try_from_fn`](https://github.com/rust-lang/rust/issues/89379) when stabilized
        array_util::try_from_fn(|i| {
            T::parse(parser, args.clone()).with_context(|| format!("[{i}]"))
        })
    }
}

macro_rules! impl_cell_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> CellDeserialize<'de> for ($($t,)+)
        where $(
            $t: CellDeserialize<'de>,
        )+
        {
            type Args = ($($t::Args,)+);

            #[inline]
            fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>>
            {
                Ok(($(
                    $t::parse(parser, args.$n).context(concat!(".", stringify!($n)))?,
                )+))
            }
        }
    };
}
impl_cell_deserialize_for_tuple!(0:T0);
impl_cell_deserialize_for_tuple!(0:T0,1:T1);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_deserialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

impl<'de, T> CellDeserialize<'de> for Box<T>
where
    T: CellDeserialize<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserialize<'de> for Rc<T>
where
    T: CellDeserialize<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>(args)
    }
}

impl<'de, T> CellDeserialize<'de> for Arc<T>
where
    T: CellDeserialize<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>(args)
    }
}

/// Always deserializes as [`Cow::Owned`]
impl<'de, 'a, T> CellDeserialize<'de> for Cow<'a, T>
where
    T: ToOwned + ?Sized,
    T::Owned: CellDeserialize<'de>,
{
    type Args = <T::Owned as CellDeserialize<'de>>::Args;

    #[inline]
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        parser.parse::<T::Owned>(args).map(Self::Owned)
    }
}

/// Implementation of [`Either X Y`](https://docs.ton.org/develop/data-formats/tl-b-types#either):
/// ```tlb
/// left$0 {X:Type} {Y:Type} value:X = Either X Y;
/// right$1 {X:Type} {Y:Type} value:Y = Either X Y;
/// ```
impl<'de, Left, Right> CellDeserialize<'de> for Either<Left, Right>
where
    Left: CellDeserialize<'de>,
    Right: CellDeserialize<'de>,
{
    /// `(left_args, right_args)`
    type Args = (Left::Args, Right::Args);

    #[inline]
    fn parse(
        parser: &mut CellParser<'de>,
        (la, ra): Self::Args,
    ) -> Result<Self, CellParserError<'de>> {
        match parser.unpack(()).context("tag")? {
            false => parser.parse(la).map(Either::Left).context("left"),
            true => parser.parse(ra).map(Either::Right).context("right"),
        }
    }
}

/// Implementation of [`Maybe X`](https://docs.ton.org/develop/data-formats/tl-b-types#maybe):
/// ```tlb
/// nothing$0 {X:Type} = Maybe X;
/// just$1 {X:Type} value:X = Maybe X;
/// ```
impl<'de, T> CellDeserialize<'de> for Option<T>
where
    T: CellDeserialize<'de>,
{
    type Args = T::Args;

    #[inline]
    fn parse(parser: &mut CellParser<'de>, args: Self::Args) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, Either<(), Same>>(args)
    }
}

impl<'de> CellDeserialize<'de> for Cell {
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            data: mem::take(&mut parser.data).to_bitvec(),
            references: mem::take(&mut parser.references).to_vec(),
        })
    }
}
