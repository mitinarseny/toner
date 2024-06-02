use core::marker::PhantomData;

use crate::{
    de::{
        args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
    },
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
    ResultExt,
};

use super::Same;

pub struct Ref<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Ref<As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store_reference_as::<&T, &As>(source).context("^")?;
        Ok(())
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for Ref<As>
where
    As: CellSerializeAsWithArgs<T> + ?Sized,
{
    type Args = As::Args;

    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder
            .store_reference_as_with::<&T, &As>(source, args)
            .context("^")?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Ref<As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        parser.parse_reference_as::<T, As>().context("^")
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for Ref<As>
where
    As: CellDeserializeAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        parser.parse_reference_as_with::<T, As>(args).context("^")
    }
}
