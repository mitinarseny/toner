use std::sync::Arc;

use crate::cell_type::CellType;
use crate::{bits::{
    bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
    ser::{BitWriter, LimitWriter},
}, r#as::Ref, Cell, Error, ResultExt, OrdinaryCell, PrunedBranchCell, LibraryReferenceCell, MerkleProofCell, MerkleUpdateCell};

use super::{
    args::{r#as::CellSerializeAsWithArgs, CellSerializeWithArgs},
    r#as::CellSerializeAs,
    CellSerialize,
};

type CellBitWriter = LimitWriter<BitVec<u8, Msb0>>;

/// [`Error`] for [`CellBuilder`]
pub type CellBuilderError = <CellBuilder as BitWriter>::Error;

/// Cell builder created with [`Cell::builder()`].
///
/// [`CellBuilder`] can then be converted to constructed [`Cell`] by using
/// [`.into_cell()`](CellBuilder::into_cell).
pub struct CellBuilder {
    r#type: CellType,
    data: CellBitWriter,
    references: Vec<Arc<Cell>>,
}

const MAX_BITS_LEN: usize = 1023;
const MAX_REFS_COUNT: usize = 4;

impl CellBuilder {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            r#type: CellType::Ordinary,
            data: LimitWriter::new(BitVec::EMPTY, MAX_BITS_LEN),
            references: Vec::new(),
        }
    }

    #[inline]
    pub fn set_type(&mut self, r#type: CellType) -> &mut Self {
        self.r#type = r#type;

        self
    }

    /// Store the value using its [`CellSerialize`] implementation
    #[inline]
    pub fn store<T>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        value.store(self)?;
        Ok(self)
    }

    /// Store the value with args using its [`CellSerializeWithArgs`]
    /// implementation
    #[inline]
    pub fn store_with<T>(&mut self, value: T, args: T::Args) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerializeWithArgs,
    {
        value.store_with(self, args)?;
        Ok(self)
    }

    /// Store all values from given iterator using [`CellSerialize`]
    /// implementation of its item type.
    #[inline]
    pub fn store_many<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store(v).with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store all values from given iterator with args using
    /// [`CellSerializeWithArgs`] implementation of its item type.
    #[inline]
    pub fn store_many_with<T>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: T::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerializeWithArgs,
        T::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_with(v, args.clone())
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store given value using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_as<T, As>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        As::store_as(&value, self)?;
        Ok(self)
    }

    /// Store given value with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_as_with<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
    {
        As::store_as_with(&value, self, args)?;
        Ok(self)
    }

    /// Store all values from iterator using an adapter.  s
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_many_as<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as::<T, As>(v)
                .with_context(|| format!("[{i}]"))?;
        }
        Ok(self)
    }

    /// Store all values from iterator with args using an adapter.  
    /// See [`as`](crate::as) module-level documentation for more.
    #[inline]
    pub fn store_many_as_with<T, As>(
        &mut self,
        values: impl IntoIterator<Item = T>,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
        As::Args: Clone,
    {
        for (i, v) in values.into_iter().enumerate() {
            self.store_as_with::<T, As>(v, args.clone())
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
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        self.ensure_reference()?;
        let mut builder = Self::new();
        builder.store_as::<T, As>(value)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    #[inline]
    pub(crate) fn store_reference_as_with<T, As>(
        &mut self,
        value: T,
        args: As::Args,
    ) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAsWithArgs<T> + ?Sized,
    {
        self.ensure_reference()?;
        let mut builder = Self::new();
        builder.store_as_with::<T, As>(value, args)?;
        self.references.push(builder.into_cell().into());
        Ok(self)
    }

    /// Convert builder to [`Cell`]
    #[inline]
    #[must_use]
    pub fn into_cell(self) -> Cell {
        match self.r#type {
            CellType::Ordinary => Cell::Ordinary(OrdinaryCell { data: self.data.into_inner(), references: self.references }),
            CellType::PrunedBranch => Cell::PrunedBranch(PrunedBranchCell { data: self.data.into_inner() }),
            CellType::LibraryReference => Cell::LibraryReference(LibraryReferenceCell { data: self.data.into_inner() }),
            CellType::MerkleProof => Cell::MerkleProof(MerkleProofCell { data: self.data.into_inner(), references: self.references }),
            CellType::MerkleUpdate => Cell::MerkleUpdate(MerkleUpdateCell { data: self.data.into_inner(), references: self.references })
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
    fn store(&self, builder: &mut CellBuilder) -> Result<(), CellBuilderError> {
        builder.write_bitslice(&self.data)?;
        builder.store_many_as::<_, Ref>(&self.references)?;
        Ok(())
    }
}
