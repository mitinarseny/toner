use core::marker::PhantomData;

use crate::{
    Cell, Context, Ref, Same,
    de::{CellDeserializeAs, CellParser, CellParserError},
    ser::{CellBuilder, CellBuilderError, CellSerializeAs},
};

/// ```tlb
/// list_empty$_ {X:Type} = List X 0;
/// list$_ {X:Type} {n:#} prev:^(List X n) v:X = List X (n + 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct List<T = Same>(PhantomData<T>);

impl<T, As> CellSerializeAs<Vec<T>> for List<As>
where
    As: CellSerializeAs<T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn store_as(
        source: &Vec<T>,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        builder.store(
            source.iter().try_fold(Cell::builder(), |prev, v| {
                let mut list = Cell::builder();
                list.store_as::<_, Ref>(prev, ())?
                    .store_as::<_, &As>(v, args.clone())?;
                Ok(list)
            })?,
            (),
        )?;
        Ok(())
    }
}

impl<'de, T, As> CellDeserializeAs<'de, Vec<T>> for List<As>
where
    As: CellDeserializeAs<'de, T>,
    As::Args: Clone,
{
    type Args = As::Args;

    #[inline]
    fn parse_as(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<Vec<T>, CellParserError<'de>> {
        let mut v = Vec::new();
        let mut p: CellParser<'de> = parser.parse(())?;
        while !p.no_references_left() {
            v.push(
                p.parse_as::<_, As>(args.clone())
                    .with_context(|| format!("[{}]", v.len()))?,
            );
            p = p.parse_as::<_, Ref>(())?;
        }
        v.reverse();
        Ok(v)
    }
}
