use core::marker::PhantomData;

use tlb::{
    de::{r#as::CellDeserializeAs, CellParser, CellParserError},
    r#as::{Ref, Same},
    ser::{r#as::CellSerializeAs, CellBuilder, CellBuilderError},
    Cell, ResultExt,
};

/// ```tlb
/// list_empty$_ {X:Type} = List X 0;
/// list$_ {X:Type} {n:#} prev:^(List X n) v:X = List X (n + 1);
/// ```
pub struct List<T = Same>(PhantomData<T>);

impl<T, As> CellSerializeAs<T> for List<As>
where
    for<'a> &'a T: IntoIterator,
    for<'a> As: CellSerializeAs<<&'a T as IntoIterator>::Item>,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.store(source.into_iter().try_fold(Cell::builder(), |prev, v| {
            let mut list = Cell::builder();
            list.store_as::<_, Ref>(prev)?.store_as::<_, As>(v)?;
            Ok(list)
        })?)?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Vec<T>> for List<As>
where
    As: CellDeserializeAs<'de, T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Vec<T>, CellParserError<'de>> {
        let mut v = Vec::new();
        let mut p: CellParser<'de> = parser.parse()?;
        while !p.no_references_left() {
            v.push(
                p.parse_as::<_, As>()
                    .with_context(|| format!("[{}]", v.len()))?,
            );
            p = p.parse_as::<_, Ref>()?;
        }
        v.reverse();
        Ok(v)
    }
}
