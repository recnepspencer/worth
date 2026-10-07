//! Source verification grants only a same-image dirty-mark edit. It does not
//! prove canonical output equality or authorize downstream cutoff.

use std::sync::Arc;

use worth_relational::facade::{
    mvcc::{
        CompanionBranchCell, CompanionBranchImage, CompanionCellEditStop, CompanionPreflightStop,
    },
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

use crate::domain_computation::primary_graph::application_attempt::{
    Movement, WorthQuerySourceCurrentnessFailure,
};

use super::super::RecordedSettlementIdentity;
use super::{
    admission::IndexAdmission,
    index_capacity,
    mark_state::{SettlementCurrentness, SettlementMarks},
    retention,
    source_alignment::{BranchMarkRoot, SnapshotAlignedMarkState},
    InvalidationEditAdmission, SourceInvalidationOwner,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum SettlementVerificationStop {
    Alignment,
    Admission(CompanionPreflightStop),
    SourceRead(WorthQuerySourceCurrentnessFailure),
    Edit(CompanionCellEditStop),
    PendingUpstream,
}

impl From<CompanionPreflightStop> for SettlementVerificationStop {
    fn from(value: CompanionPreflightStop) -> Self {
        Self::Admission(value)
    }
}

impl From<super::SettlementRegistrationStop> for SettlementVerificationStop {
    fn from(stop: super::SettlementRegistrationStop) -> Self {
        match stop {
            super::SettlementRegistrationStop::Alignment(_)
            | super::SettlementRegistrationStop::Foreign
            | super::SettlementRegistrationStop::SourceUnavailable => Self::Alignment,
            super::SettlementRegistrationStop::Admission(reason) => Self::Admission(reason),
            super::SettlementRegistrationStop::Edit(reason) => Self::Edit(reason),
        }
    }
}

pub(in crate::domain_computation::primary_graph) enum DirtyReverification {
    Verified(VerifiedDirtySettlement),
    HistoricalCurrent,
    AlreadyCurrent,
    ChangedOrdinal(usize),
    ChangedComputation,
}

/// Minted only after native comparisons at the exact selected source root.
/// The non-clone image binds cell topology, source position, and derived row.
pub(in crate::domain_computation::primary_graph) struct VerifiedDirtySettlement {
    runtime_instance_id: u64,
    cell: CompanionBranchCell<BranchMarkRoot>,
    image: CompanionBranchImage<BranchMarkRoot>,
    identity: Arc<RecordedSettlementIdentity>,
    row: Arc<SettlementMarks>,
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn reverify_dirty(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        identity: &Arc<RecordedSettlementIdentity>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<DirtyReverification, SettlementVerificationStop> {
        // Resolving the native root precedes reading its mark image. The
        // positioned proof is descriptive; this pins its actual read source.
        admission.work(1 + selected.branch_id().0.len() as u64)?;
        admission.bytes(selected.branch_id().0.len() as u64)?;
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let cell = self
            .cell_for_read(selected, admission)?
            .ok_or(SettlementVerificationStop::Alignment)?;
        let image = cell.read_image();
        admission.ordered_read(image.payload().past.len())?;
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(|_| SettlementVerificationStop::Alignment)?;
        admission.ordered_read(aligned.settlement_count())?;
        let live = image.root_id() == selected.root_id()
            && image.commit_id() == selected.commit_id()
            && image.position() == selected.position();
        let state = if live {
            &image.payload().current
        } else {
            admission.ordered_read(image.payload().past.len())?;
            &image
                .payload()
                .past
                .get(&selected.position())
                .expect("aligned historical image retains its selected state")
                .state
        };
        admission.ordered_read(state.settlements.len())?;
        let row = Arc::clone(
            state
                .settlements
                .get(identity)
                .ok_or(SettlementVerificationStop::Alignment)?,
        );
        if row.facts.for_comparison().is_none() {
            return Ok(DirtyReverification::ChangedComputation);
        }
        let ordinals = match aligned.currentness(identity) {
            SettlementCurrentness::Clean => return Ok(DirtyReverification::AlreadyCurrent),
            SettlementCurrentness::Dirty(ordinals) => ordinals.clone(),
            SettlementCurrentness::PendingUpstream(_) => {
                return Err(SettlementVerificationStop::PendingUpstream)
            }
            SettlementCurrentness::FullVerificationRequired(_) | SettlementCurrentness::Foreign => {
                return Err(SettlementVerificationStop::Alignment)
            }
        };
        for ordinal in ordinals {
            admission.work(1)?;
            let fact = row
                .fact_at(ordinal)
                .ok_or(SettlementVerificationStop::Alignment)?;
            let movement = fact
                .source_currentness_in(runtime, snapshot, admission)?
                .map_err(SettlementVerificationStop::SourceRead)?;
            if movement.movement() == Movement::Moved {
                return Ok(DirtyReverification::ChangedOrdinal(ordinal));
            }
        }
        if !live {
            return Ok(DirtyReverification::HistoricalCurrent);
        }
        Ok(DirtyReverification::Verified(VerifiedDirtySettlement {
            runtime_instance_id: self.runtime_instance_id,
            cell,
            image,
            identity: Arc::clone(identity),
            row,
        }))
    }

    /// Historical states remain immutable. A concurrent native delivery or
    /// registration invalidates the sealed image and leaves dirty marks intact.
    pub(in crate::domain_computation::primary_graph) fn clear_verified_dirty(
        &self,
        verified: VerifiedDirtySettlement,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), SettlementVerificationStop> {
        let VerifiedDirtySettlement {
            runtime_instance_id,
            cell,
            image,
            identity,
            row,
        } = verified;
        if runtime_instance_id != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment);
        }
        let before = admission.charged_bytes();
        admission.bytes(
            index_capacity::arc_bytes::<SettlementMarks>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        admission.bytes(
            index_capacity::arc_bytes::<super::mark_state::MarkState>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut replacement = (*row).clone();
        replacement.dirty_ordinals = im::OrdSet::new();
        let mut state = (*image.payload().current).clone();
        state.dirty_ordinal_count -= row.dirty_ordinals.len();
        admission.ordered_edit::<Arc<RecordedSettlementIdentity>, Arc<SettlementMarks>>(
            state.settlements.len(),
        )?;
        state.settlements.insert(identity, Arc::new(replacement));
        let root = retention::admit_live_replacement(
            image.payload(),
            state,
            before,
            &self.resources,
            admission,
        )?;
        let prepared = self
            .prepare_root_replacement(cell, image, Arc::new(root), admission)
            .map_err(SettlementVerificationStop::from)?;
        let cleanup = prepared
            .install()
            .map_err(|stopped| SettlementVerificationStop::Edit(stopped.reason()))?;
        drop(cleanup);
        Ok(())
    }
}
