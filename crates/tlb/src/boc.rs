//! Collection of types related to [Bag Of Cells](https://docs.ton.org/develop/data-formats/cell-boc#bag-of-cells)
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    ops::Div,
    sync::Arc,
};

use bitvec::mem::bits_of;
use crc::Crc;

use crate::{
    Cell, Context, Error, StringError,
    bits::{
        NBits, NoArgs, VarNBytes,
        bitvec::{order::Msb0, vec::BitVec, view::AsBits},
        de::{BitReader, BitReaderExt, BitUnpack},
        ser::{BitPack, BitWriter, BitWriterExt},
    },
};

/// Alias to [`BagOfCells`]
pub type BoC = BagOfCells;

/// [Bag Of Cells](https://docs.ton.org/develop/data-formats/cell-boc#bag-of-cells) is used to **de**/**ser**ialize a set of cells from/into
/// bytes.
///
/// ```rust
/// # use tlb::{
/// #     Data,
/// #     bits::{de::unpack_fully, ser::{BitWriterExt, pack}},
/// #     BagOfCells, BagOfCellsArgs, Cell,
/// #     ser::CellSerializeExt,
/// #     StringError,
/// # };
/// # fn main() -> Result<(), StringError> {
/// let data: u32 = 1234;
/// let mut builder = Cell::builder();
/// builder.pack(data, ())?;
/// let root = builder.into_cell();
///
/// let boc = BagOfCells::from_root(root);
/// let packed = pack(boc, BagOfCellsArgs {
///     has_idx: false,
///     has_crc32c: true,
/// })?;
///
/// let unpacked: BagOfCells = unpack_fully(&packed, ())?;
/// let got: u32 = unpacked
///     .single_root()
///     .unwrap()
///     .parse_fully_as::<_, Data>(())?;
///
/// assert_eq!(got, data);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, Eq)]
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

    /// Consume `self` and return single root or `None` otherwise
    #[inline]
    pub fn into_single_root(self) -> Option<Arc<Cell>> {
        let [root] = self.roots.try_into().ok()?;
        Some(root)
    }

    /// Returns the root cells as a slice.
    #[inline]
    pub fn roots(&self) -> &[Arc<Cell>] {
        self.roots.as_ref()
    }

    /// Consumes `self` and returns the root cells.
    #[inline]
    pub fn into_roots(self) -> Vec<Arc<Cell>> {
        self.roots
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

    pub fn serialize(&self, args: BagOfCellsArgs) -> Result<Vec<u8>, StringError> {
        let mut buf = BitVec::new();
        self.pack(&mut buf, args)?;
        if buf.len() % bits_of::<u8>() != 0 {
            return Err(Error::custom("data is not aligned"));
        }
        Ok(buf.into_vec())
    }

    /// Parse from bytes
    #[inline]
    pub fn deserialize(bytes: impl AsRef<[u8]>) -> Result<Self, StringError> {
        Self::unpack(&mut bytes.as_bits(), ())
    }

    /// Parse hexadecimal string
    #[inline]
    pub fn parse_hex(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        hex::decode(s)
            .map_err(Error::custom)
            .and_then(Self::deserialize)
    }

    /// Parse base64-encoded string
    #[cfg(feature = "base64")]
    #[inline]
    pub fn parse_base64(s: impl AsRef<[u8]>) -> Result<Self, StringError> {
        use base64::{Engine, engine::general_purpose::STANDARD};

        STANDARD
            .decode(s)
            .map_err(Error::custom)
            .and_then(Self::deserialize)
    }
}

impl Debug for BagOfCells {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(&self.roots).finish()
    }
}

/// [`BitPack::Args`] for [`BagOfCells`]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
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
///   = BagOfCells;
///
/// serialized_boc_idx_crc32c#acc3a728 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:uint32 = BagOfCells;
///
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
impl BitPack for BagOfCells {
    type Args = BagOfCellsArgs;

    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
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

        RawBagOfCells {
            cells: ordered_cells
                .into_iter()
                .map(|cell| RawCell {
                    data: cell.data.clone(),
                    references: cell
                        .references
                        .iter()
                        .map(|c| *indices.get(c).unwrap())
                        .collect(),
                    is_exotic: cell.is_exotic,
                    level: cell.level(),
                })
                .collect(),
            roots: self
                .roots
                .iter()
                .map(|c| *indices.get(c).unwrap())
                .collect(),
        }
        .pack(writer, args)
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
///   = BagOfCells;
///
/// serialized_boc_idx_crc32c#acc3a728 size:(## 8) { size <= 4 }
///   off_bytes:(## 8) { off_bytes <= 8 }
///   cells:(##(size * 8))
///   roots:(##(size * 8)) { roots = 1 }
///   absent:(##(size * 8)) { roots + absent <= cells }
///   tot_cells_size:(##(off_bytes * 8))
///   index:(cells * ##(off_bytes * 8))
///   cell_data:(tot_cells_size * [ uint8 ])
///   crc32c:uint32 = BagOfCells;
///
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
impl<'de> BitUnpack<'de> for BagOfCells {
    type Args = ();

    fn unpack<R>(reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let raw = RawBagOfCells::unpack(reader, ())?;
        let num_cells = raw.cells.len();
        let mut cells: Vec<Arc<Cell>> = Vec::new();
        for (i, raw_cell) in raw.cells.into_iter().enumerate().rev() {
            cells.push(
                Cell {
                    is_exotic: raw_cell.is_exotic,
                    data: raw_cell.data,
                    references: raw_cell
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
                        .collect::<Result<_, _>>()?,
                }
                .into(),
            );
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

impl TryFrom<Vec<u8>> for BagOfCells {
    type Error = StringError;

    #[inline]
    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Self::deserialize(value)
    }
}

#[cfg(feature = "arbitrary")]
const _: () = {
    use arbitrary::{Arbitrary, Result, Unstructured};

    impl<'a> Arbitrary<'a> for BagOfCells {
        #[inline]
        fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
            Cell::arbitrary(u).map(Self::from_root)
        }

        #[inline]
        fn size_hint(depth: usize) -> (usize, Option<usize>) {
            Cell::size_hint(depth)
        }

        fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
            Cell::arbitrary_take_rest(u).map(Self::from_root)
        }
    }
};

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

impl BitPack for RawBagOfCells {
    type Args = BagOfCellsArgs;

    fn pack<W>(&self, writer: &mut W, args: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        if self.roots.len() > 1 {
            return Err(Error::custom("only single root cell supported"));
        }
        let size_bits: u32 = 32 - (self.cells.len() as u32).leading_zeros();
        let size_bytes: u32 = size_bits.div_ceil(8);

        let mut tot_cells_size: u32 = 0;
        let mut index = Vec::<u32>::with_capacity(self.cells.len());
        for cell in &self.cells {
            index.push(tot_cells_size);
            tot_cells_size += cell.size(size_bytes);
        }

        let off_bits: u32 = 32 - tot_cells_size.leading_zeros();
        let off_bytes: u32 = off_bits.div_ceil(8);

        let mut buffered = writer.as_mut().tee(BitVec::<u8, Msb0>::new());
        buffered
            // serialized_boc#b5ee9c72
            .pack(Self::GENERIC_BOC_TAG, ())?
            // has_idx:(## 1)
            .pack(args.has_idx, ())?
            // has_crc32c:(## 1)
            .pack(args.has_crc32c, ())?
            // has_cache_bits:(## 1)
            .pack(false, ())?
            // flags:(## 2) { flags = 0 }
            .pack_as::<u8, NBits<2>>(0, ())?
            // size:(## 3) { size <= 4 }
            .pack_as::<_, NBits<3>>(size_bytes, ())?
            // off_bytes:(## 8) { off_bytes <= 8 }
            .pack_as::<_, NBits<8>>(off_bytes, ())?
            // cells:(##(size * 8))
            .pack_as::<_, VarNBytes>(self.cells.len() as u32, size_bytes)?
            // roots:(##(size * 8)) { roots >= 1 }
            .pack_as::<_, VarNBytes>(1u32, size_bytes)? // single root
            // absent:(##(size * 8)) { roots + absent <= cells }
            .pack_as::<_, VarNBytes>(0u32, size_bytes)? // complete BoCs only
            // tot_cells_size:(##(off_bytes * 8))
            .pack_as::<_, VarNBytes>(tot_cells_size, off_bytes)?
            // root_list:(roots * ##(size * 8))
            .pack_as::<_, VarNBytes>(0u32, size_bytes)?; // root should have index 0
        if args.has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            buffered.pack_many_as::<_, VarNBytes>(index, off_bytes)?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        for (i, cell) in self.cells.iter().enumerate() {
            cell.pack(&mut buffered, size_bytes)
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

impl<'de> BitUnpack<'de> for RawBagOfCells {
    type Args = ();

    fn unpack<R>(reader: &mut R, _: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let mut buffered = reader.as_mut().tee(BitVec::<u8, Msb0>::new());

        let tag = buffered.unpack::<u32>(())?;
        let (has_idx, has_crc32c) = match tag {
            Self::INDEXED_BOC_TAG => (true, false),
            Self::INDEXED_CRC32_TAG => (true, true),
            Self::GENERIC_BOC_TAG => {
                // has_idx:(## 1) has_crc32c:(## 1)
                let (has_idx, has_crc32c) = buffered.unpack(NoArgs::EMPTY)?;
                // has_cache_bits:(## 1)
                let _has_cache_bits: bool = buffered.unpack(())?;
                // flags:(## 2) { flags = 0 }
                let _flags: u8 = buffered.unpack_as::<_, NBits<2>>(())?;
                (has_idx, has_crc32c)
            }
            _ => return Err(Error::custom(format!("invalid BoC tag: {tag:#x}"))),
        };
        // size:(## 3) { size <= 4 }
        let size_bytes: u32 = buffered.unpack_as::<_, NBits<3>>(())?;
        if size_bytes > 4 {
            return Err(Error::custom(format!("invalid size: {size_bytes}")));
        }
        // off_bytes:(## 8) { off_bytes <= 8 }
        let off_bytes: u32 = buffered.unpack_as::<_, NBits<8>>(())?;
        if off_bytes > 8 {
            return Err(Error::custom(format!("invalid off_bytes: {off_bytes}")));
        }
        // cells:(##(size * 8))
        let cells: u32 = buffered.unpack_as::<_, VarNBytes>(size_bytes)?;
        // roots:(##(size * 8)) { roots >= 1 }
        let roots: u32 = buffered.unpack_as::<_, VarNBytes>(size_bytes)?;
        // absent:(##(size * 8)) { roots + absent <= cells }
        let absent: u32 = buffered.unpack_as::<_, VarNBytes>(size_bytes)?;
        if roots + absent > cells {
            return Err(Error::custom("roots + absent > cells"));
        }
        // tot_cells_size:(##(off_bytes * 8))
        let _tot_cells_size: usize = buffered.unpack_as::<_, VarNBytes>(off_bytes)?;
        let root_list = if tag == Self::GENERIC_BOC_TAG {
            // root_list:(roots * ##(size * 8))
            buffered
                .unpack_iter_as::<_, VarNBytes>(size_bytes)
                .take(roots as usize)
                .collect::<Result<_, _>>()?
        } else {
            Vec::new()
        };
        if has_idx {
            // index:has_idx?(cells * ##(off_bytes * 8))
            let _index: Vec<u32> = buffered
                .unpack_iter_as::<_, VarNBytes>(off_bytes)
                .take(cells as usize)
                .collect::<Result<_, _>>()?;
        }
        // cell_data:(tot_cells_size * [ uint8 ])
        let cell_data: Vec<RawCell> = buffered
            .unpack_iter(size_bytes)
            .take(cells as usize)
            .collect::<Result<_, _>>()
            .context("cell_data")?;

        let buf = buffered.into_writer();
        if buf.len() % 8 != 0 {
            return Err(Error::custom("produced stream is not byte-aligned"));
        }
        if has_crc32c {
            // crc32c:has_crc32c?uint32
            let cs = u32::from_le_bytes(reader.unpack(())?);
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
    pub data: BitVec<u8, Msb0>,
    pub references: Vec<u32>,
    pub is_exotic: bool,
    pub level: u8,
}

impl<'de> BitUnpack<'de> for RawCell {
    /// size_bytes
    type Args = u32;

    fn unpack<R>(reader: &mut R, size_bytes: Self::Args) -> Result<Self, R::Error>
    where
        R: BitReader<'de> + ?Sized,
    {
        let refs_descriptor: u8 = reader.unpack(())?;
        let level: u8 = refs_descriptor >> 5;
        let is_exotic: bool = (refs_descriptor >> 3) & 0b1 == 1;
        let has_hashes: bool = (refs_descriptor >> 4) & 0b1 == 1;
        let ref_num: usize = refs_descriptor as usize & 0b111;

        let bits_descriptor: u8 = reader.unpack(())?;
        if has_hashes {
            let hashes_num = level.count_ones() + 1;
            reader.skip((hashes_num as usize) * (32 + 2) * 8)?;
        }

        let num_bytes: usize = ((bits_descriptor >> 1) + (bits_descriptor & 1)) as usize;
        let full_bytes = (bits_descriptor & 1) == 0;

        let mut data: BitVec<u8, Msb0> = reader.unpack(num_bytes * 8)?;
        if !data.is_empty() && !full_bytes {
            let trailing_zeros = data.trailing_zeros();
            if trailing_zeros >= 8 {
                return Err(Error::custom("last byte must be non zero"));
            }
            data.truncate(data.len() - trailing_zeros - 1);
        }

        let references: Vec<u32> = reader
            .unpack_iter_as::<_, VarNBytes>(size_bytes)
            .take(ref_num)
            .collect::<Result<_, _>>()?;

        Ok(RawCell {
            data,
            references,
            is_exotic,
            level,
        })
    }
}

impl BitPack for RawCell {
    /// ref_size_bytes
    type Args = u32;

    fn pack<W>(&self, writer: &mut W, ref_size_bytes: Self::Args) -> Result<(), W::Error>
    where
        W: BitWriter + ?Sized,
    {
        let is_exotic: u8 = if self.is_exotic { 1 } else { 0 };
        let refs_descriptor: u8 = self.references.len() as u8 + is_exotic * 8 + self.level * 32;
        writer.pack(refs_descriptor, ())?;

        let padding_bits = self.data.len() % 8;
        let full_bytes = padding_bits == 0;
        let bits_descriptor: u8 = self.data.len().div(8) as u8 + self.data.len().div_ceil(8) as u8;
        writer.pack(bits_descriptor, ())?;

        writer.write_bitslice(&self.data)?;
        if !full_bytes {
            writer.write_bit(true)?;
            writer.repeat_bit(8 - padding_bits - 1, false)?;
        }

        writer.pack_many_as::<_, &VarNBytes>(&self.references, ref_size_bytes)?;

        Ok(())
    }
}

impl RawCell {
    fn size(&self, ref_size_bytes: u32) -> u32 {
        let data_len: u32 = self.data.len().div_ceil(8) as u32;
        2 + data_len + self.references.len() as u32 * ref_size_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_boc() {
        let hex_data = "b5ee9c72e202014400010000269c0000002400cc00f4018a026c0308033a035c036b0384039e040e047e04ca057205b206a406dc078407c408b609260973099609ba0a660a860aa40ac20ae00afe0b1a0b360b520b6e0b8a0ba60c4c0cd00cf40d140d600dac0dcc0dec0e0c0e2c0e4c0e6c0e8c0eac0ecc0f760ffe105e107010ee113a120612ae12bc12ca1360136e137c138a139813a613b413c213d013de13ec13fa14ae14cc14ea15081526154415621580159e15bc15da15f81616163416521670168e169c16aa16b816c616d416e216f016fe170c171a1728173617821790179e17ac17ba17c817d617e417f21800184c1870189418e1198c19ac19ca1a171a631a801a9e1aeb1b371b541b701bbd1c091c241c401c8d1cd91cf41d101d5d1da91dc41e6a1eb71f3a1f871fd920242044209120dd20fc211c2169218821d522212240228d22ac22cc23192338238523d123f0243d245c2506255325da2627268626d326e4273127ae27fb28472912295f2a062a142a222a6f2b042b122b5f2b6c2bb92bc62c132c202c2e2c7b2cc72cd42d212d2e2d3c2d892dd52de22df02e3d2e892e962f4a2f972fe33000304d306a30b730d43121313e318b31a831f53212325f327c32c932e633333350339d33ba340734243471348e34db34f83545356235af35cc3619363636543702374f379b37a837b637c43811381e386b387838c538d2391f392c3979398639d339e03a2d3a3a3a873a943ae13aee3ba53bb23bff3c763d2a3d383d853d923ddf3dec3e393e853e923ea03eed3f393f463f543fa13fae3ffb40474054406240af41624216422242284232425a42ae42be4366440a4416442244a8456845ee460046a44764476b47f1480248a648b348f6490049e449fc4a0a4a194ad94ae24b684b844c354cd84d39041011ef55aaffffff11000100020003000401a09bc7a9870000000004010377cfc90000000100ffffffff000000000000000069b196d700003db46c934b4000003db46c934b442b654f63000c25070377cfc60377b158c40000000d00000000000003ee0005021b3ebf98b74a7d6b99120373ff48a0000600070a8a045e83a496b5a219ec80ebdf6a5b906b84eaab08d1ef50f3b26e11e0d12762cc85f4c95e4105c1b00d3bac19a9a9eeaa8f4365aaa005ad03898e5df4ce937201fe01700170000b000c148997a7ef54adcedf1a8eb578eca3fa3c1cc2bbcb8b4959ad0ca1d372669604f7be00074a33f6fd2a81cabc129465ab865659f2cb3cd430170e28546715051cfdc127c54338f3592294c3e072372b7cbaf412023c778c2f32d8f23d5cf35fb3bd1b7974b8e37e29c00122012301240125009800003db46c8409040377cfc825ea22c3370d423812b3fa0c387164b3d5ea70ffea1108b4f7cf0d77d209ebce776bbcf3afc7bdaebfafe713212d686402cf3c8b976eae7b3aa5e5e8e45f754c022581d1deceeb2d6df5cc0e8ef677ad56cc76c00800080008001d4496ac722253eb5cc891954fc400080201200009000a0015be000003bcb355ab466ad00015bfffffffbcbd0efda563d0245b9023afe2ffffff1100ffffffff00000000000000000377cfc80000000169b196d500003db46c8409040377cfc560000d000e000f0010245b9023afe2ffffff1100ffffffff00000000000000000377cfc90000000169b196d700003db46c934b440377cfc660001100120013001428480101ea944515a1b255dd433cae9d4ad20f5a7b349e9bdea456aa3fde86cb4b56cc64000132138e5cf53c15c38dd863b0f5a952809cf44feae5e9f903bfce7c604fb71accff3cbcc3a782d3a36095219578baac7e9797381eb53c872c574ea53443b35565ed60016f00138207477b3bacb5b7d7300017009222330000000000000000ffffffffffffffff81d1deceeb2d6df5c828009200163455c3779efad35035383ccb44f879596d29bc076079ac1a1937a682784d511f2e1145f573f264c0c8649077f01fd8b0c0b3f0f55f3de1d36ee8e404396babaad15f001c0013cc26aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac23d09e4e56e75e17e003900af003a00b1012f00000000000000006000000000000000008000000000004000153213d85cef6e2f27a91d0aede412a64a1755e73ee82b872ce15abfa66864dfbc42a00ede7c7495cce6843f1425447e438cce0d9a345410e92a5cc3426e6b0e255cb6016f00138207477b3bd6ab663b700072009222330000000000000000ffffffffffffffff81d1decef5aad98ed828009200163455589cac44b8f19c98f56df4f74be5028656231e8f25574a17e9c544ce18782ce02441766f439585c0f9f0ba0a3246a7009c5af6e5e518d659c25e9dd4bf147e81001c0014cc26aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaac23d09e4ea75ebb97e013600af00b000b1006bb0400000000000000001bbe7e480001eda36420481ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffc02848010115f3da4137d3c618ccd3db684c139eca3f1efa11c50e00f1af1d690c9c696781002a23130103a3bd9dd65adbeb980018007400922313010290794025e3be1cb80019001a0092331321a6dc64f153639d5334268803f579c7fbf8e13036b880ea40ae8788619ed7676f9d43f11454724da59fe51723c85b088d89f7eeb164b8b761cb3179f0a1ee5f002e001001022e10d9951b5743f80027002800922213010062686690c866d8c8001b0078221100ed87b2c7cdd56ea80079001c221100e0bb46e407d091e8001d007c221100e0369ed7dfc0e3a8007d001e221100e033bce63a61fd28001f0080220f00c15cbc3935c68800810020220f00c1438847527fc800210084220f00c1410d7d25bd0800850022220f00c050fb6749c0a800230088220f00c0440a74ff940800890024220f00c044016bc0ad880025008c219dbceaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa818087f8564be4480cec350224cf54e63f4b5dbb58783dac3c1068180529aed0dd2f91e161779d400007b68d908120700262277cff5555555555555555555555555555555555555555555555555555555555555555410b0c2270200000000000000f6d1b21024118087f8564be455d0008e008f23130100420a0567b50b0e980029002a009222130101ec06d42d664c35680093002b284801018fef934f62af7cdc673c6fc9a6a72f7413abd3ae6295387a1cd81633ba6849a50028284801017d071f999abae859709c1b7ed27e045002a631b817501910cc2ec4222feb3399002c22130101dec13202a7a324e8002c009622130101c523088c51746fa8002d009822130101ae7b57000e3d01e80099002e22130101ae6dce051a55f4e8009b002f22130101ae6dc31c9a05c2680030009e22130101ae6b7d5e970d2648003100a022130101ae6b33fa3b6ba2e800a1003222130101ae6b337eb03ad10800a3003322130101ae6b33340fc27148003400a621a1bcd99999999999999999999999999999999999999999999999999999999999982035cd66663378edd82a33ebd0edb6781cced8e899e75f93ded39b3e573e7248f6f93435595dc4779400007b68d90812050035227bcff333333333333333333333333333333333333333333333333333333333333333340d74c208a9c0000000000000f6d1b210240e035cd66663378edd96d000a8003622536a04a47ea060d3629e11cf06cedf6c52ba673d4c27137a6bfb98b9adc309962dccd45afef1a119993e2b00aa003722058f69b100ac00382175a09e10d3659e1000010001cf06cedf6c52ba673d4c27137a6bfb98b9adc309962dccd45afef1a119993e2b80684e5944df9ff4e6133f4f4fd02b4000ae284801011abb1f220297dbb74a1cf53846a5d71f0e00c1e93cdce29610598c59d53d7000000222bf0001dd639bfa000c2507600007b68d8e98d8880001ed94d4b0a6201bbd8ac141c6f80a0dc19f93ca7d7915833fde80a514a08a26dc22eae15a6920a40da48449184f82ed3f1c90615d9e925680c2b8cf3e49fe16b7ebdeea81c33f00bf0880be003b003c32138afe50230492ba4f8198dfcc948760b8a86f2efd00e3ca9ce5c7d475a994c023b0290bc5c5e3e8b44c04329268c9cc82ba9c164c33434d6560b0f56717ba233f001a0011c340000f6d1b1d31b12000ce004b220120003d00b5220120003e003f320132b50bc52443078ea574f9c3afbee31da9ef17846ad6ee5c73eca438acc38f9ebfa4d21dd89c1daa6383b565bdcb1782fe54d42a20ba59cd0844dca07293a7ca000f000c20005b005c22012000b8004022012000ba004122012000bc0042220120004300bf22012000c0004422012000c20045220120004600c522012000c60047220120004800c922012000ca0049220148004a00cd00b0bcaf94dcb95433b0e907e5ed7756da5fcaef5b0d9d3bf9ded57a825f820d7f640000000000000000000000000000000000000000000000000000000069b196a2000000000000000e00000004f024fb940000000d2d2d32332211200007b68d8e98d89000d0004c2211480001eda363a6362400d2004d2211200007b68d8e98d89000d4004e2211200007b68d8e98d89000d6004f2211480001eda363a6362400d800502211200007b68d8e98d89000da00512211200007b68d8e98d89000dc00522211200007b68d8e98d89000de00532211200007b68d8e98d89000e0005422116000007b68d8e98d8900e200552211000007b68d8e98d89000e400562211000007b68d8e98d89000e600572211000007b68d8e98d89000e800582211000007b68d8e98d89000ea00592211000007b68d8e98d89000ec005a2211cc00007b68d8e98d8900f000f1220120005d00f52201200068010d220120005e00f7220120005f00f9220120006000fb220120006100fd220120006200ff2201200063010122012000640103220120006501052201200066010722012000670109284801019b3d0647b06f329fe419e65440c3ee50a686e3dfc096c4323d6762a2b0198a9800012201200069010f220120006a01112201200112006b220120006c01152201200116006d220120006e0119220120006f011b220120011c00702201200071011f28480101cba06cdc7540919af175e35a0bb02b6632a153e0f0778d68b1d87ea10f23c760000123130103a3bd9deb55b31db8007300740092231301029079403ade954ed8007500760092284801019fe0947cceac1acfcd4310870bd1368013dc16a806540fb1026dc2fe72c87c46016d331383b617c8dbab9b94b15dd0daadb2ec3031034d1ffa562b72d5a1d8c9fff275c7cbad2ea1c1365537e2eeede319327f84b50e5ae69c04245ffb107877d8125447002e001001022e10d9aa162e76180090009100922213010062686690c866d8c800770078221100ed87b2c7cdd56ea80079007a2848010175af14079379a6140e6587879f5ec30ce45064eb6e6d24fbca1779804e48a7b7002b2848010156a152d1770a1f4eb05f804338e86e2d4f8de283375d39a3abb31d89e24c4cc1001b221100e0bb46e407d091e8007b007c221100e0369ed7dfc0e3a8007d007e28480101f10ab3d52bed20b81e93dc8c76cd47b2e98626f6330714914e97a16a9503e419001d284801011f7f74017f13f6db46f42ba52e121d6dd5bce8b9f989d16190ffcbb17655fe88001b221100e033bce63a61fd28007f0080220f00c15cbc3935c6880081008228480101badb5663b25c4f4d678ca564e4e2deea8d9fdbcb227ddf4fe25d1d78a0904dc7001828480101628eeb4bf13ca5b28536b659a627c950bab9141ed5f49ff91417a91d5d0cbc790017220f00c1438847527fc800830084220f00c1410d7d25bd08008500862848010144c66b0d476e11d1de9b8996c1886586f81d68a24c5107529fc2af678cd3c13900152848010197b58e28a221493247cd4a77ca7852b664d3e37b77be81e8ff56ec45d68dbc500012220f00c050fb6749c0a800870088220f00c0440a74ff94080089008a284801019a38de250c568dab8863700a55160f220d85e2019da8fad060e69a9062880b0e000b28480101782e394646a43b351268d3cf73f2f750381ee0ca12825639cbc3834b7d8fa9110009220f00c044016bc0ad88008b008c219dbceaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa818087f8564be4430c1f9105c6fa1ba821810d72f182884dba7b97d19459bd83a0f251c4991ec7a00007b68d9269687008d284801010143b3d2dd671b2559543155e003f847022e510b3a57afabbca05d4069c327ef000d2277cff5555555555555555555555555555555555555555555555555555555555555555410b0c2270200000000000000f6d1b24d2d118087f8564be455d0008e008f28480101c4844c82c82277d65fba7dd320b390f16e89e9c106812e367b679bf0de1313e0000c21490000002a82b17caadb303d53c3286c06a6e1affc517d1bc1d3ef2e4489d18b873f5d7cd14000af2848010122c3225cfcf645b3b3473c3b99083e8b6ea981b627026336e08f652dc7f16986002d22130101ec06d442612367880093009428480101a5a7d24057d8643b2527709d986cda3846adcb3eddc32d28ec21f69e17dbaaef0001284801017f79c6ed120a540972cdd0e6f979db9e51eb82c45827a8577d3c8af9d6289ba8002722130101dec13217a27a57080095009622130101c52308a14c4ba1c8009700982848010100e5a763c94e89a79db9b8181bd9a2eff435808e3c4ac3ddf59e6d6343c3e2fb002622130101ae7b5715091434080099009a284801018dcc5bf5391e19bca37db761393859f432bac33fbac83e69cd89a52457f17eda0023284801018dbe11820a6a83d09b337c32959d2aa42cf01bcc13857f8782c5c7a3b709f1e4002522130101ae6dce1a152d2708009b009c28480101f3d650cae793633d0cea4478ef26b9fd646085480a8cb4e498faceb305d6888c001722130101ae6dc33194dcf488009d009e22130101ae6b7d7391e45868009f00a02848010132946c57a31876e37a2c97475ac4b7877b5ff80b5d807d6b024b8b643ef7ae16001722130101ae6b340f3642d50800a100a22848010197b847abf419303501fcfdaea47588651176862e46b6d754a136a247fc135cbe0013284801018b05fc39cabcc22da5c2fe02888333585ff2b42e3e45f2765d3d962ad98dc474001222130101ae6b3393ab12032800a300a428480101f963341ee7e9fa365598dcc0450b29f0706a885c8f49fc0025ed7983a8b14625001122130101ae6b33490a99a36800a500a621a1bcd99999999999999999999999999999999999999999999999999999999999982035cd6668d2d3d41cc4512a4e4e52f9d4508859d256a5610a400b55a8603a43a140af22bdd4690ea200007b68d926968500a728480101dae1005048ab005be350e728e9c485d84a876176642bba673d51fac8a4b5b79a0010227bcff333333333333333333333333333333333333333333333333333333333333333340d74c208a9c0000000000000f6d1b24d2d0e035cd6668d2d3d41d6d000a800a9284801016217f872c99fafcb870f2c11a362f59339be95095f70d00b9cff2f6dcd69d3dd000e22536a04a47ea060d3629e11cf06cedf6c52ba673d4c27137a6bfb98b9adc309962dccd45afef1a119993e2b00aa00ab28480101d6df899d93405ef2e25e7d6e61e28bab689a9991ff881b473b66a38bb1f57332000722058f69b100ac00ad28480101ea6857bc512a1ab01cd2ceb2a4252a583d2d743d38ffbceb0ef4f6ba27eaf67d000d2175a09e10d3659e1000010001cf06cedf6c52ba673d4c27137a6bfb98b9adc309962dccd45afef1a119993e2b80684e5944df9ff4e6133ff72689bc4000ae28480101213345872bbc25c75e4cd520d7be786e5e2b76456312a2021407c0d1f5f9a4fb000c284801010fe60cf50ccbd6885b8d77c1e584b92d4d1f8843b5a4a5b90f4da54b5ec2a119001222bf0001dd639bfa000c2507600007b68d908120880001ed94d4b0a6201bbd8ac141c6f80a0dc19f93ca7d7915833fde80a514a08a26dc22eae15a6920a40da48449184f82ed3f1c90615d9e925680c2b8cf3e49fe16b7ebdeea81c33f00bf0880be00b200b328480101b20e36a3b36a4cdee601106c642e90718b0a58daf200753dbb3189f956b494b600013213c35686dd1ef22c5fc9d40fe9fb1bcb97302b4293f82991eeaa69f525f99c7b94fc784e05c3e937ddf23474e68bf076aa327cafb2611e36eab9b50ea5e1f0913f001a0012c340000f6d1b2102412000ce00cf22012000b400b522012000b600b7284801013dd3ddf45df45d71772aa2bcc3793cdfd6238d3716295475d9f2b83b257756f800103201aaa7e81184eddba87ab51200c230f7f7e458dde8dcd58e128e3ff4a24d325dde0ab82b07b87874f425b49d5b81594f29d714710246eeefeb9f6ef726059425b5000f000d2000f200f322012000b800b928480101287035ff9a6fa02b4946e02beae6764ee6d257c14f656c0d28f92045bcdbab9f000e22012000ba00bb28480101ab7cdcc6f0ef6bdcd305691b879e544b73c39826f62a39606ed978c84a50e5a4000d22012000bc00bd284801017ccf7fa75bfc053ba3e6aeded2360296038c6779a7427638ae44683d2e99a534000c22012000be00bf22012000c000c128480101296457042c003e416df7cf81f2af1dcd14baeacadb3ecd485810f08597cae57d000a284801013a1b8fe2731e74393ed93ae4d14e834922f313f0a25021f07eadc32329754c59000922012000c200c328480101834fea4749d47f92a677aca2b1a35d82f646b6fe4766e283e0749efe8d194907000822012000c400c522012000c600c728480101572ac2da3c800965d94514558cdf664146053834de44c215b95d6dfa399d4890000928480101640d18200c18b5c83f7c71a0c3ee659dc61107edc820ca74f8f1645c7c9f10ed000622012000c800c922012000ca00cb28480101d1d96faf78f4d28f1ad8c726e28a73314e7f981e357614c633f922662f5c2ec100012848010148210f6051739208dbd55adffd65d6bef311b78fdd88d36863ff483908fdce86000122014800cc00cd00b0bcaf94dcb95433b0e907e5ed7756da5fcaef5b0d9d3bf9ded57a825f820d7f640000000000000000000000000000000000000000000000000000000069b196d7000000000000000f00000005cfd976d80000000e2a73201d2848010114569a552373ff0c994688e33f71fda9e9345152bf3e069f5858999208da6cf3000328480101a896079a068698f9843115db2bdd1c4eb6ca1d1acbe84ba39934b4326020f17600192211200007b68d9081209000d000d128480101cd8886acc9a49753fd2c989f25ef5f8d3e0ed60508584a6173116405392ce74100182211480001eda36420482400d200d328480101b00aaa9144910375892fb5c55df9af8a206648e3eb6b0dacd89ea73e574442d300162211200007b68d9081209000d400d5284801019d8ae07c8b8755c039e17b2a3b597706428858162bdabde4649bef7722c6729800152211200007b68d9081209000d600d7284801014a60e0f514536c5dbd3fc561749059aeb8a15e3cea8f6560c62d1cbd54ac2ca100142211480001eda36420482400d800d92848010128b47e69bc21aa163d1fab1631a004a851486387218fc94f557befd902a464dd00122211200007b68d9081209000da00db28480101db5edc99d10c56449723b65a52b2c152d8c2f5b89547f9cb8735b5a07500627600112211200007b68d9081209000dc00dd2848010138b034f96734341a1fcc42bf8ed378a4049caa3458cf8798acfeabdf8c22398100102211200007b68d9081209000de00df28480101fa21ca4a2c330e3db8f7da3df9be47d2b9c80887fc03e08f3b2dc8d1fe87a3e2000f2211200007b68d9081209000e000e128480101afc39030e5608659cb65b39601267a2740ed20f42741797f55f6fc274eed4320000e22116000007b68d908120900e200e328480101df6075c1fcd4bd0b7ef803f927440665a4d2d1c09296eedeaf7468f33d97a650000b2211000007b68d9081209000e400e528480101c70858f68c7af860829d3816f7da0d5873ec2015cab1eff577a37a0bf71160cb000a2211000007b68d9081209000e600e72848010167c1678b3ac22276a92cefb084c921d1f9e552c430c9dccd1e20bef1ba02cdf300092211000007b68d9081209000e800e9284801014ec41c464e20fa299d49e36fd4ff1b6cdfea28c8a4e45ca0d23d3a9e0c5fa77800082211000007b68d9081209000ea00eb284801015cf9f29c781a3affc788a91cb45c746b293fbaf1b6cd2375560fdc1cc66bf51100072211000007b68d9081209000ec00ed2848010181e39f600a046f736888106e62e65542716fba47a8b88e6f6f2d34b52d136aec000622116000007b68d908120900ee00ef2211000007b68d8e98d89000f000f100a9d80000f6d1b210241000007b68d908120806ef9f904bd445866e1a84702567f41870e2c967abd4e1ffd4221169ef9e1aefa413d79ceed779e75f8f7b5d7f5fce26425ad0c8059e79172edd5cf6754bcbd1c8beea9928480101ff764dd5d920b44f1fa74f3bc56f1e0606f6006e7869d86df1efe9d00d8e2eea0002284801011acaa15d0a863caeb6e2014bc3eebca45a961029b9677e99896257311828c755000222012000f400f5220120010c010d22012000f600f7284801018ec4ed074e2bed443c3b7c5731140b2536b4973f1d39df45ca75857f94dd48d4000d22012000f800f928480101500449e2ebbfaa1db1ca4a2fc10d179b3eec40100da5e927c904b2b25105868c000b22012000fa00fb28480101728b882614eb2b243aa2bb24ef2fa6dc1b4da319853a6cde06ec9ff65d46febf000a22012000fc00fd28480101e66c18421bb08b4cdf6c1ddd467783f47f6bfea4a995bb36afca36d4d2c0030d000922012000fe00ff28480101e8958c76e11bc36c37c176a18a24ba9be11b15bbaa471b22e472043a0ef9e9ab000822012001000101284801012ba1b96a3efa478ae365eae5f7a83c392cc4a3779dc74c104108d6655968b2500006220120010201032848010139005b2e25898785414ca2ef23f622fca56f7ee9e0421f0183ac1894182f31510005220120010401052848010189668259541cecc9b507e90d55542cf4be5525914a1bb9e12711b175a404f85100032201200106010728480101a87c200ef507871300468825d73e2d84a8fd0dadb659b6940d07c1bcc4f3ed0800022201200108010900b1bcd50a77b4596dcfd79b58ed75d82a18fb95b9014875d126733a7ff9a614cca20000000000000000000000000000000000000000000000000000000034d0a524800000000000002280000002771faff000000016e0a0838bc0020120010a010b284801013613709e6a02fba0a5512036e0f88bbe943790223ac9de4b43d76d01d6543f4300010073de28d3632dae0000000006ef9f90000006cfe3ca18d20000d901c5b93f8cd3632dae000000001d7b073800000650480c058c0000cd16347af13300afbc6449560bdc4f6acd659a7762f260469aad1ef70e904a5de82c44c1231c268800000000000000000000000000000000000000000000000000000000d32e9d4800000000000000640000000989187f7e0000003ceb87dd1b220120010e010f28480101d121e31a7368cf45801b9138074eb229e309c7d875805423db47ff2d56bb03b2000c2201200110011128480101580413edd87480343e5c0164b2162b1a408dec78ff71d1cdd6b30576a3fb8175000c2201200112011328480101caf160e6ed7e1f98a66669cfba42ec1f1f87b1699810110d4369ee5facb23719000a28480101d9f103e3a400c89138988e84809679dfe0ea612e2e496e1b9f84750108dea4100008220120011401152201200116011728480101d2fb50e46e3b86da7f94a2f6a642bfe3e5f9750c416474c91dd8bc99180bd8b60008284801015dc8ff7be227e03e6a165288779873802de3fc6f9e9d2f1327bbac5622f26e6b000622012001180119220120011a011b28480101fab76b54c877a92184fc381a235a4d1f73f8f5634234eb86a19dc4ea0fd311cd0003220120011c011d28480101e36b97cbefa2ea343cb1b27db436d2b96b1ef9630e89c2f55262b5133a8e3dfe000428480101a4c4ad6f8878cddf7d92317a81d01632dc08f5d80c2b22e0811fed9096e045f80001220120011e011f020120012001212848010127a1aaf44ac169a333b9021664dfe795fe629ec8c4a519330675ffc3e3c95df2000200b0bcb7627cf63a9863ad66e318ab87b4480ea6a87072c533b705ea8b04e941417469b14ef5000000000000012000000009abbed4d1000000b4b102e10c69b14cc1000000000000006a0000000743e53200000000464430ae5d00b0bc8c3e072372b7cbaf412023c778c2f32d8f23d5cf35fb3bd1b7974b8e37e29469b196d7000000000000004a0000000832abe20700000040626da18d69b1809a000000000000001a0000000663c51a69000000183fb371cf01038020012600010201018201270317cca56892d58e4443b9aca0040136013701380247a0156f062d1d4c4d92a5158757dafa119b34b2a889ba9cf1a8b6869bb1d22fac1ee00610013c013d02034040012801290397bfb333333333333333333333333333333333333333333333333333333333333333029999999999999999999999999999999999999999999999999999999999999999cf80000f6d1b24d2d004012a012b012c0297bf955555555555555555555555555555555555555555555555555555555555555502aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaad000000f6d1b24d2d0c10131013301035040012d01034040013d008272df570140b2ba6d8689c491875be2f48ff9549e555ee4fcd5e5f2f25302e8e673825077497c25deee5e32c4d907e8fcb8f8149610031fc1286ede55d66540133503af7333333333333333333333333333333333333333333333333333333333333333300003db46c934b411519f5e876db3c0e676c744cf3afc9ef69cd9f2b9f39247b7c9a1aacaee23bca00003db46c84090269b196d700014080132012e012f008272df570140b2ba6d8689c491875be2f48ff9549e555ee4fcd5e5f2f25302e8e673360fdc4cd7344b9fa5ed443f0f9597b33f67b73e32f0e5f21249a32b99f773a302052030240130014300a044667010b076000000000000000000b50000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003af7555555555555555555555555555555555555555555555555555555555555555500003db46c934b43406761a811267aa731fa5aeddac3c1ed61e08340c0294d7686e97c8f0b0bbcea00003db46c84090369b196d700014080132013301340001200082725f413dd86810be8e6718d5ae2a9fb2cf5f44e2cfb50e3eecd5d51dd965b7bd7a9f75230007799f8bc2ce251b4860747cea889507fdb2e7f7912f14ee50a7399402053030240135014300a04136f010b0760000000000000000002e000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000103d0400139003fb0000000004000000000000000224b5639110ee6b2800892d58e4443b9aca004010150013b01db501dfd03701bbe7e480001eda3642048000001eda3642048a45ee3edc8c5a6ba7f6ebb0429455ce5cd165fc3cf0880aa93afbf89bc1d202ed9894bef4e823d6bed5055a187f844cbdcaef7735d0cd7b0acea6a6ce4293f4610800061660400000000000000001bbe7e334d8cb6a2013a00134496ac72221dcd650020020161013c013d0106460600014103af7333333333333333333333333333333333333333333333333333333333333333300003db46c934b4277455615b2e70aa055745a3bc090871d9b12a07fa3b4fb55c6683905c6e5187600003db46c934b4169b196d70001408013e013f01400101a00141008272360fdc4cd7344b9fa5ed443f0f9597b33f67b73e32f0e5f21249a32b99f773a3825077497c25deee5e32c4d907e8fcb8f8149610031fc1286ede55d665401335020f040929f5ae6458110142014300ab69fe00000000000000000000000000000000000000000000000000000000000000013fccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccd29f5ae6440000007b68d9269680d3632dae4000a042af7010b0760000000000000000006400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005bc00000000000000000000000012d452da449e50b8cf7dd27861f146122afe1b546bb8b70fc8216f0c614139f8e048852b82a";
        let boc = BagOfCells::parse_hex(hex_data).unwrap();

        assert_eq!(boc.roots.len(), 1);
    }

    #[test]
    fn test_block_header_boc() {
        let hex_data = "b5ee9c720102070100014700094603ef0a1e4e8f974a891d074588cc97e9cbccd802850a269e940625cb3dc095c275001601241011ef55aaffffff110203040501a09bc7a9870000000004010377d36a0000000100ffffffff000000000000000069b19f6f00003db4a44e430000003db4a44e43044e74e93c000c25100377d3670377b158c40000000d00000000000003ee06284801016628453b781f46d532de8328d5cfd759901026e623ce0dfdd6a9c5366d0acdb2000328480101374c44751598b26bc68da35458f2c4b1b2e583ce11fca6536a3818700659c7a500152848010160c62256a46f2119dace4a876d3107fecc66ca742c71125a6a1590ee6ddb706e0007009800003db4a43f00c40377d369b20655a8aff399b4497bbe6324f72afa8c489f1d1aeab77a6c8144d8fba199ab3ebb28da6dbe4913cd67d60b2cb989c4b2e6afb79ca4ddaf5413da2574a63450";
        let boc = BagOfCells::parse_hex(hex_data).unwrap();

        assert_eq!(boc.roots.len(), 1);
    }
}
