use crate::{bits::DefaultOnNone, CellDeserializeAs, CellParser, CellParserError};

impl<'de, T, As> CellDeserializeAs<'de, T> for DefaultOnNone<As>
where
    T: Default,
    As: CellDeserializeAs<'de, T>,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        parser
            .parse_as::<_, Option<As>>()
            .map(Option::unwrap_or_default)
    }
}
