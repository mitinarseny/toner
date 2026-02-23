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
        NBits, VarNBytes,
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

/// [`BitPackWithArgs::Args`] for [`BagOfCells`]
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
                let (has_idx, has_crc32c) = buffered.unpack(((), ()))?;
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
        if size_bytes > 8 {
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
        let _is_exotic: bool = (refs_descriptor >> 3) & 0b1 == 1;
        let ref_num: usize = refs_descriptor as usize & 0b111;

        let bits_descriptor: u8 = reader.unpack(())?;
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
        let level: u8 = 0;
        let is_exotic: u8 = 0;
        let refs_descriptor: u8 = self.references.len() as u8 + is_exotic * 8 + level * 32;
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
