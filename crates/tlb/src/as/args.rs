use crate::{
    de::{
        args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
    },
    ser::{
        args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs, CellBuilder, CellBuilderError,
    },
};

pub use crate::bits::r#as::args::{DefaultArgs, NoArgs};

impl<T, As, Args> CellSerializeAsWithArgs<T> for NoArgs<Args, As>
where
    As: CellSerializeAs<T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        _args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::store_as(source, builder)
    }
}

impl<'de, T, As, Args> CellDeserializeAsWithArgs<'de, T> for NoArgs<Args, As>
where
    As: CellDeserializeAs<'de, T> + ?Sized,
{
    type Args = Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        _args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::parse_as(parser)
    }
}
