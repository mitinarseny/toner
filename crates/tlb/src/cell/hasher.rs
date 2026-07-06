use digest::{Digest, Output};
use std::iter::once;

use crate::{Cell, cell::level_mask::LevelMask};

#[derive(Debug, Clone)]
pub struct CellHasher<D> {
    _marker: core::marker::PhantomData<D>,
}

impl<D> CellHasher<D>
where
    D: Digest,
    Output<D>: Into<[u8; 32]>,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            _marker: core::marker::PhantomData,
        }
    }

    #[inline]
    pub fn level_hash(&self, cell: &Cell, level: u8) -> (u16, [u8; 32]) {
        let hashes = self.compute_root(cell);
        let l = hashes.get(level);
        (l.depth, l.hash)
    }

    #[inline]
    pub fn repr_hash(&self, cell: &Cell) -> [u8; 32] {
        self.compute_root(cell).get(0).hash
    }

    #[inline]
    fn compute_root(&self, cell: &Cell) -> CellHashes {
        cell.iter()
            .augmented(Self::compute)
            .last()
            .map(|(_, hashes)| hashes)
            .unwrap_or_default()
    }

    fn compute(cell: &Cell, children: &[CellHashes]) -> CellHashes {
        let kind = cell.exotic_kind();
        let is_pruned = kind.is_some_and(|k| k.is_pruned_branch());
        let merkle_offset = u8::from(kind.is_some_and(|k| k.is_merkle()));
        let mask = cell.level_mask_with(children.iter().map(|c| c.mask));
        let mut hasher = D::new();

        if is_pruned {
            let data = cell.data.as_raw_slice();
            let n = data[1].count_ones() as usize;
            let mut iter = data[2..2 + 32 * n]
                .chunks_exact(32)
                .zip(data[2 + 32 * n..2 + 32 * n + 2 * n].chunks_exact(2))
                .map(|(h, d)| LevelHash {
                    hash: h.try_into().expect("chunk is 32 bytes"),
                    depth: u16::from_be_bytes([d[0], d[1]]),
                });

            let repr = iter.next().expect("at least one level hash");
            let hash = {
                cell.write_descriptors(&mut hasher, mask);
                cell.write_data(&mut hasher);

                LevelHash {
                    hash: hasher.finalize().into(),
                    depth: 0,
                }
            };

            CellHashes {
                mask,
                repr,
                higher: iter.chain(once(hash)).collect(),
            }
        } else {
            let repr = {
                cell.write_descriptors(&mut hasher, mask.limited_by(0));
                cell.write_data(&mut hasher);
                cell.write_children(&mut hasher, merkle_offset, children);

                LevelHash {
                    hash: hasher.finalize().into(),
                    depth: max_depth(children, merkle_offset),
                }
            };
            let higher = (1..=mask.level())
                .filter(|&lvl| mask.contains(lvl))
                .scan(repr.hash, |prev, lvl| {
                    let mut hasher = D::new();
                    let child_level = lvl + merkle_offset;

                    cell.write_descriptors(&mut hasher, mask.limited_by(lvl));
                    hasher.update(&prev);
                    cell.write_children(&mut hasher, child_level, children);

                    *prev = hasher.finalize().into();
                    Some(LevelHash {
                        hash: prev.to_owned(),
                        depth: max_depth(children, child_level),
                    })
                })
                .collect();

            CellHashes { mask, repr, higher }
        }
    }
}

#[inline]
fn max_depth(children: &[CellHashes], level: u8) -> u16 {
    children
        .iter()
        .map(|c| c.get(level).depth)
        .max()
        .map(|d| d + 1)
        .unwrap_or(0)
}

impl Cell {
    #[inline]
    fn refs_descriptor(&self, applied_mask: LevelMask) -> u8 {
        self.references.len() as u8 | (u8::from(self.is_exotic) << 3) | (applied_mask.value() << 5)
    }

    #[inline]
    fn bits_descriptor(&self) -> u8 {
        let b = self.data.len();
        ((b / 8) + b.div_ceil(8)) as u8
    }

    #[inline]
    fn write_descriptors<D: Digest>(&self, d: &mut D, mask: LevelMask) {
        d.update([
            self.refs_descriptor(mask.limited_by(mask.level())),
            self.bits_descriptor(),
        ]);
    }

    #[inline]
    fn write_data<D: Digest>(&self, d: &mut D) {
        let raw = self.data.as_raw_slice();
        let rest_bits = self.data.len() % 8;
        if rest_bits == 0 {
            d.update(raw);
        } else if let Some((last, head)) = raw.split_last() {
            d.update(head);
            d.update([(last & !0u8 << (8 - rest_bits)) | (1 << (8 - rest_bits - 1))]);
        }
    }

    #[inline]
    fn write_children<D: Digest>(&self, d: &mut D, level: u8, children: &[CellHashes]) {
        for c in children {
            d.update(c.get(level).depth.to_be_bytes());
        }
        for c in children {
            d.update(c.get(level).hash);
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct LevelHash {
    hash: [u8; 32],
    depth: u16,
}

#[derive(Debug, Default, Clone)]
struct CellHashes {
    mask: LevelMask,
    repr: LevelHash,
    higher: Vec<LevelHash>,
}

impl CellHashes {
    fn get(&self, level: u8) -> &LevelHash {
        let idx = self.mask.limited_by(level).hash_index();
        if idx == 0 {
            &self.repr
        } else {
            self.higher.get(idx - 1).unwrap_or(&self.repr)
        }
    }
}
