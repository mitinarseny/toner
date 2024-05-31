use core::marker::PhantomData;

use crate::de::{
    args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs, CellParser, CellParserError,
};

use super::Same;

pub struct ParseFully<As = Same>(PhantomData<As>);

impl<'de, T, As> CellDeserializeAs<'de, T> for ParseFully<As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        let v = parser.parse_as::<_, As>()?;
        parser.ensure_empty()?;
        Ok(v)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for ParseFully<As>
where
    As: CellDeserializeAsWithArgs<'de, T>,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        let v = parser.parse_as_with::<_, As>(args)?;
        parser.ensure_empty()?;
        Ok(v)
    }
}
