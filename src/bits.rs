use bitvec::{
    array::BitArray, order::BitOrder, slice::BitSlice, store::BitStore, vec::BitVec,
    view::BitViewSized,
};

use crate::{CellBuilder, Result, TLBSerialize};

impl<S, O> TLBSerialize for BitSlice<S, O>
where
    S: BitStore,
    O: BitOrder,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.push_bits(self)?;
        Ok(())
    }
}

impl<S, O> TLBSerialize for BitVec<S, O>
where
    S: BitStore,
    O: BitOrder,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        self.as_bitslice().store(builder)
    }
}

impl<A, O> TLBSerialize for BitArray<A, O>
where
    A: BitViewSized,
    O: BitOrder,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        self.as_bitslice().store(builder)
    }
}

impl TLBSerialize for str {
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.push_bytes(self)?;
        Ok(())
    }
}
