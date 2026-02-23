use core::marker::PhantomData;

use tlbits::{either::Either, ser::BitWriter};

use crate::{
    Cell, Context,
    de::{CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerializeAs},
};

use super::Same;

/// Adapter to **de**/**ser**ialize value from/into reference to the child cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ref<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Ref<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    type Args = As::Args;

    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_reference_as::<&T, &As>(source, args)
            .context("^")?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Ref<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        parser.parse_reference_as::<T, As>(args).context("^")
    }
}

/// ```tlb
/// {X:Type} Either X ^X = EitherInlineOrRef X
/// ```
pub struct EitherInlineOrRef<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for EitherInlineOrRef<As>
where
    As: CellSerializeAs<T>,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        let mut b = Cell::builder();
        As::store_as(source, &mut b, args)?;
        let cell = b.into_cell();
        builder.store_as::<_, Either<Same, Ref>>(
            if cell.data.len() <= builder.capacity_left() {
                Either::Left
            } else {
                Either::Right
            }(cell),
            ((), ()),
        )?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for EitherInlineOrRef<As>
where
    As: CellDeserializeAs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(parser: &mut CellParser<'de>, args: Self::Args) -> Result<T, CellParserError<'de>> {
        Either::<As, Ref<As>>::parse_as(parser, (args.clone(), args)).map(Either::into_inner)
    }
}
