use core::iter;

use bitvec::{order::Msb0, vec::BitVec, view::AsBits};
use crc::Crc;
use tlb::{
    BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Error, NBits, ResultExt,
    StringError,
};

const CRC_32_ISCSI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub(crate) struct RawBagOfCells {
    pub cells: Vec<RawCell>,
    pub roots: Vec<usize>,
}

impl RawBagOfCells {
    // serialized_boc_idx#68ff65f3
    const INDEXED_BOC_TAG: u32 = 0x68ff65f3;
    // serialized_boc_idx_crc32c#acc3a728
    const INDEXED_CRC32_TAG: u32 = 0xacc3a728;
    // serialized_boc#b5ee9c72
    const GENERIC_BOC_TAG: u32 = 0xb5ee9c72;

    pub fn pack_flags(&self, has_idx: bool, has_crc32c: bool) -> Result<Vec<u8>, StringError> {
        if self.roots.len() > 1 {
            return Err(Error::custom("only single root cell supported"));
        }
        let size_bits: usize = 32 - self.cells.len().leading_zeros() as usize;
        let size: usize = (size_bits + 7) / 8;

        let mut tot_cells_size: usize = 0;
        let mut index = Vec::<usize>::with_capacity(self.cells.len());
        for cell in &self.cells {
            index.push(tot_cells_size);
            tot_cells_size += cell.size(size);
        }

        let off_bits: usize = 32 - tot_cells_size.leading_zeros() as usize;
        let off_bytes: usize = (off_bits + 7) / 8;

        let mut writer: BitVec<u8, Msb0> = BitVec::new();
        writer
            // serialized_boc#b5ee9c72
            .pack(Self::GENERIC_BOC_TAG)?
            // has_idx:(## 1)
            .pack(has_idx)?
            // has_crc32c:(## 1)
            .pack(has_crc32c)?
            // has_cache_bits:(## 1)
            .pack(false)?
            // flags:(## 2) { flags = 0 }
            .pack_as::<u8, NBits<2>>(0)?
            // size:(## 3) { size <= 4 }
            .pack_as::<_, NBits<3>>(size)?
            // off_bytes:(## 8) { off_bytes <= 8 }
            .pack_as::<_, NBits<8>>(off_bytes)?
            // cells:(##(size * 8))
            .pack_usize_as_bytes(size, self.cells.len())?
            // roots:(##(size * 8)) { roots >= 1 }
            .pack_usize_as_bytes(size, 1)? // single root
            // absent:(##(size * 8)) { roots + absent <= cells }
            .pack_usize_as_bytes(size, 0)? // complete BoCs only
            // tot_cells_size:(##(off_bytes * 8))
            .pack_usize_as_bytes(off_bytes, tot_cells_size)?
            // root_list:(roots * ##(size * 8))
            .pack_usize_as_bytes(size, 0)?; // root should have index 0
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            for id in index {
                writer.pack_usize_as_bytes(id, off_bytes)?;
            }
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        for (i, cell) in self.cells.iter().enumerate() {
            cell.pack(&mut writer, size)
                .with_context(|| format!("[{i}]"))?;
        }

        if writer.len() % 8 != 0 {
            return Err(Error::custom("produced stream is not byte-aligned"));
        }
        // crc32c:has_crc32c?uint32
        if has_crc32c {
            let cs = CRC_32_ISCSI.checksum(writer.as_raw_slice());
            writer.write_bitslice(cs.to_le_bytes().as_bits())?;
        }
        Ok(writer.into_vec())
    }
}

impl BitUnpack for RawBagOfCells {
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut buff = BitVec::<u8, Msb0>::new();
        let mut reader = reader.tee(&mut buff);

        let tag = reader.unpack::<u32>()?;
        let (has_idx, has_crc32c) = match tag {
            Self::INDEXED_BOC_TAG => (true, false),
            Self::INDEXED_CRC32_TAG => (true, true),
            Self::GENERIC_BOC_TAG => {
                // has_cache_bits:(## 1)
                let _has_cache_bits: bool = reader.unpack()?;
                // flags:(## 2) { flags = 0 }
                let _flags: u8 = reader.unpack_as::<_, NBits<2>>()?;
                // has_idx:(## 1) has_crc32c:(## 1)
                (reader.unpack()?, reader.unpack()?)
            }
            _ => return Err(Error::custom(format!("invalid BoC tag: {tag:#x}"))),
        };
        // size:(## 3) { size <= 4 }
        let size: usize = reader.unpack_as::<_, NBits<3>>()?;
        if size > 4 {
            return Err(Error::custom(format!("invalid size: {size}")));
        }
        // off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes: usize = reader.unpack_as::<_, NBits<8>>()?;
        if size > 8 {
            return Err(Error::custom(format!("invalid off_bytes: {off_bytes}")));
        }
        // cells:(##(size * 8))
        let cells: usize = reader.unpack_usize_as_bytes(size)?;
        // roots:(##(size * 8)) { roots >= 1 }
        let roots: usize = reader.unpack_usize_as_bytes(size)?;
        // absent:(##(size * 8)) { roots + absent <= cells }
        let absent: usize = reader.unpack_usize_as_bytes(size)?;
        if roots + absent > cells {
            return Err(Error::custom("roots + absent > cells"));
        }
        // tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size: usize = reader.unpack_usize_as_bytes(off_bytes)?;
        let root_list = if tag == Self::GENERIC_BOC_TAG {
            // root_list:(roots * ##(size * 8))
            iter::repeat_with(|| reader.unpack_usize_as_bytes(size))
                .take(roots)
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            let _index: Vec<usize> = iter::repeat_with(|| reader.unpack_usize_as_bytes(off_bytes))
                .take(cells)
                .into_iter()
                .collect::<Result<_, _>>()?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        let cell_data: Vec<RawCell> = iter::repeat_with(|| RawCell::unpack(&mut reader, size))
            .take(cells)
            .enumerate()
            .map(|(i, v)| v.with_context(|| format!("[{i}]")))
            .collect::<Result<_, _>>()
            .context("cell_data")?;

        let mut reader = reader.into_inner();
        if buff.len() % 8 != 0 {
            return Err(Error::custom("produced stream is not byte-aligned"));
        }
        if has_crc32c {
            // crc32c:has_crc32c?uint32
            let cs: u32 = reader.unpack()?;
            if cs != CRC_32_ISCSI.checksum(buff.as_raw_slice()) {
                return Err(Error::custom("checksum failed"));
            }
        }

        Ok(RawBagOfCells {
            cells: cell_data,
            roots: root_list,
        })
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub(crate) struct RawCell {
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<usize>,
    pub level: u8,
}

impl RawCell {
    fn unpack<R>(mut reader: R, size: usize) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let refs_descriptor: u8 = reader.unpack()?;
        let level: u8 = refs_descriptor >> 5;
        let _is_exotic: bool = refs_descriptor & 8 == 0;
        let ref_num: usize = refs_descriptor as usize & 0b111;

        let bits_descriptor: u8 = reader.unpack()?;
        let num_bytes: usize = ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize;
        let full_bytes = (bits_descriptor & 1) == 0;

        let mut data = reader.read_bitvec(num_bytes * 8)?;
        if data.len() > 0 && !full_bytes {
            let trailing_zeros = data.trailing_zeros();
            if trailing_zeros >= 8 {
                return Err(Error::custom("last byte must be non zero"));
            }
            data.truncate(data.len() - trailing_zeros - 1);
        }

        let references: Vec<usize> = iter::repeat_with(|| reader.unpack_usize_as_bytes(size))
            .take(ref_num)
            .collect::<Result<_, _>>()?;

        Ok(RawCell {
            data,
            references,
            level,
        })
    }

    fn pack<W>(&self, mut writer: W, ref_size_bytes: usize) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let level = 0u8;
        let is_exotic = 0u8;
        writer.pack(self.references.len() as u8 + is_exotic + level * 32)?;

        let padding_bits = self.data.len() % 8;
        let full_bytes = padding_bits == 0;
        writer.pack(self.data.as_bitslice())?;
        if !full_bytes {
            writer.write_bit(true)?;
            writer.repeat_bit(padding_bits - 1, false)?;
        }

        for r in &self.references {
            writer.pack_usize_as_bytes(*r, ref_size_bytes)?;
        }

        Ok(())
    }

    fn size(&self, ref_size_bytes: usize) -> usize {
        let data_len = (self.data.len() + 7) / 8;
        2 + data_len as usize + self.references.len() as usize * ref_size_bytes
    }
}
