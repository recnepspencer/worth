//! One exact pending dependency selected from retained consumed evidence.

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::{
    verification::{map_admission_stop as admission_stop, ConsumedOutputVerificationStop},
    ConsumedOutputEvidence,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    ConsumedOutputCurrentness, InvalidationEditAdmission, SourceSettlementCurrentness,
};
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::SourceInvalidationOwner;

/// Scheduling custody only. The retained evidence funds its exact identity;
/// the selected native root remains borrowed from the admitted Product read.
pub(in crate::domain_computation::primary_graph) struct SelectedPendingConsumedOutput<'selected> {
    evidence: ConsumedOutputEvidence,
    selected: &'selected PositionedRelationalSnapshot,
}

impl SelectedPendingConsumedOutput<'_> {
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &RecordedSettlementIdentity {
        self.evidence.identity().as_ref()
    }

    /// Retain the exact selected evidence identity after a short prepared
    /// permission ends. This is a prepaid shallow Arc pin.
    pub(in crate::domain_computation::primary_graph) fn retain_identity(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        std::sync::Arc<RecordedSettlementIdentity>,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        admission.charge_external_work(
            u64::try_from(std::mem::size_of::<std::sync::Arc<RecordedSettlementIdentity>>() + 1)
                .map_err(|_| {
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow
                })?,
        )?;
        Ok(std::sync::Arc::clone(self.evidence.identity()))
    }

    pub(in crate::domain_computation::primary_graph) fn selected_root(
        &self,
    ) -> &PositionedRelationalSnapshot {
        self.selected
    }

    /// Reborrow the same selected native root after a shorter Product
    /// permission callback ends. This moves the retained evidence unchanged;
    /// a different root cannot inherit its pending-dependency selection.
    pub(in crate::domain_computation::primary_graph) fn rebind_selected<'selected>(
        self,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'selected>>, ConsumedOutputVerificationStop>
    {
        charge(admission, 1)?;
        if !std::ptr::eq(self.selected, selected) {
            return Ok(None);
        }
        Ok(Some(SelectedPendingConsumedOutput {
            evidence: self.evidence,
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
        roots: &[Self],
        owner: &SourceInvalidationOwner,
        selected: &'selected PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedPendingConsumedOutput<'selected>>, ConsumedOutputVerificationStop>
    {
        charge(admission, roots.len())?;
        for evidence in roots {
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
            if matches!(
                current,
                ConsumedOutputCurrentness::Direct(
                    SourceSettlementCurrentness::Dirty(_)
                        | SourceSettlementCurrentness::PendingUpstream(_)
                )
            ) {
                // This is the retained immediate root, not a descendant
                // selected by walking through an unresolved producer.
                charge(admission, 8)?;
                return Ok(Some(SelectedPendingConsumedOutput {
                    evidence: evidence.clone(),
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
