//! Diagnostic verification of authoritative facts, independent of mark decisions.

use std::{mem::size_of, sync::Arc};

use im::OrdSet;
use worth_relational::facade::{
    mvcc::CompanionPreflightStop,
    runtime::{
        PositionedRelationalSnapshot, RelationalRuntime, RelationalSnapshotPositionAdmissionStop,
    },
    snapshots::SnapshotHandle,
};

use super::{
    admission::IndexAdmission,
    mark_state::{MarkState, OutputFactCoverage},
    source_alignment::SnapshotAlignedMarkState,
    FullVerificationReason, InvalidationEditAdmission, SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::{
    application_attempt::{Movement, WorthQuerySourceCurrentnessFailure},
    output_lineage::{RecordedSettlementIdentity, SealedNativeOutputWitness},
    WorthQueryApplicationObservedFact,
};

mod equality;
mod image_fence;
mod root_source;
use crate::domain_computation::primary_graph::invariant_projection::EvidenceView;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum FullVerificationDecision {
    Current,
    Changed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum FullVerificationStop {
    Alignment(FullVerificationReason),
    /// The selected source or a row it reaches belongs to another source.
    Foreign,
    /// The selected source snapshot cannot be positioned.
    SourceUnavailable,
    Admission(CompanionPreflightStop),
    SourceRead(WorthQuerySourceCurrentnessFailure),
    OutputEvidenceUnavailable,
    ActorImageChanged,
}

impl From<CompanionPreflightStop> for FullVerificationStop {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Admission(stop)
    }
}

/// The observer retains the exact pre-verification image. It grants no edit,
/// scheduling, equality, or publication authority.
pub(in crate::domain_computation::primary_graph) struct FullVerificationImage<'selected> {
    state: Arc<MarkState>,
    selected: &'selected PositionedRelationalSnapshot,
    runtime: &'selected RelationalRuntime,
    snapshot: &'selected SnapshotHandle,
    owner: &'selected SourceInvalidationOwner,
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn capture_full_verification<'selected>(
        &'selected self,
        runtime: &'selected RelationalRuntime,
        snapshot: &'selected SnapshotHandle,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<FullVerificationImage<'selected>, FullVerificationStop> {
        let actual = runtime
            .read_truth()
            .positioned_snapshot_admitted(snapshot, |work, bytes| {
                admission.work(work)?;
                admission.bytes(bytes)
            })
            .map_err(|stop| match stop {
                RelationalSnapshotPositionAdmissionStop::Admission(stop) => {
                    FullVerificationStop::Admission(stop)
                }
                RelationalSnapshotPositionAdmissionStop::AccountingOverflow => {
                    FullVerificationStop::Admission(CompanionPreflightStop::WorkCounterOverflow)
                }
                RelationalSnapshotPositionAdmissionStop::Position(_) => {
                    FullVerificationStop::SourceUnavailable
                }
            })?;
        admission.work(1 + selected.branch_id().0.len() as u64)?;
        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(FullVerificationStop::Foreign);
        }
        let cell =
            self.cell_for_read(selected, admission)?
                .ok_or(FullVerificationStop::Alignment(
                    FullVerificationReason::MissingSettlement,
                ))?;
        admission.work(3)?;
        let image = cell.read_image();
        admission.ordered_read(image.payload().past.len())?;
        let aligned = SnapshotAlignedMarkState::select_image(image, selected)
            .map_err(FullVerificationStop::Alignment)?;
        Ok(FullVerificationImage {
            state: aligned.into_full_verification_state(),
            selected,
            runtime,
            snapshot,
            owner: self,
        })
    }
}

impl FullVerificationImage<'_> {
    pub(in crate::domain_computation::primary_graph) fn verify_source_view(
        &self,
        root: EvidenceView<'_>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        self.verify_root_evidence(
            root.identity(),
            root.native_output_witness(),
            Some(root),
            admission,
        )
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn verify_root(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        self.verify_root_evidence(identity, fallback_witness, None, admission)
    }

    fn verify_root_evidence(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
        root_source: Option<EvidenceView<'_>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut observed_rows = OrdSet::new();
        let result = self.verify_closure(
            identity,
            fallback_witness,
            root_source,
            &mut observed_rows,
            admission,
        )?;
        self.fence_semantic_image(&observed_rows, admission)?;
        Ok(result)
    }

    fn verify_closure(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
        root_source: Option<EvidenceView<'_>>,
        observed_rows: &mut OrdSet<Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        reserve_pending(&mut pending, 1, admission)?;
        admission.work(1)?;
        pending.push((Arc::clone(identity), true));
        while let Some((original, direct)) = pending.pop() {
            image_fence::observe_row(observed_rows, &original, admission)?;
            let terminal = equality::terminal(&self.state, &original, admission)?;
            image_fence::observe_row(observed_rows, terminal, admission)?;
            let source = root_source.filter(|_| direct && Arc::ptr_eq(&original, terminal));
            if let Some(source) = source {
                if !self.root_source_is_current(source, admission)? {
                    return Ok(FullVerificationDecision::Changed);
                }
                self.require_complete_root_upstream(source, terminal, admission)?;
            }
            admission.admit_visited_settlement(visited.len())?;
            if visited.contains(terminal) {
                continue;
            }
            visited.insert(Arc::clone(terminal));
            admission.ordered_read(self.state.settlements.len())?;
            let row =
                self.state
                    .settlements
                    .get(terminal)
                    .ok_or(FullVerificationStop::Alignment(
                        FullVerificationReason::MissingSettlement,
                    ))?;
            admission.work(1 + row.read_basis.branch_id().0.len() as u64)?;
            if row.read_basis.runtime_instance_id() != self.selected.runtime_instance_id()
                || row.read_basis.branch_id() != self.selected.branch_id()
                || row.read_basis.version_id() > self.selected.version_id()
                || row.read_basis.position() > self.selected.position()
            {
                return Err(FullVerificationStop::Foreign);
            }
            for fact in row.facts.iter() {
                if !fact_is_current(fact, self.runtime, self.snapshot, admission)? {
                    return Ok(FullVerificationDecision::Changed);
                }
            }
            let projection =
                equality::output_projection(&self.state, terminal, observed_rows, admission)?;
            if let Some(projection) = projection {
                for fact in projection.facts.iter() {
                    if !fact_is_current(fact, self.runtime, self.snapshot, admission)? {
                        return Ok(FullVerificationDecision::Changed);
                    }
                }
            } else if let Some(witness) = fallback_witness.filter(|_| direct) {
                if !witness.unchanged_in(self.runtime, self.snapshot, admission)? {
                    return Ok(FullVerificationDecision::Changed);
                }
            } else {
                return Err(FullVerificationStop::OutputEvidenceUnavailable);
            }
            reserve_pending(&mut pending, row.consumed_upstream.len(), admission)?;
            admission.work(row.consumed_upstream.len() as u64)?;
            pending.extend(
                row.consumed_upstream
                    .iter()
                    .map(|upstream| (Arc::clone(upstream), false)),
            );
        }
        Ok(FullVerificationDecision::Current)
    }
}

fn fact_is_current(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, FullVerificationStop> {
    let movement = fact
        .source_currentness_in(runtime, snapshot, admission)?
        .map_err(FullVerificationStop::SourceRead)?;
    Ok(movement.movement() == Movement::Unmoved)
}

fn reserve_pending(
    pending: &mut Vec<(Arc<RecordedSettlementIdentity>, bool)>,
    additional: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), FullVerificationStop> {
    let needed = pending
        .len()
        .checked_add(additional)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    if needed <= pending.capacity() {
        return Ok(());
    }
    let bytes = needed
        .checked_mul(size_of::<(Arc<RecordedSettlementIdentity>, bool)>())
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    admission.bytes(bytes)?;
    admission.work(pending.len() as u64)?;
    pending
        .try_reserve_exact(additional)
        .map_err(|_| FullVerificationStop::OutputEvidenceUnavailable)
}
