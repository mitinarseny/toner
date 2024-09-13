//! Collection of types related to [Bag Of Cells](https://docs.ton.org/develop/data-formats/cell-boc#bag-of-cells)
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    sync::Arc,
};

use crate::cell_type::RawCellType;
use base64::{engine::general_purpose::STANDARD, Engine};
use crc::Crc;
use tlb::{
    bits::{
        bitvec::{order::Msb0, vec::BitVec, view::AsBits},
        de::{args::BitUnpackWithArgs, BitReader, BitReaderExt, BitUnpack},
        r#as::{NBits, VarNBytes},
        ser::{args::BitPackWithArgs, BitWriter, BitWriterExt},
    },
    Cell, Error, LibraryReferenceCell, MerkleProofCell, OrdinaryCell, PrunedBranchCell, ResultExt,
    StringError,
};

/// Alias to [`BagOfCells`]
pub type BoC = BagOfCells;

/// [Bag Of Cells](https://docs.ton.org/develop/data-formats/cell-boc#bag-of-cells) is used to **de**/**ser**ialize a set of cells from/into
/// bytes.
///
/// ```rust
/// # use tlb::{
/// #     r#as::Data,
/// #     bits::{de::unpack_fully, ser::{BitWriterExt, pack_with}},
/// #     Cell,
/// #     ser::CellSerializeExt,
/// #     StringError,
/// # };
/// # use tlb_ton::{boc::{BagOfCells, BagOfCellsArgs}, MsgAddress};
/// # fn main() -> Result<(), StringError> {
/// let addr = MsgAddress::NULL;
/// let mut builder = Cell::builder();
/// builder.pack(addr)?;
/// let root = builder.into_cell();
///
/// let boc = BagOfCells::from_root(root);
/// let packed = pack_with(boc, BagOfCellsArgs {
///     has_idx: false,
///     has_crc32c: true,
/// })?;
///
/// let unpacked: BagOfCells = unpack_fully(packed)?;
/// let got: MsgAddress = unpacked
///     .single_root()
///     .unwrap()
///     .parse_fully_as::<_, Data>()?;
///
/// assert_eq!(got, addr);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct BagOfCells {
    roots: Vec<Arc<Cell>>,
}

impl BagOfCells {
    /// Create from single root cell
    #[inline]
    pub fn from_root(root: impl Into<Arc<Cell>>) -> Self {
        Self {
            roots: [root.into()].into(),
        }
    }

    /// Add root
    #[inline]
    pub fn add_root(&mut self, root: impl Into<Arc<Cell>>) {
        self.roots.push(root.into())
    }

    /// Return single root or `None` otherwise
    #[inline]
    pub fn single_root(&self) -> Option<&Arc<Cell>> {
        let [root]: &[_; 1] = self.roots.as_slice().try_into().ok()?;
        Some(root)
    }

    /// Traverses all cells, fills all_cells set and inbound references map.
    fn traverse_cell_tree(
        cell: &Arc<Cell>,
        all_cells: &mut HashSet<Arc<Cell>>,
        in_refs: &mut HashMap<Arc<Cell>, HashSet<Arc<Cell>>>,
    ) -> Result<(), StringError> {
        if all_cells.insert(cell.clone()) {
            for r in cell.references() {
                if r == cell {
                    return Err(Error::custom("cell must not reference itself"));
                }
                in_refs.entry(r.clone()).or_default().insert(cell.clone());
                Self::traverse_cell_tree(r, all_cells, in_refs)?;
            }
        }
        Ok(())
    }

    /// Parse hexadecimal string
    pub fn parse_hex(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        let bytes = hex::decode(s).map_err(Error::custom)?;
        Self::unpack(bytes.as_bits())
    }

    /// Parse base64-encoded string
    pub fn parse_base64(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        let bytes = STANDARD.decode(s).map_err(Error::custom)?;
        Self::unpack(bytes.as_bits())
    }
}

impl Debug for BagOfCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(&self.roots).finish()
    }
}

/// [`BitPackWithArgs::Args`] for [`BagOfCells`]
#[derive(Debug, Clone, Copy, Default)]
pub struct BagOfCellsArgs {
    pub has_idx: bool,
    pub has_crc32c: bool,
}

/// ```tlb
/// serialized_boc_idx#68ff65f3 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   = BagOfCells;///

/// serialized_boc_idx_crc32c#acc3a728 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:uint32 = BagOfCells;///

/// serialized_boc#b5ee9c72 has_idx:(## 1) has_crc32c:(## 1)
///   has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
///   size:(## 3) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots >= 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   root_list:(roots * ##(size * 8))
///   index:has_idx?(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:has_crc32c?uint32
///   = BagOfCells;
/// ```
impl BitPackWithArgs for BagOfCells {
    type Args = BagOfCellsArgs;

    fn pack_with<W>(&self, writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
        let mut all_cells: HashSet<Arc<Cell>> = HashSet::new();
        let mut in_refs: HashMap<Arc<Cell>, HashSet<Arc<Cell>>> = HashMap::new();
        for r in &self.roots {
            Self::traverse_cell_tree(r, &mut all_cells, &mut in_refs).map_err(Error::custom)?;
        }
        let mut no_in_refs: HashSet<Arc<Cell>> = HashSet::new();
        for c in &all_cells {
            if !in_refs.contains_key(c) {
                no_in_refs.insert(c.clone());
            }
        }
        let mut ordered_cells: Vec<Arc<Cell>> = Vec::new();
        let mut indices: HashMap<Arc<Cell>, u32> = HashMap::new();
        while let Some(cell) = no_in_refs.iter().next().cloned() {
            ordered_cells.push(cell.clone());
            indices.insert(cell.clone(), indices.len() as u32);
            for child in cell.references() {
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

        RawBagOfCells {
            cells: ordered_cells
                .into_iter()
                .map(|cell| RawCell {
                    r#type: cell.as_type().into(),
                    data: cell.as_bitslice().into(),
                    references: cell
                        .references()
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
        }
        .pack_with(writer, args)
    }
}

/// ```tlb
/// serialized_boc_idx#68ff65f3 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   = BagOfCells;///

/// serialized_boc_idx_crc32c#acc3a728 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:uint32 = BagOfCells;///

/// serialized_boc#b5ee9c72 has_idx:(## 1) has_crc32c:(## 1)
///   has_cache_bits:(## 1) flags:(## 2) { flags = 0 }
///   size:(## 3) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots >= 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   root_list:(roots * ##(size * 8))
///   index:has_idx?(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:has_crc32c?uint32
///   = BagOfCells;
/// ```
impl BitUnpack for BagOfCells {
    fn unpack<R>(reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let raw = RawBagOfCells::unpack(reader)?;
        let num_cells = raw.cells.len();
        let mut cells: Vec<Arc<Cell>> = Vec::new();
        for (i, raw_cell) in raw.cells.into_iter().enumerate().rev() {
            cells.push({
                let references = raw_cell
                    .references
                    .into_iter()
                    .map(|r| {
                        if r <= i as u32 {
                            return Err(Error::custom(format!(
                                "references to previous cells are not supported: [{i}] -> [{r}]"
                            )));
                        }
                        Ok(cells[num_cells - 1 - r as usize].clone())
                    })
                    .collect::<Result<_, _>>()?;

                Arc::new(match raw_cell.r#type {
                    RawCellType::Ordinary => Cell::Ordinary(OrdinaryCell {
                        data: raw_cell.data,
                        references,
                    }),
                    RawCellType::LibraryReference => {
                        if !references.is_empty() {
                            return Err(Error::custom("library reference cannot have references"));
                        }

                        Cell::LibraryReference(LibraryReferenceCell {
                            data: raw_cell.data,
                        })
                    }
                    RawCellType::PrunedBranch => {
                        if !references.is_empty() {
                            return Err(Error::custom("pruned branch cannot have references"));
                        }

                        Cell::PrunedBranch(PrunedBranchCell {
                            level: raw_cell.level,
                            data: raw_cell.data,
                        })
                    }
                    RawCellType::MerkleProof => Cell::MerkleProof(MerkleProofCell {
                        level: raw_cell.level,
                        data: raw_cell.data,
                        references,
                    }),
                    _ => unimplemented!(),
                })
            });
        }
        Ok(BagOfCells {
            roots: raw
                .roots
                .into_iter()
                .map(|r| cells[num_cells - 1 - r as usize].clone())
                .collect(),
        })
    }
}

const CRC_32_ISCSI: Crc<u32> = Crc::<u32>::new(&crc::CRC_32_ISCSI);

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
struct RawBagOfCells {
    pub cells: Vec<RawCell>,
    pub roots: Vec<u32>,
}

impl RawBagOfCells {
    ///```tlb
    /// serialized_boc_idx#68ff65f3
    /// ```
    const INDEXED_BOC_TAG: u32 = 0x68ff65f3;

    /// ```tlb
    /// serialized_boc_idx_crc32c#acc3a728
    /// ```
    const INDEXED_CRC32_TAG: u32 = 0xacc3a728;

    /// ```tlb
    /// serialized_boc#b5ee9c72
    /// ```
    const GENERIC_BOC_TAG: u32 = 0xb5ee9c72;
}

impl BitPackWithArgs for RawBagOfCells {
    type Args = BagOfCellsArgs;

    fn pack_with<W>(&self, mut writer: W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter,
    {
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

        let mut buffered = writer.as_mut().tee(BitVec::<u8, Msb0>::new());
        buffered
            // serialized_boc#b5ee9c72
            .pack(Self::GENERIC_BOC_TAG)?
            // has_idx:(## 1)
            .pack(args.has_idx)?
            // has_crc32c:(## 1)
            .pack(args.has_crc32c)?
            // has_cache_bits:(## 1)
            .pack(false)?
            // flags:(## 2) { flags = 0 }
            .pack_as::<u8, NBits<2>>(0)?
            // size:(## 3) { size <= 4 }
            .pack_as::<_, NBits<3>>(size_bytes)?
            // off_bytes:(## 8) { off_bytes <= 8 }
            .pack_as::<_, NBits<8>>(off_bytes)?
            // cells:(##(size * 8))
            .pack_as_with::<_, VarNBytes>(self.cells.len() as u32, size_bytes)?
            // roots:(##(size * 8)) { roots >= 1 }
            .pack_as_with::<_, VarNBytes>(1u32, size_bytes)? // single root
            // absent:(##(size * 8)) { roots + absent <= cells }
            .pack_as_with::<_, VarNBytes>(0u32, size_bytes)? // complete BoCs only
            // tot_cells_size:(##(off_bytes * 8))
            .pack_as_with::<_, VarNBytes>(tot_cells_size, off_bytes)?
            // root_list:(roots * ##(size * 8))
            .pack_as_with::<_, VarNBytes>(0u32, size_bytes)?; // root should have index 0
        if args.has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            buffered.pack_many_as_with::<_, VarNBytes>(index, off_bytes)?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        for (i, cell) in self.cells.iter().enumerate() {
            cell.pack_with(&mut buffered, size_bytes)
                .with_context(|| format!("[{i}]"))?;
        }

        let buf = buffered.into_writer();
        if buf.len() % 8 != 0 {
            return Err(Error::custom("produced stream is not byte-aligned"));
        }
        // crc32c:has_crc32c?uint32
        if args.has_crc32c {
            let cs = CRC_32_ISCSI.checksum(buf.as_raw_slice());
            writer.write_bitslice(cs.to_le_bytes().as_bits())?;
        }
        Ok(())
    }
}

impl BitUnpack for RawBagOfCells {
    fn unpack<R>(mut reader: R) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let mut buffered = reader.as_mut().tee(BitVec::<u8, Msb0>::new());

        let tag = buffered.unpack::<u32>()?;
        let (has_idx, has_crc32c) = match tag {
            Self::INDEXED_BOC_TAG => (true, false),
            Self::INDEXED_CRC32_TAG => (true, true),
            Self::GENERIC_BOC_TAG => {
                // has_idx:(## 1) has_crc32c:(## 1)
                let (has_idx, has_crc32c) = buffered.unpack()?;
                // has_cache_bits:(## 1)
                let _has_cache_bits: bool = buffered.unpack()?;
                // flags:(## 2) { flags = 0 }
                let _flags: u8 = buffered.unpack_as::<_, NBits<2>>()?;
                (has_idx, has_crc32c)
            }
            _ => return Err(Error::custom(format!("invalid BoC tag: {tag:#x}"))),
        };
        // size:(## 3) { size <= 4 }
        let size_bytes: u32 = buffered.unpack_as::<_, NBits<3>>()?;
        if size_bytes > 4 {
            return Err(Error::custom(format!("invalid size: {size_bytes}")));
        }
        // off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes: u32 = buffered.unpack_as::<_, NBits<8>>()?;
        if size_bytes > 8 {
            return Err(Error::custom(format!("invalid off_bytes: {off_bytes}")));
        }
        // cells:(##(size * 8))
        let cells: u32 = buffered.unpack_as_with::<_, VarNBytes>(size_bytes)?;
        // roots:(##(size * 8)) { roots >= 1 }
        let roots: u32 = buffered.unpack_as_with::<_, VarNBytes>(size_bytes)?;
        // absent:(##(size * 8)) { roots + absent <= cells }
        let absent: u32 = buffered.unpack_as_with::<_, VarNBytes>(size_bytes)?;
        if roots + absent > cells {
            return Err(Error::custom("roots + absent > cells"));
        }
        // tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size: usize = buffered.unpack_as_with::<_, VarNBytes>(off_bytes)?;
        let root_list = if tag == Self::GENERIC_BOC_TAG {
            // root_list:(roots * ##(size * 8))
            buffered
                .unpack_iter_as_with::<_, VarNBytes>(size_bytes)
                .take(roots as usize)
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            let _index: Vec<u32> = buffered
                .unpack_iter_as_with::<_, VarNBytes>(off_bytes)
                .take(cells as usize)
                .collect::<Result<_, _>>()?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        let cell_data: Vec<RawCell> = buffered
            .unpack_iter_with(size_bytes)
            .take(cells as usize)
            .collect::<Result<_, _>>()
            .context("cell_data")?;

        let buf = buffered.into_writer();
        if buf.len() % 8 != 0 {
            return Err(Error::custom("produced stream is not byte-aligned"));
        }
        if has_crc32c {
            // crc32c:has_crc32c?uint32
            let cs = u32::from_le_bytes(reader.unpack()?);
            if cs != CRC_32_ISCSI.checksum(buf.as_raw_slice()) {
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
    pub r#type: RawCellType,
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<u32>,
    pub level: u8,
}

impl BitUnpackWithArgs for RawCell {
    /// size_bytes
    type Args = u32;

    fn unpack_with<R>(mut reader: R, size_bytes: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader,
    {
        let refs_descriptor: u8 = reader.unpack()?;
        let level: u8 = refs_descriptor >> 5;
        let is_exotic: bool = refs_descriptor >> 3 & 0b1 == 1;
        let ref_num: usize = refs_descriptor as usize & 0b111;

        let bits_descriptor: u8 = reader.unpack()?;
        let num_bytes = if is_exotic {
            ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize - 1
        } else {
            ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize
        };
        let full_bytes = (bits_descriptor & 1) == 0;
        let r#type = if is_exotic {
            reader.unpack::<RawCellType>()?
        } else {
            RawCellType::Ordinary
        };

        let mut data: BitVec<u8, Msb0> = reader.unpack_with(num_bytes * 8)?;
        if !data.is_empty() && !full_bytes {
            let trailing_zeros = data.trailing_zeros();
            if trailing_zeros >= 8 {
                return Err(Error::custom("last byte must be non zero"));
            }
            data.truncate(data.len() - trailing_zeros - 1);
        }

        let references: Vec<u32> = reader
            .unpack_iter_as_with::<_, VarNBytes>(size_bytes)
            .take(ref_num)
            .collect::<Result<_, _>>()?;

        Ok(RawCell {
            r#type,
            data,
            references,
            level,
        })
    }
}

impl BitPackWithArgs for RawCell {
    /// ref_size_bytes
    type Args = u32;

    fn pack_with<W>(&self, mut writer: W, ref_size_bytes: Self::Args) -> Result<(), W::Error>
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
            writer.repeat_bit(8 - padding_bits - 1, false)?;
        }

        writer.pack_many_as_with::<_, &VarNBytes>(&self.references, ref_size_bytes)?;

        Ok(())
    }
}

impl RawCell {
    fn size(&self, ref_size_bytes: u32) -> u32 {
        let data_len: u32 = (self.data.len() as u32 + 7) / 8;
        2 + data_len + self.references.len() as u32 * ref_size_bytes
    }
}

#[cfg(test)]
mod tests {
    use crate::boc::BagOfCells;
    use tlb::bits::de::unpack_bytes;
    use tlb::cell_type::CellType;

    #[test]
    fn block_header_with_merkle_proof_and_pruned_branch() {
        let bytes = hex::decode("b5ee9c720102070100014700094603a7f81658c6047b243f495ae6ba8787517814431f2c1c7896fabe8361b9e16587001601241011ef55aaffffff110203040501a09bc7a9870000000004010267a7050000000100ffffffff000000000000000066e43ab200002cb04eecad8000002cb04eecad847897845d000940eb0267a6ff0267a3d4c40000000800000000000001ee0628480101b815af9b18dca15b27b79ff26f4adfc5613df7a17b27f96bc0593d12f2b9170e0003284801011b9a32271632c8170fbc0071e0f2800c58496f9959021e4ac344f93b69915e69001528480101a98f69c6479a583577cd185eaa589db44e6a49715918356393ae68638fe9c01c0007009800002cb04edd6b440267a7040cd9841277aacd63b5597bfa64fc63aac32be67009332d5ff80e8658acf9cd28dc9b686e30ddfbf904215e24bc991eebe45d5bfd4d26f31f2dee712e67926048").unwrap();

        let boc: BagOfCells = unpack_bytes(bytes).unwrap();

        let root = boc.single_root().unwrap();
        assert!(root.as_merkle_proof().expect("must be a merkle proof").verify()); 
        assert!(matches!(root.as_type(), CellType::MerkleProof));
        let child = root.references().first().unwrap();
        assert!(matches!(child.as_type(), CellType::Ordinary));
        let children = child.references();
        assert!(matches!(
            children.first().unwrap().as_type(),
            CellType::Ordinary
        ));
        assert!(matches!(
            children.get(1).unwrap().as_type(),
            CellType::PrunedBranch
        ));
        assert!(matches!(
            children.get(2).unwrap().as_type(),
            CellType::PrunedBranch
        ));
        assert!(matches!(
            children.get(3).unwrap().as_type(),
            CellType::PrunedBranch
        ));
    }
}
