pub mod args;
pub mod r#as;
mod parser;

pub use self::parser::*;

use core::mem::{self, MaybeUninit};
use std::{rc::Rc, sync::Arc};

use crate::{
    bits::de::BitReaderExt,
    either::Either,
    r#as::{FromInto, Same},
    Cell, ResultExt,
};

pub trait CellDeserialize<'de>: Sized {
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>>;
}

pub trait CellDeserializeOwned: for<'de> CellDeserialize<'de> {}
impl<T> CellDeserializeOwned for T where T: for<'de> CellDeserialize<'de> {}

impl<'de> CellDeserialize<'de> for () {
    #[inline]
    fn parse(_parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(())
    }
}

impl<'de, T, const N: usize> CellDeserialize<'de> for [T; N]
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        let mut arr: [MaybeUninit<T>; N] = unsafe { MaybeUninit::uninit().assume_init() };
        for a in &mut arr {
            a.write(T::parse(parser)?);
        }
        Ok(unsafe { arr.as_ptr().cast::<[T; N]>().read() })
    }
}

macro_rules! impl_cell_deserialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<'de, $($t),+> CellDeserialize<'de> for ($($t,)+)
        where $(
            $t: CellDeserialize<'de>,
        )+
        {
            #[inline]
            fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>>
            {
                Ok(($(
                    $t::parse(parser).context(concat!(".", stringify!($n)))?,
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
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

impl<'de, T> CellDeserialize<'de> for Rc<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

impl<'de, T> CellDeserialize<'de> for Arc<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, FromInto<T>>()
    }
}

impl<'de, Left, Right> CellDeserialize<'de> for Either<Left, Right>
where
    Left: CellDeserialize<'de>,
    Right: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        match parser.unpack().context("tag")? {
            false => parser.parse().map(Either::Left).context("left"),
            true => parser.parse().map(Either::Right).context("right"),
        }
    }
}

/// [Maybe](https://docs.ton.org/develop/data-formats/tl-b-types#maybe)
impl<'de, T> CellDeserialize<'de> for Option<T>
where
    T: CellDeserialize<'de>,
{
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        parser.parse_as::<_, Either<(), Same>>()
    }
}

impl<'de> CellDeserialize<'de> for Cell {
    #[inline]
    fn parse(parser: &mut CellParser<'de>) -> Result<Self, CellParserError<'de>> {
        Ok(Self {
            data: mem::take(&mut parser.data).to_bitvec(),
            references: mem::take(&mut parser.references).to_vec(),
        })
    }
}
