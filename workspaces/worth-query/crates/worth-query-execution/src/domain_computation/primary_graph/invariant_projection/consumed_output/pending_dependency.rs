//! One exact pending dependency selected from retained consumed evidence.

use std::sync::Arc;

use worth_relational::facade::runtime::{PositionedRelationalSnapshot, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;

use super::{
    verification::{
        map_admission_stop as admission_stop, ConsumedOutputVerification,
        ConsumedOutputVerificationStop,
    },
    ConsumedOutputEvidence,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    ConsumedOutputCurrentness, InvalidationEditAdmission, SourceSettlementCurrentness,
};
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;

/// Scheduling custody only. The retained evidence funds its exact identity;
/// the selected native root remains borrowed from the admitted Product read.
/// The record's whole root set is kept so a consumer of several outputs can
/// continue to its next pending root once this one is resolved.
pub(in crate::domain_computation::primary_graph) struct SelectedPendingConsumedOutput<'selected> {
    roots: Arc<[ConsumedOutputEvidence]>,
    index: usize,
    selected: &'selected PositionedRelationalSnapshot,
}

impl<'selected> SelectedPendingConsumedOutput<'selected> {
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &RecordedSettlementIdentity {
        self.roots[self.index].identity().as_ref()
    }

    /// Retain the exact selected evidence identity after a short prepared
    /// permission ends. This is a prepaid shallow Arc pin.
    pub(in crate::domain_computation::primary_graph) fn retain_identity(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Arc<RecordedSettlementIdentity>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        admission.charge_external_work(
            u64::try_from(std::mem::size_of::<Arc<RecordedSettlementIdentity>>() + 1).map_err(
                |_| worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
            )?,
        )?;
        Ok(Arc::clone(self.roots[self.index].identity()))
    }

    pub(in crate::domain_computation::primary_graph) fn selected_root(
        &self,
    ) -> &PositionedRelationalSnapshot {
        self.selected
    }

    /// The same record's next immediate pending root after this one, on the
    /// same selected native root.
    pub(in crate::domain_computation::primary_graph) fn next_pending(
        &self,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<Self>, ConsumedOutputVerificationStop> {
        ConsumedOutputEvidence::select_pending_from(
            &self.roots,
            self.index + 1,
            owner,
            runtime,
            snapshot,
            self.selected,
            admission,
        )
    }

    /// Reborrow the same selected native root after a shorter Product
    /// permission callback ends. This moves the retained evidence unchanged;
    /// a different root cannot inherit its pending-dependency selection.
    pub(in crate::domain_computation::primary_graph) fn rebind_selected<'rebound>(
        self,
        selected: &'rebound PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'rebound>>, ConsumedOutputVerificationStop>
    {
        charge(admission, 1)?;
        if !std::ptr::eq(self.selected, selected) {
            return Ok(None);
        }
        Ok(Some(SelectedPendingConsumedOutput {
            roots: self.roots,
            index: self.index,
            selected,
        }))
    }
}

impl ConsumedOutputEvidence {
    /// Select one immediate consumed edge whose actor mark requires its own
    /// progression. Its own Ready proof may then select a further upstream
    /// edge; skipping that intermediate record would miss its output cutoff.
    pub(in crate::domain_computation::primary_graph) fn select_exact_pending_dependency<
        'selected,
    >(
        roots: &Arc<[Self]>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'selected>>, ConsumedOutputVerificationStop>
    {
        Self::select_pending_from(roots, 0, owner, runtime, snapshot, selected, admission)
    }

    fn select_pending_from<'selected>(
        roots: &Arc<[Self]>,
        start: usize,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'selected>>, ConsumedOutputVerificationStop>
    {
        charge(admission, roots.len().saturating_sub(start))?;
        for (index, evidence) in roots.iter().enumerate().skip(start) {
            let observed = &evidence.selected_native_root;
            let branch_comparison = selected
                .branch_id()
                .0
                .len()
                .checked_add(observed.branch_id().0.len())
                .and_then(|visits| visits.checked_add(4))
                .ok_or(ConsumedOutputVerificationStop::WorkExhausted)?;
            charge(admission, branch_comparison)?;
            if selected.runtime_instance_id() != observed.runtime_instance_id()
                || selected.branch_id() != observed.branch_id()
                || selected.version_id() < observed.version_id()
                || selected.position() < observed.position()
            {
                return Ok(None);
            }
            let current = owner
                .consumed_output_currentness(selected, evidence.identity(), admission)
                .map_err(admission_stop)?;
            let pending = match current {
                ConsumedOutputCurrentness::Direct(
                    SourceSettlementCurrentness::Dirty(_)
                    | SourceSettlementCurrentness::PendingUpstream(_),
                ) => true,
                // The edge reads an older row of a certified-equal chain whose
                // newest row is marked: that upstream's own progression
                // answers for the edge, exactly as for a directly marked row.
                ConsumedOutputCurrentness::PendingEqualSuccessor => true,
                // Marks no longer cover this edge: it is pending when its own
                // facts or output show a change.
                ConsumedOutputCurrentness::Direct(
                    SourceSettlementCurrentness::FullVerificationRequired(_),
                ) => match Self::verify_many_with_admission(
                    std::slice::from_ref(evidence),
                    owner,
                    runtime,
                    snapshot,
                    selected,
                    admission,
                ) {
                    Ok(verification) => verification != ConsumedOutputVerification::Current,
                    Err(ConsumedOutputVerificationStop::PendingUpstream) => true,
                    Err(ConsumedOutputVerificationStop::Unavailable) => false,
                    Err(stop) => return Err(stop),
                },
                _ => false,
            };
            if pending {
                // This is the retained immediate root, not a descendant
                // selected by walking through an unresolved producer.
                charge(admission, 8)?;
                return Ok(Some(SelectedPendingConsumedOutput {
                    roots: Arc::clone(roots),
                    index,
                    selected,
                }));
            }
        }
        Ok(None)
    }
}

fn charge(
    admission: &mut InvalidationEditAdmission,
    visits: usize,
) -> Result<(), ConsumedOutputVerificationStop> {
    admission
        .charge_external_work(
            u64::try_from(visits).map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?,
        )
        .map_err(admission_stop)
}
