use std::sync::{Arc, Mutex};

use worth_store_physical_backend::{ArtifactTreeFile, QualifiedFilesystemMedia};
use worth_store_wal::{LogSequenceNumber, WalAppendFrontier};

use crate::physical_runtime::durability::retention::PhysicalPublicationAdmission;
use crate::physical_runtime::record_serving::PreparedPhysicalMutation;
use crate::physical_runtime::{PhysicalSignalProfileIdentity, RuntimeIdentity};

use super::preparation_admission::{AdmittedWalPreparedMutation, PhysicalWalPreparationAdmission};
use super::{
    inventory::{
        PhysicalWalSegmentInventory, ReopenedPhysicalWalInventory, ReopenedWalPublicationGroup,
    },
    PhysicalWalAppendDeclaration, PhysicalWalReservationDenial,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalWalRuntimeOwner {
    pub(super) shared: Arc<Mutex<PhysicalWalRuntimeState>>,
    pub(super) preparation: Arc<PhysicalWalPreparationAdmission>,
    pub(super) publication: Arc<Mutex<Option<Arc<PhysicalPublicationAdmission>>>>,
}

struct PlannedMaintenanceFrame {
    payload_digest: [u8; 32],
    frame_identity_digest: [u8; 32],
    frame_payload_digest: [u8; 32],
    bytes: Vec<u8>,
    frontier: WalAppendFrontier,
    segment: worth_store_wal::WalSegmentId,
    generation: worth_store_wal::WalSegmentGeneration,
    lsn_range: worth_store_wal::WalLsnRange,
    retirement: Option<(crate::physical_runtime::durability::RetiredArtifact, bool)>,
    extent_copy: Option<worth_store_physical_format::PhysicalExtentCopyRecord>,
}

pub(super) struct PhysicalWalRuntimeState {
    pub(super) record_format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
    pub(super) copy_obligations: Vec<super::RetainedExtentCopyObligation>,
    pub(super) reopened_checkpoint_cutoff: u64,
    pub(super) frontier: WalAppendFrontier,
    pub(super) durable_lsn_end: Option<LogSequenceNumber>,
    pub(super) active_artifact: ArtifactTreeFile,
    pub(super) policy: crate::physical_runtime::PhysicalWalPolicy,
    pub(super) segment_count: u32,
    pub(super) in_flight: bool,
    pub(super) sealed: bool,
    pub(super) appended_frames: u64,
    pub(super) appended_bytes: u64,
    pub(super) rotations: u64,
    pub(super) reclaimed_segments: u64,
    pub(super) reclaimed_bytes: u64,
    pub(super) reopened_frames: u64,
    pub(super) reopened_publication_groups: Vec<ReopenedWalPublicationGroup>,
    pub(super) reopened_release_metadata: Vec<(u64, u64, u64, u64)>,
    pub(super) retained_maintenance: Vec<super::inventory::RetainedMaintenanceIntent>,
    pub(super) reopened_bytes: u64,
    pub(super) reopen_peak_buffer_bytes: u64,
    pub(super) segments: PhysicalWalSegmentInventory,
    maintenance: Option<PlannedMaintenanceFrame>,
    awaiting_barrier: bool,
    pub(super) unresolved_retirement_spans: Vec<(
        crate::physical_runtime::durability::RetiredArtifact,
        u64,
        u64,
    )>,
}

pub(super) enum PhysicalWalMemberCompletionDenial {
    Inventory,
    Idempotency,
}

impl PhysicalWalRuntimeOwner {
    pub(in crate::physical_runtime) fn from_reopened(
        media: &QualifiedFilesystemMedia,
        runtime: RuntimeIdentity,
        signal_profile: PhysicalSignalProfileIdentity,
        policy: crate::physical_runtime::PhysicalWalPolicy,
        inventory: ReopenedPhysicalWalInventory,
    ) -> Self {
        Self {
            shared: Arc::new(Mutex::new(PhysicalWalRuntimeState {
                record_format: inventory.record_format,
                copy_obligations: inventory.copy_obligations,
                reopened_checkpoint_cutoff: inventory.checkpoint_cutoff,
                frontier: inventory.frontier,
                durable_lsn_end: inventory.frontier.last_lsn_end(),
                active_artifact: inventory.active_artifact,
                policy,
                segment_count: inventory.segment_count,
                in_flight: false,
                sealed: inventory.requires_inspection,
                appended_frames: 0,
                appended_bytes: 0,
                rotations: 0,
                reclaimed_segments: 0,
                reclaimed_bytes: 0,
                reopened_frames: inventory.frame_count,
                reopened_publication_groups: inventory.publication_groups,
                reopened_release_metadata: inventory.release_metadata,
                retained_maintenance: inventory.retained_maintenance,
                reopened_bytes: inventory.byte_count,
                reopen_peak_buffer_bytes: inventory.peak_buffer_bytes,
                segments: inventory.segments,
                maintenance: None,
                awaiting_barrier: false,
                unresolved_retirement_spans: Vec::new(),
            })),
            preparation: Arc::new(PhysicalWalPreparationAdmission::new(
                media.store_identity(),
                runtime,
                signal_profile,
            )),
            publication: Arc::new(Mutex::new(None)),
        }
    }

    pub(super) fn admit_preparation(
        &self,
        prepared: PreparedPhysicalMutation,
    ) -> Result<AdmittedWalPreparedMutation, (PreparedPhysicalMutation, PhysicalWalReservationDenial)>
    {
        self.preparation.admit(prepared)
    }

    pub(super) fn complete_member(
        &self,
        frontier: WalAppendFrontier,
        artifact: ArtifactTreeFile,
        declaration: PhysicalWalAppendDeclaration,
        bytes: u64,
        idempotency: &crate::physical_runtime::durability::PhysicalMutationIdempotencyRuntimeAuthority,
        persisted: crate::physical_runtime::durability::PersistedPhysicalMutationAttemptBinding,
        canonical_redo: &[u8],
    ) -> Result<(), PhysicalWalMemberCompletionDenial> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let identity = worth_store_wal::WalSegmentArtifactIdentity::new(
            declaration.segment(),
            declaration.generation(),
        );
        let format = state.record_format;
        if super::inventory::observe_bound_copy_publication(
            canonical_redo,
            &persisted,
            format,
            &mut state.copy_obligations,
            0,
        )
        .is_err()
        {
            state.sealed = true;
            return Err(PhysicalWalMemberCompletionDenial::Inventory);
        }
        if let Err(_denial) =
            state
                .segments
                .record_completed_append(identity, declaration.lsn_range(), bytes)
        {
            state.sealed = true;
            return Err(PhysicalWalMemberCompletionDenial::Inventory);
        }
        if let Err(_denial) = idempotency.record_wal_binding(persisted) {
            state.sealed = true;
            return Err(PhysicalWalMemberCompletionDenial::Idempotency);
        }
        if state.segment_count == 0 {
            state.segment_count = 1;
        } else if state.frontier.segment() != frontier.segment() {
            state.segment_count = state.segment_count.saturating_add(1);
            state.rotations = state.rotations.saturating_add(1);
        }
        state.frontier = frontier;
        state.active_artifact = artifact;
        state.appended_frames = state.appended_frames.saturating_add(1);
        state.appended_bytes = state.appended_bytes.saturating_add(bytes);
        Ok(())
    }

    pub(in crate::physical_runtime) fn finish_group(&self) {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .in_flight = false;
    }

    pub(in crate::physical_runtime) fn record_durable_barrier(
        &self,
        lsn_start: u64,
        lsn_end_exclusive: u64,
    ) -> bool {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.record_durable_barrier(lsn_start, lsn_end_exclusive)
    }

    pub(in crate::physical_runtime) fn checkpoint_source_range(
        &self,
    ) -> Option<worth_store_physical_format::CheckpointWalSourceRange> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .checkpoint_source_range()
    }

    pub(in crate::physical_runtime) fn release_group_before_effect(&self) {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .in_flight = false;
    }

    pub(in crate::physical_runtime) fn seal_for_inspection(&self) {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.in_flight = false;
        state.sealed = true;
    }

    /// Publications whose WAL frames survived reopen and so remain charged.
    pub(in crate::physical_runtime) fn take_reopened_release_metadata(
        &self,
    ) -> Vec<(u64, u64, u64, u64)> {
        let mut state = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        std::mem::take(&mut state.reopened_release_metadata)
    }

    pub(in crate::physical_runtime) fn recovered_copy_obligations(
        &self,
    ) -> Vec<super::RetainedExtentCopyObligation> {
        let state = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        state
            .copy_obligations
            .iter()
            .copied()
            .filter(|entry| entry.holds_at(state.reopened_checkpoint_cutoff))
            .collect()
    }

    pub(in crate::physical_runtime) fn take_reopened_publication_groups(
        &self,
    ) -> Vec<ReopenedWalPublicationGroup> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::mem::take(&mut state.reopened_publication_groups)
    }

    pub(in crate::physical_runtime) fn reopened_wal_segments(&self) -> Vec<((u64, u64), u64)> {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .segments
            .entries()
            .iter()
            .map(|entry| {
                (
                    (
                        entry.identity().segment().get(),
                        entry.identity().generation().get(),
                    ),
                    entry.byte_count(),
                )
            })
            .collect()
    }

    pub(in crate::physical_runtime) fn observation(&self) -> super::PhysicalWalObservation {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        super::PhysicalWalObservation::new(
            state.frontier.segment().get(),
            state.frontier.generation().get(),
            state.appended_frames,
            state.appended_bytes,
            state.frontier.valid_prefix_bytes(),
            state.frontier.last_lsn_end().map(LogSequenceNumber::get),
            state.segment_count,
            state.reopened_frames,
            state.reopened_bytes,
            state.reopen_peak_buffer_bytes,
            state.rotations,
            state.reclaimed_segments,
            state.reclaimed_bytes,
            state.sealed,
        )
    }
}

#[path = "runtime_owner/durable_barrier.rs"]
mod durable_barrier;
#[path = "runtime_owner/maintenance.rs"]
mod maintenance;
#[path = "runtime_owner/retirement_hold.rs"]
mod retirement_hold;

#[cfg(test)]
mod tests;
