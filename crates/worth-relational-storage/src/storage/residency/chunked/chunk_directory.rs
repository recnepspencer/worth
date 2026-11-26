//! A region's validated chunk directory.
//!
//! Admission proves that each slot domain's chunks are exactly its fixed
//! partition: every slot of `[0, entity_slots)` and `[0, relation_slots)` lies
//! in one chunk, and no two chunks overlap. Only an admitted directory lets a
//! reader trust a negative summary to skip a chunk.
use super::consolidation::ConsolidatedBinding;
use super::layout::ChunkLayout;
use super::locator_binding::ChunkLocatorBinding;
use super::semantic_entry::{root_semantic_merkle, ChunkSemanticEntry};
use super::slot_range::{fixed_partition, SlotDomain, SlotRange};

/// One chunk's semantic entry and the binding of its members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundChunk {
    semantic: ChunkSemanticEntry,
    binding: ChunkLocatorBinding,
}

impl BoundChunk {
    pub fn semantic(&self) -> &ChunkSemanticEntry {
        &self.semantic
    }

    pub fn binding(&self) -> &ChunkLocatorBinding {
        &self.binding
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkCoverageRefusal {
    /// The layout's chunk geometry is invalid (e.g. `chunk_slots == 0`).
    InvalidLayout { chunk_slots: u64 },
    /// A chunk's binding names another range than its semantic entry.
    BindingMismatch { slot_range: SlotRange },
    /// The domain's chunks differ from its fixed partition: a gap, an
    /// overlap, a missing tail, a misplaced boundary or a foreign chunk.
    PartitionMismatch {
        domain: SlotDomain,
        expected: Option<SlotRange>,
        found: Option<SlotRange>,
    },
    /// A consolidated binding was verified against no chunk of this directory.
    UnknownChunk {
        domain: SlotDomain,
        slot_range: SlotRange,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegionChunkDirectory {
    entity_slots: u64,
    relation_slots: u64,
    /// Entity chunks ascending, then relation chunks ascending.
    chunks: Vec<BoundChunk>,
}

impl RegionChunkDirectory {
    /// Admits sealed chunks, given in entity-then-relation partition order.
    pub fn admit(
        layout: ChunkLayout,
        entity_slots: u64,
        relation_slots: u64,
        sealed: impl IntoIterator<Item = (ChunkSemanticEntry, ChunkLocatorBinding)>,
    ) -> Result<Self, ChunkCoverageRefusal> {
        if layout.chunk_slots == 0 {
            return Err(ChunkCoverageRefusal::InvalidLayout {
                chunk_slots: layout.chunk_slots,
            });
        }
        let chunks = sealed
            .into_iter()
            .map(|(semantic, binding)| {
                if semantic.slot_range() == binding.slot_range() {
                    Ok(BoundChunk { semantic, binding })
                } else {
                    Err(ChunkCoverageRefusal::BindingMismatch {
                        slot_range: binding.slot_range(),
                    })
                }
            })
            .collect::<Result<Vec<_>, _>>()?;
        let expected = fixed_partition(entity_slots, layout.chunk_slots)
            .map(|range| (SlotDomain::Entity, range))
            .chain(
                fixed_partition(relation_slots, layout.chunk_slots)
                    .map(|range| (SlotDomain::Relation, range)),
            );
        let mut found = chunks
            .iter()
            .map(|chunk| (chunk.semantic.domain(), chunk.semantic.slot_range()));
        let mut expected = expected;
        loop {
            match (expected.next(), found.next()) {
                (None, None) => break,
                (Some(expected), Some(found)) if expected == found => continue,
                (expected, found) => {
                    let domain = expected.or(found).map_or(SlotDomain::Entity, |(d, _)| d);
                    return Err(ChunkCoverageRefusal::PartitionMismatch {
                        domain,
                        expected: expected.map(|(_, range)| range),
                        found: found.map(|(_, range)| range),
                    });
                }
            }
        }
        Ok(Self {
            entity_slots,
            relation_slots,
            chunks,
        })
    }

    pub fn entity_slots(&self) -> u64 {
        self.entity_slots
    }

    pub fn relation_slots(&self) -> u64 {
        self.relation_slots
    }

    pub fn chunks(&self) -> &[BoundChunk] {
        &self.chunks
    }

    /// Chunks of one domain, ascending.
    pub fn domain_chunks(&self, domain: SlotDomain) -> impl Iterator<Item = &BoundChunk> {
        self.chunks
            .iter()
            .filter(move |chunk| chunk.semantic.domain() == domain)
    }

    /// The one chunk covering `slot` of `domain`.
    pub fn chunk_of(&self, domain: SlotDomain, slot: u64) -> Option<&BoundChunk> {
        self.domain_chunks(domain)
            .find(|chunk| chunk.semantic.slot_range().contains(slot))
    }

    /// Merkle root over the ordered semantic entries. Bindings never enter it.
    pub fn root_semantic_merkle(&self) -> [u8; 32] {
        let entries = self
            .chunks
            .iter()
            .map(|chunk| chunk.semantic.clone())
            .collect::<Vec<_>>();
        root_semantic_merkle(&entries)
    }

    /// Installs a binding the consolidator proved meaning-preserving.
    pub fn install_consolidated(
        &mut self,
        consolidated: ConsolidatedBinding,
    ) -> Result<(), ChunkCoverageRefusal> {
        let (domain, verified_commitment, binding) = consolidated.into_parts();
        let chunk = self
            .chunks
            .iter_mut()
            .find(|chunk| {
                chunk.semantic.domain() == domain
                    && chunk.semantic.slot_range() == binding.slot_range()
                    && chunk.semantic.semantic_chunk_commitment() == verified_commitment
            })
            .ok_or(ChunkCoverageRefusal::UnknownChunk {
                domain,
                slot_range: binding.slot_range(),
            })?;
        chunk.binding = binding;
        Ok(())
    }
}
