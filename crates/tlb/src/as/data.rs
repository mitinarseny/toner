use core::{fmt::Display, marker::PhantomData};

use tlbits::{
    de::args::r#as::BitUnpackAsWithArgs,
    ser::{BitWriter, args::r#as::BitPackAsWithArgs},
};

use crate::{
    Cell, Error,
    r#as::Ref,
    bits::{
        r#as::AsBytes,
        bitvec::mem::bits_of,
        de::{BitReaderExt, r#as::BitUnpackAs},
        ser::{BitWriterExt, r#as::BitPackAs},
    },
    de::{
        CellParser, CellParserError, args::r#as::CellDeserializeAsWithArgs, r#as::CellDeserializeAs,
    },
    ser::{
        CellBuilder, CellBuilderError, args::r#as::CellSerializeAsWithArgs, r#as::CellSerializeAs,
    },
};

use super::Same;

/// Adapter to implement cell **de**/**ser**ialization from/into binary data.
///
/// ```rust
/// # use tlb::{
/// #       r#as::Data,
/// #       bits::{
/// #           de::{BitUnpack, BitReader, BitReaderExt},
/// #           ser::{BitPack, BitWriter, BitWriterExt},
/// #       },
/// #       Cell,
/// #       StringError,
/// # };
/// # #[derive(Debug, Clone, Copy, PartialEq)]
/// struct BinaryData {
///     field: u8,
/// }
///
/// impl BitPack for BinaryData {
///     fn pack<W>(&self, mut writer: W) -> Result<(), W::Error>
///         where W: BitWriter,
///     {
///         writer.pack(self.field)?;
///         Ok(())
///     }
/// }
///
/// impl<'de> BitUnpack<'de> for BinaryData {
///     fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
///         where R: BitReader<'de>,
///     {
///         Ok(Self {
///             field: reader.unpack()?,
///         })
///     }
/// }
///
/// # fn main() -> Result<(), StringError> {
/// let v = BinaryData { field: 123 };
/// # let mut builder = Cell::builder();
/// // store as binary data
/// builder.store_as::<_, Data>(v)?;
/// # let cell = builder.into_cell();
/// # let mut parser = cell.parser();
/// # let got =
/// // parse as binary data
/// parser.parse_as::<BinaryData, Data>()?;
/// # assert_eq!(got, v);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Data<As: ?Sized = Same>(PhantomData<As>);

impl<T, As> CellSerializeAs<T> for Data<As>
where
    As: BitPackAs<T> + ?Sized,
    T: ?Sized,
{
    #[inline]
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        As::pack_as(source, builder)
    }
}

impl<T, As> CellSerializeAsWithArgs<T> for Data<As>
where
    As: BitPackAsWithArgs<T> + ?Sized,
    T: ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn store_as_with(
        source: &T,
        builder: &mut CellBuilder,
        args: Self::Args,
    ) -> Result<(), CellBuilderError> {
        As::pack_as_with(source, builder, args)
    }
}

impl<'de, T, As> CellDeserializeAs<'de, T> for Data<As>
where
    As: BitUnpackAs<'de, T> + ?Sized,
{
    #[inline]
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        As::unpack_as(parser)
    }
}

impl<'de, T, As> CellDeserializeAsWithArgs<'de, T> for Data<As>
where
    As: BitUnpackAsWithArgs<'de, T> + ?Sized,
{
    type Args = As::Args;

    #[inline]
    fn parse_as_with(
        parser: &mut CellParser<'de>,
        args: Self::Args,
    ) -> Result<T, CellParserError<'de>> {
        As::unpack_as_with(parser, args)
    }
}

/// From [TEP-64](https://github.com/ton-blockchain/TEPs/blob/master/text/0064-token-data-standard.md#data-serialization):
///  ```tlb
/// tail#_ {bn:#} b:(bits bn) = SnakeData ~0;
/// cons#_ {bn:#} {n:#} b:(bits bn) next:^(SnakeData ~n) = SnakeData ~(n + 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SnakeData;

impl<T> CellSerializeAs<T> for SnakeData
where
    T: AsRef<[u8]>,
{
    fn store_as(source: &T, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        fn pack_max<'a>(
            mut s: &'a [u8],
            b: &mut CellBuilder,
        ) -> Result<&'a [u8], CellBuilderError> {
            let cur: &[u8];
            (cur, s) = s.split_at(s.len().min(b.capacity_left() / bits_of::<u8>()));
            b.pack_as::<_, AsBytes>(cur)?;
            Ok(s)
        }

        let mut s = source.as_ref();
        s = pack_max(s, builder)?;

        let mut stack: Vec<CellBuilder> = Vec::new();
        while !s.is_empty() {
            let mut b = Cell::builder();
            s = pack_max(s, &mut b)?;
            stack.push(b);
        }

        if let Some(last) = stack.pop() {
            let child = stack.into_iter().try_rfold(last, |child, mut parent| {
                parent.store_as::<_, Ref>(child)?;
                Ok(parent)
            })?;
            builder.store_as::<_, Ref>(child)?;
        }

        Ok(())
    }
}

impl<'de, T> CellDeserializeAs<'de, T> for SnakeData
where
    T: TryFrom<Vec<u8>>,
    <T as TryFrom<Vec<u8>>>::Error: Display,
{
    fn parse_as(parser: &mut CellParser<'de>) -> Result<T, CellParserError<'de>> {
        let mut parser: CellParser = parser.parse()?;

        let mut data = Vec::new();
        while !parser.no_bits_left() {
            let cur_len = data.len();
            let more = parser.bits_left() / bits_of::<u8>();
            data.resize(cur_len + more, 0);
            let n = parser.read_bytes_into(&mut data[cur_len..])?;
            if n != more * bits_of::<u8>() {
                return Err(Error::custom("EOF"));
            }
            if parser.no_references_left() {
                break;
            }
            parser = parser.parse_as::<CellParser, Ref>()?;
        }

        data.try_into().map_err(Error::custom)
    }
}

/// From [TEP-64](https://github.com/ton-blockchain/TEPs/blob/master/text/0064-token-data-standard.md#data-serialization):
///  ```tlb
/// text#_ {n:#} data:(SnakeData ~n) = Text;
/// ```
pub type Text = SnakeData;

#[cfg(test)]
mod tests {
    use crate::ser::{CellSerializeExt, r#as::CellSerializeWrapAsExt};

    use super::*;

    #[test]
    fn serde() {
        let data = "Hello, TON!"
            // make it long, so it doesn't fit into one Cell
            .repeat(100);

        let cell = data.wrap_as::<SnakeData>().to_cell().unwrap();
        let got: String = cell.parse_fully_as::<_, SnakeData>().unwrap();

        assert_eq!(got, data);
    }
}
