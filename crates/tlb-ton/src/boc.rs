use core::iter;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use base64::{engine::general_purpose::STANDARD, Engine};
use bitvec::{order::Msb0, vec::BitVec, view::AsBits};
use crc::Crc;
use tlb::{
    BitReader, BitReaderExt, BitUnpack, BitWriter, BitWriterExt, Cell, Error, NBits, ResultExt,
    StringError,
};

pub type BoC = BagOfCells;

#[derive(Debug, Clone)]
pub struct BagOfCells {
    roots: Vec<Arc<Cell>>,
}

impl BagOfCells {
    #[inline]
    pub fn from_root(root: impl Into<Arc<Cell>>) -> Self {
        Self {
            roots: [root.into()].into(),
        }
    }

    #[inline]
    pub fn add_root(&mut self, root: impl Into<Arc<Cell>>) {
        self.roots.push(root.into())
    }

    #[inline]
    pub fn single_root(&self) -> Option<&Arc<Cell>> {
        let [root]: &[_; 1] = self.roots.as_slice().try_into().ok()?;
        Some(root)
    }

    #[inline]
    pub fn pack(&self, has_crc32c: bool) -> Result<Vec<u8>, StringError> {
        self.pack_flags(false, has_crc32c)
    }

    pub fn pack_flags(&self, has_idx: bool, has_crc32c: bool) -> Result<Vec<u8>, StringError> {
        self.to_raw()?.pack_flags(has_idx, has_crc32c)
    }

    fn to_raw(&self) -> Result<RawBagOfCells, StringError> {
        let mut all_cells: HashSet<Arc<Cell>> = HashSet::new();
        let mut in_refs: HashMap<Arc<Cell>, HashSet<Arc<Cell>>> = HashMap::new();
        for r in &self.roots {
            Self::traverse_cell_tree(r, &mut all_cells, &mut in_refs)?;
        }
        let mut no_in_refs: HashSet<Arc<Cell>> = HashSet::new();
        for c in &all_cells {
            if !in_refs.contains_key(c) {
                no_in_refs.insert(c.clone());
            }
        }
        let mut ordered_cells: Vec<Arc<Cell>> = Vec::new();
        let mut indices: HashMap<Arc<Cell>, usize> = HashMap::new();
        while let Some(cell) = no_in_refs.iter().next().cloned() {
            ordered_cells.push(cell.clone());
            indices.insert(cell.clone(), indices.len());
            for child in &cell.references {
                if let Some(refs) = in_refs.get_mut(child) {
                    refs.remove(&cell);
                    if refs.is_empty() {
                        no_in_refs.insert(child.clone());
                        in_refs.remove(child);
                    }
                }
            }
            no_in_refs.remove(&cell);
        }
        if !in_refs.is_empty() {
            return Err(Error::custom("reference cycle detected"));
        }
        Ok(RawBagOfCells {
            cells: ordered_cells
                .into_iter()
                .map(|cell| RawCell {
                    data: cell.data.clone(),
                    references: cell
                        .references
                        .iter()
                        .map(|c| *indices.get(c).unwrap())
                        .collect(),
                    level: cell.level(),
                })
                .collect(),
            roots: self
                .roots
                .iter()
                .map(|c| *indices.get(c).unwrap())
                .collect(),
        })
    }

    /// Traverses all cells, fills all_cells set and inbound references map.
    fn traverse_cell_tree(
        cell: &Arc<Cell>,
        all_cells: &mut HashSet<Arc<Cell>>,
        in_refs: &mut HashMap<Arc<Cell>, HashSet<Arc<Cell>>>,
    ) -> Result<(), StringError> {
        if all_cells.insert(cell.clone()) {
            for r in &cell.references {
                if r == cell {
                    return Err(Error::custom("cell must not reference itself"));
                }
                in_refs.entry(r.clone()).or_default().insert(cell.clone());
                Self::traverse_cell_tree(r, all_cells, in_refs)?;
            }
        }
        Ok(())
    }

    pub fn parse_hex(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        let bytes = hex::decode(s).map_err(Error::custom)?;
        Self::unpack(bytes.as_bits())
    }

    pub fn parse_base64(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        let bytes = STANDARD.decode(s).map_err(Error::custom)?;
        Self::unpack(bytes.as_bits())
    }
}

impl BitUnpack for BagOfCells {
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let raw = RawBagOfCells::unpack(reader)?;
        let num_cells = raw.cells.len();
        let mut cells: Vec<Arc<Cell>> = Vec::new();
        for (i, raw_cell) in raw.cells.into_iter().enumerate() {
            cells.push(
                Cell {
                    data: raw_cell.data,
                    references: raw_cell
                        .references
                        .into_iter()
                        .map(|r| {
                            if r <= i {
                                return Err(Error::custom(format!(
                                    "references to previous cells are not supported: [{i}] -> [{r}]"
                                )));
                            }
                            Ok(cells[num_cells - 1 - r].clone())
                        })
                        .collect::<Result<_, _>>()?,
                }
                .into(),
            );
        }
        Ok(BagOfCells {
            roots: raw
                .roots
                .into_iter()
                .map(|r| cells[num_cells - 1 - r].clone())
                .collect(),
        })
    }
}

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
        let size_bits: u32 = 32 - (self.cells.len() as u32).leading_zeros();
        let size_bytes: u32 = (size_bits + 7) / 8;

        let mut tot_cells_size: u32 = 0;
        let mut index = Vec::<u32>::with_capacity(self.cells.len());
        for cell in &self.cells {
            index.push(tot_cells_size);
            tot_cells_size += cell.size(size_bytes);
        }

        let off_bits: u32 = 32 - tot_cells_size.leading_zeros();
        let off_bytes: u32 = (off_bits + 7) / 8;

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
            .pack_as::<_, NBits<3>>(size_bytes)?
            // off_bytes:(## 8) { off_bytes <= 8 }
            .pack_as::<_, NBits<8>>(off_bytes)?
            // cells:(##(size * 8))
            .pack_as_n_bytes(self.cells.len() as u32, size_bytes)?
            // roots:(##(size * 8)) { roots >= 1 }
            .pack_as_n_bytes(1u32, size_bytes)? // single root
            // absent:(##(size * 8)) { roots + absent <= cells }
            .pack_as_n_bytes(0u32, size_bytes)? // complete BoCs only
            // tot_cells_size:(##(off_bytes * 8))
            .pack_as_n_bytes(tot_cells_size, off_bytes)?
            // root_list:(roots * ##(size * 8))
            .pack_as_n_bytes(0u32, size_bytes)?; // root should have index 0
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            for id in index {
                writer.pack_as_n_bytes(id, off_bytes)?;
            }
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        for (i, cell) in self.cells.iter().enumerate() {
            cell.pack(&mut writer, size_bytes)
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
                // has_idx:(## 1) has_crc32c:(## 1)
                let (has_idx, has_crc32c) = reader.unpack()?;
                // has_cache_bits:(## 1)
                let _has_cache_bits: bool = reader.unpack()?;
                // flags:(## 2) { flags = 0 }
                let _flags: u8 = reader.unpack_as::<_, NBits<2>>()?;
                (has_idx, has_crc32c)
            }
            _ => return Err(Error::custom(format!("invalid BoC tag: {tag:#x}"))),
        };
        // size:(## 3) { size <= 4 }
        let size_bytes: u32 = reader.unpack_as::<_, NBits<3>>()?;
        if size_bytes > 4 {
            return Err(Error::custom(format!("invalid size: {size_bytes}")));
        }
        // off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes: u32 = reader.unpack_as::<_, NBits<8>>()?;
        if size_bytes > 8 {
            return Err(Error::custom(format!("invalid off_bytes: {off_bytes}")));
        }
        // cells:(##(size * 8))
        let cells: u32 = reader.unpack_as_n_bytes(size_bytes)?;
        // roots:(##(size * 8)) { roots >= 1 }
        let roots: u32 = reader.unpack_as_n_bytes(size_bytes)?;
        // absent:(##(size * 8)) { roots + absent <= cells }
        let absent: u32 = reader.unpack_as_n_bytes(size_bytes)?;
        if roots + absent > cells {
            return Err(Error::custom("roots + absent > cells"));
        }
        // tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size: usize = reader.unpack_as_n_bytes(off_bytes)?;
        let root_list = if tag == Self::GENERIC_BOC_TAG {
            // root_list:(roots * ##(size * 8))
            iter::repeat_with(|| reader.unpack_as_n_bytes(size_bytes))
                .take(roots as usize)
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            let _index: Vec<usize> = iter::repeat_with(|| reader.unpack_as_n_bytes(off_bytes))
                .take(cells as usize)
                .collect::<Result<_, _>>()?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        let cell_data: Vec<RawCell> =
            iter::repeat_with(|| RawCell::unpack(&mut reader, size_bytes))
                .take(cells as usize)
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
            let cs = u32::from_le_bytes(reader.unpack()?);
            if cs != CRC_32_ISCSI.checksum(buff.as_raw_slice()) {
                return Err(Error::custom("CRC mismatch"));
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
    fn unpack<R>(mut reader: R, size_bytes: u32) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let refs_descriptor: u8 = reader.unpack()?;
        let level: u8 = refs_descriptor >> 5;
        let _is_exotic: bool = refs_descriptor >> 3 & 0b1 == 1;
        let ref_num: usize = refs_descriptor as usize & 0b111;

        let bits_descriptor: u8 = reader.unpack()?;
        let num_bytes: usize = ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize;
        let full_bytes = (bits_descriptor & 1) == 0;

        let mut data = reader.read_bitvec(num_bytes * 8)?;
        if !data.is_empty() && !full_bytes {
            let trailing_zeros = data.trailing_zeros();
            if trailing_zeros >= 8 {
                return Err(Error::custom("last byte must be non zero"));
            }
            data.truncate(data.len() - trailing_zeros - 1);
        }

        let references: Vec<usize> = iter::repeat_with(|| reader.unpack_as_n_bytes(size_bytes))
            .take(ref_num)
            .collect::<Result<_, _>>()?;

        Ok(RawCell {
            data,
            references,
            level,
        })
    }

    fn pack<W>(&self, mut writer: W, ref_size_bytes: u32) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let level: u8 = 0;
        let is_exotic: u8 = 0;
        let refs_descriptor: u8 = self.references.len() as u8 + is_exotic * 8 + level * 32;
        writer.pack(refs_descriptor)?;

        let padding_bits = self.data.len() % 8;
        let full_bytes = padding_bits == 0;
        let data_bytes = (self.data.len() + 7) / 8;
        let bits_descriptor: u8 = data_bytes as u8 * 2 - if full_bytes { 0 } else { 1 }; // subtract 1 if the last byte is not full
        writer.pack(bits_descriptor)?;

        writer.pack(self.data.as_bitslice())?;
        if !full_bytes {
            writer.write_bit(true)?;
            writer.repeat_bit(padding_bits - 1, false)?;
        }

        for r in &self.references {
            writer.pack_as_n_bytes(*r, ref_size_bytes)?;
        }

        Ok(())
    }

    fn size(&self, ref_size_bytes: u32) -> u32 {
        let data_len: u32 = (self.data.len() as u32 + 7) / 8;
        2 + data_len + self.references.len() as u32 * ref_size_bytes
    }
}

#[cfg(test)]
mod tests {
    use tlb::CellSerializeExt;

    use super::*;

    #[test]
    fn boc_serde() {
        let packed = BoC::from_root(().to_cell().unwrap()).pack(true).unwrap();
        packed
            .as_bits()
            .unpack::<BoC>()
            .unwrap()
            .single_root()
            .unwrap()
            .parse_fully::<()>()
            .unwrap();
    }
}
