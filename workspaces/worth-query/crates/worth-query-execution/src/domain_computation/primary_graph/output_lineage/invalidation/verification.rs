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

use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;

use super::super::RecordedSettlementIdentity;
use super::{
    admission::IndexAdmission,
    index_capacity,
    mark_state::{FullVerificationReason, SettlementCurrentness, SettlementMarks},
    retention,
    source_alignment::{BranchMarkRoot, SnapshotAlignedMarkState},
    InvalidationEditAdmission, SourceInvalidationOwner,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) enum SettlementVerificationStop {
    Alignment(FullVerificationReason),
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

pub(in crate::domain_computation::primary_graph) enum DirtyReverification {
    Verified(VerifiedDirtySettlement),
    HistoricalCurrent,
    AlreadyCurrent,
    ChangedOrdinal(usize),
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
            .map_err(|denial| {
                SettlementVerificationStop::Alignment(
                    FullVerificationReason::SelectedSourceUnavailable(denial),
                )
            })?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(SettlementVerificationStop::Alignment(
                FullVerificationReason::ForeignSource,
            ));
        }
        let cell = self.cell_for_read(selected, admission)?.ok_or(
            SettlementVerificationStop::Alignment(FullVerificationReason::MissingSettlement),
        )?;
        let image = cell.read_image();
        admission.ordered_read(image.payload().past.len())?;
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::observe_image(&image, selected)
            .map_err(SettlementVerificationStop::Alignment)?;
        admission.ordered_read(aligned.settlement_count())?;
        let ordinals = match aligned.currentness(identity) {
            SettlementCurrentness::Clean => return Ok(DirtyReverification::AlreadyCurrent),
            SettlementCurrentness::Dirty(ordinals) => ordinals.clone(),
            SettlementCurrentness::PendingUpstream(_) => {
                return Err(SettlementVerificationStop::PendingUpstream)
            }
            SettlementCurrentness::FullVerificationRequired(reason) => {
                return Err(SettlementVerificationStop::Alignment(reason))
            }
        };
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
                .expect("aligned currentness selected this exact row"),
        );
        for ordinal in ordinals {
            admission.work(1)?;
            let fact = row
                .fact_at(ordinal)
                .ok_or(SettlementVerificationStop::Alignment(
                    FullVerificationReason::MissingSettlement,
                ))?;
            let remaining = admission.remaining_work();
            let prepaid = fact
                .exact_probe_work()
                .map_err(SettlementVerificationStop::SourceRead)?
                .unwrap_or(0);
            admission.work(prepaid as u64)?;
            let (current, work) = fact
                .source_currentness_in(runtime, snapshot, remaining)
                .map_err(SettlementVerificationStop::SourceRead)?;
            admission.work(work.saturating_sub(prepaid) as u64)?;
            if !current {
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
            return Err(SettlementVerificationStop::Alignment(
                FullVerificationReason::ForeignSource,
            ));
        }
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
        retention::admit_state(&mut state, &self.resources, admission)?;
        admission.bytes(
            index_capacity::arc_bytes::<BranchMarkRoot>()
                .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?,
        )?;
        let mut root = (**image.payload()).clone();
        root.current = Arc::new(state);
        retention::admit_root(&mut root, &self.resources, admission)?;
        let prepared = self
            .prepare_root_replacement(cell, image, Arc::new(root), admission)
            .map_err(|stop| match stop {
                super::SettlementRegistrationStop::Alignment(reason) => {
                    SettlementVerificationStop::Alignment(reason)
                }
                super::SettlementRegistrationStop::Admission(reason) => {
                    SettlementVerificationStop::Admission(reason)
                }
                super::SettlementRegistrationStop::Edit(reason) => {
                    SettlementVerificationStop::Edit(reason)
                }
            })?;
        let cleanup = prepared
            .install()
            .map_err(|stopped| SettlementVerificationStop::Edit(stopped.reason()))?;
        drop(cleanup);
        Ok(())
    }
}
