use std::sync::Arc;

use tlbits::adapters::LimitWriter;

use crate::{
    Cell, Context, Error,
    r#as::Ref,
    bits::{
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        ser::BitWriter,
    },
};

use super::{CellSerialize, CellSerializeAs};

type CellBitWriter = LimitWriter<BitVec<u8, Msb0>>;

/// [`Error`] for [`CellBuilder`]
pub type CellBuilderError = <CellBuilder as BitWriter>::Error;

/// Cell builder created with [`Cell::builder()`].
///
/// [`CellBuilder`] can then be converted to constructed [`Cell`] by using
/// [`.into_cell()`](CellBuilder::into_cell).
pub struct CellBuilder {
    data: CellBitWriter,
    references: Vec<Arc<Cell>>,
}

pub(crate) const MAX_BITS_LEN: usize = 1023;
pub(crate) const MAX_REFS_COUNT: usize = 4;

impl CellBuilder {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            data: LimitWriter::new(BitVec::EMPTY, MAX_BITS_LEN),
            references: Vec::new(),
        }
    }

    /// Store the value with args using its [`CellSerialize`]
    /// implementation
    #[inline]
    pub fn store<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        value.store(self, args)?;
        Ok(self)
    }

    /// Store all values from given iterator with args using
    /// [`CellSerialize`] implementation of its item type.
    #[inline]
    pub fn store_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store given value with args using an adapter.  
    ///
    /// This approach is heavily inspired by
    /// [serde_with](https://docs.rs/serde_with/latest/serde_with).
    /// Please, read their docs for more usage examples.
    #[inline]
    pub fn store_as<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        As::store_as(&value, self, args)?;
        Ok(self)
    }

    /// Store all values from iterator with args using an adapter.  
    ///
    /// This approach is heavily inspired by
    /// [serde_with](https://docs.rs/serde_with/latest/serde_with).
    /// Please, read their docs for more usage examples.
    #[inline]
    pub fn store_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as::<T, As>(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    #[inline]
    fn ensure_reference(&self) -> Result<(), CellBuilderError> {
        if self.references.len() == MAX_REFS_COUNT {
            return Err(Error::custom("too many references"));
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn store_reference_as<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        self.ensure_reference()?;
        let mut builder = Self::new();
        builder.store_as::<T, As>(value, args)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    /// Convert builder to [`Cell`]
    #[inline]
    #[must_use]
    pub fn into_cell(self) -> Cell {
        Cell {
            data: self.data.into_inner(),
            references: self.references,
        }
    }
}

impl BitWriter for CellBuilder {
    type Error = <CellBitWriter as BitWriter>::Error;

    #[inline]
    fn capacity_left(&self) -> usize {
        self.data.capacity_left()
    }

    #[inline]
    fn write_bit(&mut self, bit: bool) -> Result<(), Self::Error> {
        self.data.write_bit(bit)?;
        Ok(())
    }

    #[inline]
    fn write_bitslice(&mut self, bits: &BitSlice<u8, Msb0>) -> Result<(), Self::Error> {
        self.data.write_bitslice(bits)
    }

    #[inline]
    fn repeat_bit(&mut self, n: usize, bit: bool) -> Result<(), Self::Error> {
        self.data.repeat_bit(n, bit)
    }
}

impl CellSerialize for CellBuilder {
    type Args = ();

    fn store(&self, builder: &mut CellBuilder, _: Self::Args) -> Result<(), CellBuilderError> {
        builder.write_bitslice(&self.data)?;
        builder.store_many_as::<_, Ref>(&self.references, ())?;
        Ok(())
    }
}
