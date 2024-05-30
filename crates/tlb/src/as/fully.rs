use core::marker::PhantomData;

use crate::de::{r#as::CellDeserializeAs, CellParser, CellParserError};

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
