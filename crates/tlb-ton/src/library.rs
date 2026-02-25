use tlb::{
    Cell, Data, Ref,
    bits::NoArgs,
    de::{CellDeserialize, CellParser, CellParserError},
    either::Either,
    ser::{CellBuilder, CellBuilderError, CellSerialize},
};

/// ```tlb
/// libref_hash$0 lib_hash:bits256 = LibRef;
/// libref_ref$1 library:^Cell = LibRef;
/// ```
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LibRef<R = Cell> {
    Hash([u8; 32]),
    Ref(R),
}

impl<R> CellSerialize for LibRef<R>
where
    R: CellSerialize<Args: NoArgs>,
{
    type Args = ();

    #[inline]
    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder.store_as::<_, Either<Data, Ref>>(
            match self {
                Self::Hash(hash) => Either::Left(hash),
                Self::Ref(library) => Either::Right(library),
            },
            NoArgs::EMPTY,
        )?;
        Ok(())
    }
}

impl<'de, R> CellDeserialize<'de> for LibRef<R>
where
    R: CellDeserialize<'de, Args: NoArgs>,
{
    type Args = ();

    #[inline]
    fn parse(parser: &mut CellParser<'de>, _: Self::Args) -> Result<Self, CellParserError<'de>> {
        Ok(
            match parser.parse_as::<_, Either<Data, Ref>>(NoArgs::EMPTY)? {
                Either::Left(hash) => Self::Hash(hash),
                Either::Right(library) => Self::Ref(library),
            },
        )
    }
}
