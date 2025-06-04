use crate::{
    Error,
    bits::{
        r#as::{NBits, Unary, VarNBits},
        bitvec::{order::Msb0, slice::BitSlice, vec::BitVec},
        de::{BitReader, BitReaderExt, args::r#as::BitUnpackAsWithArgs},
        ser::{BitWriter, BitWriterExt, args::r#as::BitPackAsWithArgs},
    },
};

/// `HmLabel ~n m` for [`Hashmap`](super::Hashmap)
/// ```tlb
/// hml_short$0 {m:#} {n:#} len:(Unary ~n) {n <= m} s:(n * Bit) = HmLabel ~n m;
/// hml_long$10 {m:#} n:(#<= m) s:(n * Bit) = HmLabel ~n m;
/// hml_same$11 {m:#} v:Bit n:(#<= m) = HmLabel ~n m;
/// ```
pub struct HmLabel;

impl BitPackAsWithArgs<BitSlice<u8, Msb0>> for HmLabel {
    /// m
    type Args = u32;

    fn pack_as_with<W>(
        source: &BitSlice<u8, Msb0>,
        mut writer: W,
        m: Self::Args,
    ) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let n = source.len() as u32;
        // {n <= m}
        // here we check if strictly less as (Unary ~n) needs n+1 bits
        if n < m {
            writer
                // hml_short$0
                .pack(false)?
                // len:(Unary ~n)
                .pack_as::<_, Unary>(source.len())?
                // s:(n * Bit)
                .pack(source)?;
            return Ok(());
        }

        let n_bits = m.ilog2() + 1;
        let v = if source.all() {
            true
        } else if source.not_any() {
            false
        } else {
            writer
                // hml_long$10
                .pack_as::<_, NBits<2>>(0b10)?
                // n:(#<= m)
                .pack_as_with::<_, VarNBits>(n, n_bits)?
                // s:(n * Bit)
                .pack(source)?;
            return Ok(());
        };
        writer
            // hml_same$11
            .pack_as::<_, NBits<2>>(0b11)?
            // v:Bit
            .pack(v)?
            // n:(#<= m)
            .pack_as_with::<_, VarNBits>(n, n_bits)?;
        Ok(())
    }
}

impl<'de> BitUnpackAsWithArgs<'de, BitVec<u8, Msb0>> for HmLabel {
    /// m
    type Args = u32;

    fn unpack_as_with<R>(mut reader: R, m: Self::Args) -> Result<BitVec<u8, Msb0>, R::Error>
    where
        R: BitReader<'de>,
    {
        match reader.unpack()? {
            // hml_short$0
            false => {
                // len:(Unary ~n)
                let n: u32 = reader.unpack_as::<_, Unary>()?;
                // {n <= m}
                if n > m {
                    return Err(Error::custom("n > m"));
                }
                // s:(n * Bit)
                reader.unpack_with(n as usize)
            }
            true => match reader.unpack()? {
                // hml_long$10
                false => {
                    // n:(#<= m)
                    let n: u32 = reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?;
                    // s:(n * Bit)
                    reader.unpack_with(n as usize)
                }
                // hml_same$11
                true => {
                    // v:Bit
                    let v: bool = reader.unpack()?;
                    // n:(#<= m)
                    let n: u32 = reader.unpack_as_with::<_, VarNBits>(m.ilog2() + 1)?;
                    Ok(BitVec::repeat(v, n as usize))
                }
            },
        }
    }
}
