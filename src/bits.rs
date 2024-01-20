use bitvec::{
    array::BitArray, order::BitOrder, slice::BitSlice, store::BitStore, view::BitViewSized,
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

impl<A, O> TLBSerialize for BitArray<A, O>
where
    A: BitViewSized,
    O: BitOrder,
{
    fn store(&self, builder: &mut CellBuilder) -> Result<()> {
        builder.push_bits(self)?;
        Ok(())
    }
}
