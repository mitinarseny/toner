use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bitvec::view::AsBits;
use tlb::{BitReader, BitUnpack, Cell, Error, StringError};

use crate::{RawBagOfCells, RawCell};

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
