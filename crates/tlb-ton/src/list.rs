use core::marker::PhantomData;

use tlb::{
    de::{
        r#as::{CellDeserializeAs, CellDeserializeAsOwned},
        CellParser, CellParserError,
    },
    r#as::{Ref, Same},
    ser::{r#as::CellSerializeAs, CellBuilder, CellBuilderError},
    Cell,
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
        builder.store(source.into_iter().try_fold(Cell::new(), |prev, v| {
            let mut list = Cell::builder();
            list.store_as::<_, Ref>(prev)?.store_as::<_, As>(v)?;
            Ok(list.into_cell())
        })?)?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Vec<T>> for List<As>
where
    As: CellDeserializeAsOwned<T>,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<Vec<T>, CellParserError<'de>> {
        let mut v = Vec::new();
        let mut cell: Cell = parser.parse()?;
        while !cell.references.is_empty() {
            let mut p = cell.parser();
            v.push(p.parse_as::<_, As>()?);
            cell = p.parse_as::<_, Ref>()?;
        }
        v.reverse();
        Ok(v)
    }
}
