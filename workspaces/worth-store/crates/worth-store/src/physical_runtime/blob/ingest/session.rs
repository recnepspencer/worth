use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    BlobGenerationPublicationV1, BlobRecordDenial, BlobRecordKind, BlobTreeEntryV1,
    IndexedThroughBlobPublication,
};

use crate::physical_runtime::{
    durability::PhysicalBlobSessionClaim, layout::PhysicalLayoutMaintenanceFailure,
    AdmittedRecordPlacementPolicy, PhysicalReadProtectionDenial, PhysicalRecordReader,
    ServingPhysicalRuntime,
};

use super::super::{
    append::{append_blob_record, append_blob_record_with_root},
    read::BlobReadOpenFailure,
    tree::{BlobTreeBuildFailure, BlobTreeBuilder},
    BlobAppendFailure, BlobDedupeFailure, BlobGeneration, BlobIngestAllocation,
    BlobIngestDeclaration, BlobMemoryDenial, BlobMemoryObservation, BlobResidentComponent,
    BlobSessionId, PublishedBlobGeneration,
};
use super::append_pressure::BlobAppendPressure;
use super::resume::{BlobResumeObservation, RetainedBlobNodes};
use super::{content::BlobIngestContent, node_writer::BlobNodeWriter};
use super::{frontier::BlobIngestProgress, BlobIngestClaimDenial};

#[path = "session/chunk_persistence.rs"]
mod chunk_persistence;
#[path = "session/declaration_admission.rs"]
mod declaration_admission;
#[path = "session/terminal_head_non_reissue.rs"]
mod terminal_head_non_reissue;
pub(in crate::physical_runtime) use terminal_head_non_reissue::{
    TerminalHeadIdentityNonReissue, TerminalHeadNonReissueDenial,
};

#[derive(Debug)]
pub enum BlobIngestFailure {
    Claim(BlobIngestClaimDenial),
    EntropyUnavailable,
    IdentityInspection(BlobReadOpenFailure),
    ForeignObject,
    ObjectAlreadyDeclared,
    CheckpointHorizonExhausted,
    Format(BlobRecordDenial),
    Dedupe(BlobDedupeFailure),
    Memory(BlobMemoryDenial),
    PrePublicationRootProtection(PhysicalReadProtectionDenial),
    Append {
        session: BlobSessionId,
        kind: BlobRecordKind,
        ordinal: u64,
        cause: BlobAppendFailure,
    },
    TooManyBytes,
    SourceFrameExceedsWindow {
        supplied: u64,
        admitted_window: u64,
    },
    Incomplete {
        received: u64,
        declared: u64,
    },
    TreeTooDeep,
    TreeConflict,
    /// The authoritative generation is durable, but its derived catalog is
    /// not selected. Callers must not retry this as an effect-free ingest.
    PublishedIndexPending {
        published: PublishedBlobGeneration,
        cause: PhysicalLayoutMaintenanceFailure,
    },
    Poisoned,
    #[cfg(feature = "certification-test-authority")]
    CertificationQuarantineGap,
}

pub struct BlobIngestSession<'runtime> {
    pub(super) runtime: &'runtime ServingPhysicalRuntime,
    pub(super) placement: AdmittedRecordPlacementPolicy,
    pub(super) declaration: BlobIngestContent,
    pub(super) session: BlobSessionId,
    pub(super) allocation: BlobIngestAllocation<'runtime>,
    pub(super) progress: BlobIngestProgress,
    pub(super) _claimed_reader: PhysicalRecordReader,
    pub(super) _live_claim: PhysicalBlobSessionClaim,
    pub(super) pending: Vec<u8>,
    pub(super) received: u64,
    pub(super) chunk_ordinal: u64,
    pub(super) logical: Sha256,
    pub(super) tree: BlobTreeBuilder,
    pub(super) retained_nodes: RetainedBlobNodes,
    pub(super) resume_observation: Option<BlobResumeObservation>,
    pub(super) poisoned: bool,
    #[cfg(feature = "certification-test-authority")]
    pub(super) forced_dedupe_digest: Option<[u8; 32]>,
    #[cfg(feature = "certification-test-authority")]
    pub(super) fail_before_quarantine: bool,
}

impl BlobIngestSession<'_> {
    /// Certification-only, one-chunk lookup-key fault injection. It does not
    /// change the canonical digest written into an original chunk frame.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_force_next_dedupe_lookup_digest(&mut self, digest: [u8; 32]) -> bool {
        if digest == [0; 32] || self.poisoned {
            return false;
        }
        self.forced_dedupe_digest = Some(digest);
        true
    }

    /// Stops once the conflicting original chunk is selected, before the
    /// quarantine append can advance the selected C.5 marker.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_fail_before_dedupe_quarantine(&mut self) {
        self.fail_before_quarantine = true;
    }
    pub const fn session_id(&self) -> BlobSessionId {
        self.session
    }

    pub fn memory_observation(&self) -> BlobMemoryObservation {
        self.allocation.observation()
    }

    pub const fn resume_observation(&self) -> Option<BlobResumeObservation> {
        self.resume_observation
    }

    /// Accepts at most the declared byte count, retaining one partial chunk.
    pub fn push(&mut self, bytes: &[u8]) -> Result<(), BlobIngestFailure> {
        if self.poisoned {
            return Err(BlobIngestFailure::Poisoned);
        }
        // The caller retains this frame for the duration of the push. Charge
        // only frames that fit the admitted window, before any chunk effect.
        let supplied = bytes.len() as u64;
        let admitted_window = self.allocation.source_window();
        if supplied > admitted_window {
            return Err(BlobIngestFailure::SourceFrameExceedsWindow {
                supplied,
                admitted_window,
            });
        }
        if supplied > self.declaration.total_bytes() - self.received {
            return Err(BlobIngestFailure::TooManyBytes);
        }
        let chunk_size = self.declaration.chunk_size().bytes() as usize;
        let mut rest = bytes;
        while !rest.is_empty() {
            let count = (chunk_size - self.pending.len()).min(rest.len());
            let (part, tail) = rest.split_at(count);
            self.pending.extend_from_slice(part);
            self.logical.update(part);
            self.received += count as u64;
            self.allocation
                .set_live(
                    BlobResidentComponent::PendingChunk,
                    self.pending.len() as u64,
                )
                .map_err(BlobIngestFailure::Memory)?;
            rest = tail;
            if self.pending.len() == chunk_size {
                if let Err(error) = self.flush_chunk() {
                    self.poisoned = true;
                    return Err(error);
                }
            }
        }
        Ok(())
    }

    pub fn finish(mut self) -> Result<PublishedBlobGeneration, BlobIngestFailure> {
        if self.poisoned {
            return Err(BlobIngestFailure::Poisoned);
        }
        if self.received != self.declaration.total_bytes() {
            return Err(BlobIngestFailure::Incomplete {
                received: self.received,
                declared: self.declaration.total_bytes(),
            });
        }
        if !self.pending.is_empty() {
            self.flush_chunk()?;
        }
        let session = self.session;
        let runtime = self.runtime;
        let placement = self.placement;
        let deadline = self.declaration.deadline();
        let mut writer = BlobNodeWriter {
            runtime,
            placement,
            session,
            deadline,
            allocation: &mut self.allocation,
            retained: &mut self.retained_nodes,
        };
        let root = self
            .tree
            .finish(&mut |node, ordinal| writer.write(node, ordinal))
            .map_err(map_tree_failure)?;
        if !self.retained_nodes.is_empty() {
            return Err(BlobIngestFailure::TreeConflict);
        }
        let logical_digest: [u8; 32] = self.logical.finalize().into();
        let generation = BlobGeneration::published(1);
        let publication = BlobGenerationPublicationV1::new(
            runtime.store_identity().bytes(),
            session.bytes(),
            self.declaration.object().bytes(),
            generation.sequence(),
            root.record,
            root.frame_digest,
            root.covered_bytes,
            logical_digest,
            u32::try_from(self.declaration.chunk_size().bytes()).expect("admitted size fits u32"),
            self.declaration.scope_fingerprint(),
        )
        .map_err(BlobIngestFailure::Format)?;
        let encoded = publication.encode();
        let _pressure = BlobAppendPressure::admit(&mut self.allocation, encoded.len() as u64)
            .map_err(BlobIngestFailure::Memory)?;
        // The selected latest-publication marker is singular. Keep another
        // generation from overtaking this one before its derived directory
        // advances, including when different sessions finish concurrently.
        let _index_publication = runtime.lock_blob_index_publication();
        let previous_source = runtime
            .records()
            .map_err(BlobIngestFailure::PrePublicationRootProtection)?
            .selected_latest_blob_publication();
        let publication_digest: [u8; 32] = Sha256::digest(&encoded).into();
        let completed = append_blob_record_with_root(
            runtime,
            placement,
            session,
            BlobRecordKind::GenerationPublished,
            generation.sequence(),
            deadline,
            encoded,
        )
        .map_err(|cause| BlobIngestFailure::Append {
            session,
            kind: BlobRecordKind::GenerationPublished,
            ordinal: generation.sequence(),
            cause,
        })?;
        drop(_pressure);
        let published = PublishedBlobGeneration::from_completed_publication(
            runtime.store_identity(),
            session,
            self.declaration.object(),
            generation,
        );
        let source = IndexedThroughBlobPublication::new(
            completed.root_generation,
            completed.record,
            publication_digest,
        )
        .expect("completed physical root generation is nonzero");
        runtime
            .maintain_blob_catalog_publication(
                source,
                previous_source,
                self.declaration.object().bytes(),
                generation.sequence(),
                placement,
                deadline,
            )
            .map_err(|cause| BlobIngestFailure::PublishedIndexPending { published, cause })?;
        Ok(published)
    }

    fn flush_chunk(&mut self) -> Result<(), BlobIngestFailure> {
        let ordinal = self.chunk_ordinal;
        let (record, digest) = self.append_selected_chunk(ordinal)?;
        self.progress
            .record_chunk(record, digest, self.pending.len() as u64);
        let entry = BlobTreeEntryV1::new(digest, record, self.pending.len() as u64)
            .map_err(BlobIngestFailure::Format)?;
        let runtime = self.runtime;
        let placement = self.placement;
        let session = self.session;
        let deadline = self.declaration.deadline();
        let mut writer = BlobNodeWriter {
            runtime,
            placement,
            session,
            deadline,
            allocation: &mut self.allocation,
            retained: &mut self.retained_nodes,
        };
        self.tree
            .push(entry, &mut |node, node_ordinal| {
                writer.write(node, node_ordinal)
            })
            .map_err(map_tree_failure)?;
        self.pending.clear();
        self.allocation
            .set_live(BlobResidentComponent::PendingChunk, 0)
            .map_err(BlobIngestFailure::Memory)?;
        self.chunk_ordinal = ordinal
            .checked_add(1)
            .ok_or(BlobIngestFailure::TooManyBytes)?;
        self.checkpoint_if_due()
    }
}

fn map_tree_failure(failure: BlobTreeBuildFailure<BlobIngestFailure>) -> BlobIngestFailure {
    match failure {
        BlobTreeBuildFailure::Format(denial) => BlobIngestFailure::Format(denial),
        BlobTreeBuildFailure::Write(failure) => failure,
        BlobTreeBuildFailure::TooDeep => BlobIngestFailure::TreeTooDeep,
    }
}
