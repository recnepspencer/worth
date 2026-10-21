//! Diagnostic verification of authoritative facts, independent of mark decisions.

use std::sync::Arc;

use im::OrdSet;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::mark_state::MarkState;
use super::mark_state::OutputFactCoverage;
use super::source_alignment::SnapshotAlignedMarkState;
use super::FullVerificationReason;
use super::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::application_attempt::Movement;
use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::output_lineage::SealedNativeOutputWitness;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;

mod equality;
mod image_fence;
mod native_evidence;
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
    native_only: bool,
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
    ) -> Result<FullVerificationImage<'selected>, FullVerificationStop> {
        let actual = runtime
            .read_truth()
            .positioned_snapshot(snapshot)
            .map_err(|_| FullVerificationStop::SourceUnavailable)?;

        if &actual != selected || selected.runtime_instance_id() != self.runtime_instance_id {
            return Err(FullVerificationStop::Foreign);
        }
        let cell = self.cell_for_read(selected)?;
        let aligned = if let Some(cell) = cell {
            let image = cell.read_image();

            SnapshotAlignedMarkState::select_image(image, selected).ok()
        } else {
            None
        };
        let native_only = aligned.is_none();
        let state = if let Some(aligned) = aligned {
            aligned.into_full_verification_state()
        } else {
            Arc::new(MarkState::initial())
        };
        Ok(FullVerificationImage {
            state,
            native_only,
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        root: EvidenceView<'_>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        if self.native_only
            || (!self.state.settlements.contains_key(root.identity())
                && !self.state.equal_links.contains_key(root.identity()))
        {
            return self.verify_native_evidence(execution, root);
        }
        self.verify_root_evidence(
            execution,
            root.identity(),
            root.native_output_witness(),
            Some(root),
        )
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn verify_root(
        &self,
        request_execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        self.verify_root_evidence(request_execution, identity, fallback_witness, None)
    }

    fn verify_root_evidence(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
        root_source: Option<EvidenceView<'_>>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut observed_rows = OrdSet::new();
        let result = self.verify_closure(
            execution,
            identity,
            fallback_witness,
            root_source,
            &mut observed_rows,
        )?;
        self.fence_semantic_image(&observed_rows)?;
        Ok(result)
    }

    fn verify_closure(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &Arc<RecordedSettlementIdentity>,
        fallback_witness: Option<&SealedNativeOutputWitness>,
        root_source: Option<EvidenceView<'_>>,
        observed_rows: &mut OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Result<FullVerificationDecision, FullVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        reserve_pending(&mut pending, 1)?;

        pending.push((Arc::clone(identity), true));
        while let Some((original, direct)) = pending.pop() {
            image_fence::observe_row(observed_rows, &original)?;
            let terminal = equality::terminal(&self.state, &original)?;
            image_fence::observe_row(observed_rows, terminal)?;
            let source = root_source.filter(|_| direct && Arc::ptr_eq(&original, terminal));
            if let Some(source) = source {
                if !self.root_source_is_current(execution, source)? {
                    return Ok(FullVerificationDecision::Changed);
                }
                self.require_complete_root_upstream(source, terminal)?;
            }

            if visited.contains(terminal) {
                continue;
            }
            visited.insert(Arc::clone(terminal));

            let row =
                self.state
                    .settlements
                    .get(terminal)
                    .ok_or(FullVerificationStop::Alignment(
                        FullVerificationReason::MissingSettlement,
                    ))?;

            if row.read_basis.runtime_instance_id() != self.selected.runtime_instance_id()
                || row.read_basis.branch_id() != self.selected.branch_id()
                || row.read_basis.version_id() > self.selected.version_id()
                || row.read_basis.position() > self.selected.position()
            {
                return Err(FullVerificationStop::Foreign);
            }
            let Some(facts) = row.facts.for_comparison() else {
                return Ok(FullVerificationDecision::Changed);
            };
            for fact in facts.iter() {
                if !fact_is_current(execution, fact, self.runtime, self.snapshot)? {
                    return Ok(FullVerificationDecision::Changed);
                }
            }
            let projection = equality::output_projection(&self.state, terminal, observed_rows)?;
            if let Some(projection) = projection {
                for fact in projection.facts.iter() {
                    if !fact_is_current(execution, fact, self.runtime, self.snapshot)? {
                        return Ok(FullVerificationDecision::Changed);
                    }
                }
            } else if let Some(witness) = fallback_witness.filter(|_| direct) {
                if !witness.unchanged_in(self.runtime, self.snapshot)? {
                    return Ok(FullVerificationDecision::Changed);
                }
            } else {
                return Err(FullVerificationStop::OutputEvidenceUnavailable);
            }
            reserve_pending(&mut pending, row.consumed_upstream.len())?;

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
) -> Result<bool, FullVerificationStop> {
    let movement = fact
        .source_currentness_in(runtime, execution, snapshot)?
        .map_err(FullVerificationStop::SourceRead)?;
    Ok(movement.movement() == Movement::Unmoved)
}

fn reserve_pending<T>(pending: &mut Vec<T>, additional: usize) -> Result<(), FullVerificationStop> {
    let needed = pending
        .len()
        .checked_add(additional)
        .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
    if needed <= pending.capacity() {
        return Ok(());
    }
    // Doubling keeps admitted growth within twice the final backing.
    let target = needed.max(pending.capacity().saturating_mul(2));

    pending
        .try_reserve_exact(target - pending.len())
        .map_err(|_| FullVerificationStop::OutputEvidenceUnavailable)
}
