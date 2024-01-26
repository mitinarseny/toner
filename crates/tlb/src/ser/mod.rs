mod r#as;

pub use self::r#as::*;

use std::{rc::Rc, sync::Arc};

use bitvec::{order::Msb0, slice::BitSlice, vec::BitVec};
use impl_tools::autoimpl;

use crate::{BitWriter, Cell, Error, LimitWriter, ResultExt, MAX_BITS_LEN, MAX_REFS_COUNT};

#[autoimpl(for <T: trait + ?Sized> &T, &mut T, Box<T>, Rc<T>, Arc<T>)]
pub trait CellSerialize {
    fn store(&self, builder: &mut CellBuilder) -> Result<(), <CellBuilder as BitWriter>::Error>;
}

impl CellSerialize for () {
    #[inline]
    fn store(&self, _builder: &mut CellBuilder) -> Result<(), <CellBuilder as BitWriter>::Error> {
        Ok(())
    }
}

impl<T> CellSerialize for [T]
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), <CellBuilder as BitWriter>::Error> {
        for (i, v) in self.iter().enumerate() {
            v.store(builder).with_context(|| format!("[{i}]"))?;
        }
        Ok(())
    }
}

impl<T, const N: usize> CellSerialize for [T; N]
where
    T: CellSerialize,
{
    #[inline]
    fn store(&self, builder: &mut CellBuilder) -> Result<(), <CellBuilder as BitWriter>::Error> {
        self.as_slice().store(builder)
    }
}

macro_rules! impl_cell_serialize_for_tuple {
    ($($n:tt:$t:ident),+) => {
        impl<$($t),+> CellSerialize for ($($t,)+)
        where $(
            $t: CellSerialize,
        )+
        {
            #[inline]
            fn store(&self, builder: &mut CellBuilder) -> Result<(), <CellBuilder as BitWriter>::Error>
            {
                $(self.$n.store(builder).context(concat!(".", stringify!($n)))?;)+
                Ok(())
            }
        }
    };
}
impl_cell_serialize_for_tuple!(0:T0);
impl_cell_serialize_for_tuple!(0:T0,1:T1);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8);
impl_cell_serialize_for_tuple!(0:T0,1:T1,2:T2,3:T3,4:T4,5:T5,6:T6,7:T7,8:T8,9:T9);

type CellBitWriter = LimitWriter<BitVec<u8, Msb0>>;
pub type CellBuilderError = <CellBuilder as BitWriter>::Error;

pub struct CellBuilder {
    data: CellBitWriter,
    references: Vec<Arc<Cell>>,
}

impl CellBuilder {
    #[inline]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            data: LimitWriter::new(BitVec::EMPTY, MAX_BITS_LEN),
            references: Vec::new(),
        }
    }

    #[inline]
    pub fn store<T>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        T: CellSerialize,
    {
        value.store(self)?;
        Ok(self)
    }

    #[inline]
    pub fn store_as<T, As>(&mut self, value: T) -> Result<&mut Self, CellBuilderError>
    where
        As: CellSerializeAs<T> + ?Sized,
    {
        As::store_as(&value, self)?;
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

pub trait CellSerializeExt: CellSerialize {
    #[inline]
    fn to_cell(&self) -> Result<Cell, <CellBuilder as BitWriter>::Error> {
        let mut builder = Cell::builder();
        self.store(&mut builder)?;
        Ok(builder.into_cell())
    }
}
impl<T> CellSerializeExt for T where T: CellSerialize {}
