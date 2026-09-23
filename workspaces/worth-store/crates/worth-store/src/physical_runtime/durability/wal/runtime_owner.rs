use std::sync::{Arc, Mutex};

use worth_store_physical_backend::{ArtifactTreeFile, QualifiedFilesystemMedia};
use worth_store_wal::{LogSequenceNumber, WalAppendFrontier};

use crate::physical_runtime::durability::retention::PhysicalPublicationAdmission;
use crate::physical_runtime::record_serving::PreparedPhysicalMutation;
use crate::physical_runtime::{PhysicalSignalProfileIdentity, RuntimeIdentity};

use super::preparation_admission::{AdmittedWalPreparedMutation, PhysicalWalPreparationAdmission};
use super::{
    inventory::{PhysicalWalSegmentInventory, ReopenedPhysicalWalInventory},
    PhysicalWalAppendDeclaration, PhysicalWalReservationDenial,
};

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalWalRuntimeOwner {
    pub(super) shared: Arc<Mutex<PhysicalWalRuntimeState>>,
    pub(super) preparation: Arc<PhysicalWalPreparationAdmission>,
    pub(super) publication: Arc<Mutex<Option<Arc<PhysicalPublicationAdmission>>>>,
}

struct PlannedMaintenanceFrame {
    bytes: Vec<u8>,
    frontier: WalAppendFrontier,
    segment: worth_store_wal::WalSegmentId,
    generation: worth_store_wal::WalSegmentGeneration,
    lsn_range: worth_store_wal::WalLsnRange,
    retirement: Option<(u8, u64, u64)>,
}

pub(super) struct PhysicalWalRuntimeState {
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
    pub(super) reopened_publications: u64,
    pub(super) reopened_bytes: u64,
    pub(super) reopen_peak_buffer_bytes: u64,
    pub(super) segments: PhysicalWalSegmentInventory,
    maintenance: Option<PlannedMaintenanceFrame>,
    awaiting_barrier: bool,
    pub(super) unresolved_retirement_spans: Vec<(u64, u64, u64, u64)>,
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
                reopened_publications: inventory.publication_frames,
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

    pub(in crate::physical_runtime) fn plan_maintenance_frame(
        &self,
        payload: &[u8],
    ) -> Result<(ArtifactTreeFile, u64, u64, u64, Vec<u8>), ()> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if state.sealed || state.in_flight {
            return Err(());
        }
        let start = state
            .frontier
            .last_lsn_end()
            .unwrap_or(LogSequenceNumber::new(LogSequenceNumber::GENESIS.get() + 1));
        let end = LogSequenceNumber::new(start.get().checked_add(1).ok_or(())?);
        let range = worth_store_wal::WalLsnRange::new(start, end).map_err(|_| ())?;
        let planned = worth_store_wal::plan_wal_frame_append(
            state.frontier,
            range,
            "store.physical.retirement.v1",
            payload,
        )
        .map_err(|_| ())?;
        let byte_limit = state.policy.segment_byte_limit().get().get();
        if planned.resulting_frontier().valid_prefix_bytes() > byte_limit {
            return Err(());
        }
        let bytes = planned.frame().encoded_frame().to_vec();
        let offset = state.frontier.valid_prefix_bytes();
        let segment = state.frontier.segment().get();
        let generation = state.frontier.generation().get();
        state.in_flight = true;
        let retirement = super::super::retention::decode_retirement(payload)
            .map(|record| (record.action, record.segment_id, record.generation));
        state.maintenance = Some(PlannedMaintenanceFrame {
            bytes: bytes.clone(),
            frontier: planned.resulting_frontier(),
            segment: state.frontier.segment(),
            generation: state.frontier.generation(),
            lsn_range: range,
            retirement,
        });
        Ok((
            state.active_artifact.clone(),
            segment,
            generation,
            offset,
            bytes,
        ))
    }

    pub(in crate::physical_runtime) fn planned_maintenance_interval(
        &self,
    ) -> Option<(u64, u64, u64, u64, u64, u64)> {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let planned = state.maintenance.as_ref()?;
        Some((
            planned.segment.get(),
            planned.generation.get(),
            planned.lsn_range.start().get(),
            planned.lsn_range.end_exclusive().get(),
            state.frontier.valid_prefix_bytes(),
            planned.bytes.len() as u64,
        ))
    }

    pub(in crate::physical_runtime) fn note_maintenance_written(&self) {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .awaiting_barrier = true;
    }

    pub(in crate::physical_runtime) fn maintenance_awaiting_barrier(&self) -> bool {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .awaiting_barrier
    }

    pub(in crate::physical_runtime) fn planned_maintenance_artifact(
        &self,
    ) -> Option<ArtifactTreeFile> {
        let state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state
            .maintenance
            .as_ref()
            .map(|_| state.active_artifact.clone())
    }

    pub(in crate::physical_runtime) fn abort_maintenance_frame(&self) {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.maintenance = None;
        state.in_flight = false;
        state.awaiting_barrier = false;
    }

    pub(in crate::physical_runtime) fn finish_maintenance_frame(&self) -> Result<(), ()> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        state.awaiting_barrier = false;
        let planned = state.maintenance.take().ok_or(())?;
        let identity =
            worth_store_wal::WalSegmentArtifactIdentity::new(planned.segment, planned.generation);
        if state
            .segments
            .record_completed_append(identity, planned.lsn_range, planned.bytes.len() as u64)
            .is_err()
        {
            state.sealed = true;
            state.in_flight = false;
            return Err(());
        }
        state.frontier = planned.frontier;
        state.appended_frames = state.appended_frames.saturating_add(1);
        state.appended_bytes = state
            .appended_bytes
            .saturating_add(planned.bytes.len() as u64);
        state.in_flight = false;
        let start = planned.lsn_range.start().get();
        let end = planned.lsn_range.end_exclusive().get();
        super::super::retention::note_retirement_hold(
            &mut state.unresolved_retirement_spans,
            planned.retirement,
            start,
            end,
        );
        if state.record_durable_barrier(start, end) {
            Ok(())
        } else {
            Err(())
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
    ) -> Result<(), PhysicalWalMemberCompletionDenial> {
        let mut state = self
            .shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let identity = worth_store_wal::WalSegmentArtifactIdentity::new(
            declaration.segment(),
            declaration.generation(),
        );
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
    pub(in crate::physical_runtime) fn reopened_publications(&self) -> u64 {
        self.shared
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reopened_publications
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
#[path = "runtime_owner/retirement_hold.rs"]
mod retirement_hold;

#[cfg(test)]
mod tests;
