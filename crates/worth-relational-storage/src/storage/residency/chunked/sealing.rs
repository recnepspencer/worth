//! `seal_chunk_image`: the single producer owner of a chunk's commitment.
//!
//! One pass over the producer's canonical projection validates slots, folds
//! every record, history version and adjacency edge into the semantic hasher
//! and spills oversized per-slot sequences into continuation pages. Summaries
//! are derived from that same pass; a producer claim that differs is refused
//! before the commitment is computed or any chunk image is written. A refusal
//! may leave unreferenced content-addressed continuation pages, never a
//! binding.
use super::chunk_images::{ChunkHeadImage, ChunkHistoryImage, HeadSlot, HistoryLayer, HistoryRow};
use super::chunk_summary::{ChunkKindAdjacencySummary, ClaimedChunkSummaries};
use super::continuation::spill::{ContinuationWriter, SlotSpill};
use super::layout::ChunkLayout;
use super::locator_binding::ChunkLocatorBinding;
use super::member_io::ChunkMemberSink;
use super::paged_directory::DirectoryBuildError;
use super::semantic_entry::{ChunkCommitmentHasher, ChunkSemanticEntry, DerivedChunkMeaning};
use super::slot_projection::{CanonicalChunkVisitor, SlotProjection, SlotVersion};
use super::slot_range::{SlotDomain, SlotRange};
use crate::storage::data::RelationalStorageUnavailableCause as Cause;

pub struct ChunkSealRequest<'source> {
    pub layout: ChunkLayout,
    pub domain: SlotDomain,
    /// Slots in this domain of the placement group.
    pub domain_slots: u64,
    pub slot_range: SlotRange,
    /// Occupied slots, strictly ascending.
    pub slots: Box<dyn Iterator<Item = SlotProjection<'source>> + 'source>,
    /// A summary the producer computed itself, verified against derivation.
    pub claimed: Option<ClaimedChunkSummaries>,
}

/// A sealed chunk: its semantic entry and the binding of its members.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedChunk {
    pub semantic: ChunkSemanticEntry,
    pub binding: ChunkLocatorBinding,
    pub continuation_pages: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChunkSealRefusal {
    /// The range is not a fixed partition chunk of the domain.
    NotFixedChunk {
        slot_range: SlotRange,
        domain_slots: u64,
    },
    SlotOutsideChunk {
        slot: u64,
    },
    /// Repeated or descending slot: chunk slots never overlap.
    SlotOutOfOrder {
        slot: u64,
    },
    OversizeVersion {
        slot: u64,
        bytes: usize,
        limit: usize,
    },
    /// The producer's summary differs from the derived one (e.g. an omitted
    /// kind or adjacency edge).
    UnverifiedSummary {
        claimed: Box<ClaimedChunkSummaries>,
        derived_head: Box<ChunkKindAdjacencySummary>,
        derived_history: Box<ChunkKindAdjacencySummary>,
    },
    OversizeChunk {
        length: u64,
        limit: u64,
    },
    Member(Cause),
}

impl From<Cause> for ChunkSealRefusal {
    fn from(cause: Cause) -> Self {
        match cause {
            Cause::LimitExceeded { length, limit } => Self::OversizeChunk { length, limit },
            cause => Self::Member(cause),
        }
    }
}

impl From<DirectoryBuildError> for ChunkSealRefusal {
    fn from(error: DirectoryBuildError) -> Self {
        match error {
            DirectoryBuildError::Member(cause) => cause.into(),
            DirectoryBuildError::KeyOutOfOrder | DirectoryBuildError::PageTooSmall => {
                Self::Member(Cause::Damaged)
            }
        }
    }
}

pub fn seal_chunk_image(
    request: ChunkSealRequest<'_>,
    sink: &mut dyn ChunkMemberSink,
) -> Result<SealedChunk, ChunkSealRefusal> {
    let ChunkSealRequest {
        layout,
        domain,
        domain_slots,
        slot_range,
        slots,
        claimed,
    } = request;
    if !slot_range.is_fixed_chunk(domain_slots, layout.chunk_slots) {
        return Err(ChunkSealRefusal::NotFixedChunk {
            slot_range,
            domain_slots,
        });
    }
    let mut pass = SealingPass {
        layout,
        slot_range,
        hasher: ChunkCommitmentHasher::new(domain, slot_range),
        continuation: ContinuationWriter::new(layout)?,
        head_slots: Vec::new(),
        history_rows: Vec::new(),
        previous_slot: None,
    };
    for projection in slots {
        pass.seal_slot(projection, sink)?;
    }
    let derived = pass.hasher.finish();
    verify_claimed_summaries(claimed, &derived)?;
    let semantic = derived.commit();
    let continuation_pages = pass.continuation.pages_written();
    let continuation_root = pass.continuation.finish(sink)?;
    let head = ChunkHeadImage {
        domain,
        slot_range,
        slots: pass.head_slots,
    }
    .encode(layout.max_chunk_image_bytes)?;
    let history = ChunkHistoryImage {
        domain,
        slot_range,
        layers: vec![HistoryLayer {
            rows: pass.history_rows,
        }],
    }
    .encode(layout.max_chunk_image_bytes)?;
    let binding =
        ChunkLocatorBinding::new(slot_range, head.locator, history.locator, continuation_root);
    sink.put(head.locator, head.bytes)?;
    sink.put(history.locator, history.bytes)?;
    Ok(SealedChunk {
        semantic,
        binding,
        continuation_pages,
    })
}

fn verify_claimed_summaries(
    claimed: Option<ClaimedChunkSummaries>,
    derived: &DerivedChunkMeaning,
) -> Result<(), ChunkSealRefusal> {
    match claimed {
        Some(claimed)
            if claimed.head != *derived.head_summary()
                || claimed.history != *derived.history_summary() =>
        {
            Err(ChunkSealRefusal::UnverifiedSummary {
                claimed: Box::new(claimed),
                derived_head: Box::new(derived.head_summary().clone()),
                derived_history: Box::new(derived.history_summary().clone()),
            })
        }
        _ => Ok(()),
    }
}

struct SealingPass {
    layout: ChunkLayout,
    slot_range: SlotRange,
    hasher: ChunkCommitmentHasher,
    continuation: ContinuationWriter,
    head_slots: Vec<HeadSlot>,
    history_rows: Vec<HistoryRow>,
    previous_slot: Option<u64>,
}

impl SealingPass {
    fn seal_slot(
        &mut self,
        projection: SlotProjection<'_>,
        sink: &mut dyn ChunkMemberSink,
    ) -> Result<(), ChunkSealRefusal> {
        let SlotProjection {
            slot,
            head,
            history,
            adjacency,
        } = projection;
        self.admit_slot(slot)?;
        if let Some(version) = &head {
            self.admit_version(slot, version)?;
        }
        let Ok(()) = self.hasher.slot_begin(slot, head.as_ref());
        let mut history_spill = SlotSpill::new(slot);
        for version in history {
            self.admit_version(slot, &version)?;
            let Ok(()) = self.hasher.history_version(&version);
            history_spill.push(&mut self.continuation, sink, version)?;
        }
        let versions = history_spill.finish(&mut self.continuation, sink)?;
        if !versions.is_empty() {
            self.history_rows.push(HistoryRow { slot, versions });
        }
        let mut adjacency_spill = SlotSpill::new(slot);
        for edge in adjacency {
            let Ok(()) = self.hasher.adjacency_edge(&edge);
            adjacency_spill.push(&mut self.continuation, sink, edge)?;
        }
        let adjacency = adjacency_spill.finish(&mut self.continuation, sink)?;
        let Ok(()) = self.hasher.slot_end();
        self.head_slots.push(HeadSlot {
            slot,
            head,
            adjacency,
        });
        Ok(())
    }

    fn admit_slot(&mut self, slot: u64) -> Result<(), ChunkSealRefusal> {
        if !self.slot_range.contains(slot) {
            return Err(ChunkSealRefusal::SlotOutsideChunk { slot });
        }
        if self.previous_slot.is_some_and(|previous| previous >= slot) {
            return Err(ChunkSealRefusal::SlotOutOfOrder { slot });
        }
        self.previous_slot = Some(slot);
        Ok(())
    }

    fn admit_version(&self, slot: u64, version: &SlotVersion) -> Result<(), ChunkSealRefusal> {
        let limit = self.layout.max_version_payload_bytes();
        if version.payload.len() > limit {
            return Err(ChunkSealRefusal::OversizeVersion {
                slot,
                bytes: version.payload.len(),
                limit,
            });
        }
        Ok(())
    }
}
