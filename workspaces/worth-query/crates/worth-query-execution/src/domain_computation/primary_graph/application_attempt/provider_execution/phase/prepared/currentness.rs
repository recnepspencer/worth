//! Current authority and product basis for a prepared application commit.

use super::*;

pub(super) fn select_current_product<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retained: &crate::basis::WorthQueryProductBranchLease,
) -> Result<crate::basis::WorthQueryProductBranchLease, WorthQueryApplicationCommitOutcome> {
    application
        .product_runtime
        .admit_product_occurrence(retained.observation().lifecycle_incarnation())
        .map_err(|denial| {
            denied_with_detail(
                DenialStage::DecisionReadSet,
                format!("current product admission: {denial:?}"),
            )
        })
}

/// Keep the original decision facts, but select one new publication basis before
/// provider affinity is minted. Every fact is subsequently compared on this
/// exact snapshot; Relational admission and World's publication CAS still reject
/// any further head movement. Exact-parent candidates retain their old basis.
pub(super) fn readmit_current_basis<Schema, Operation, Input, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &mut WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    retained: WorthQueryApplicationSnapshotLease,
    current: crate::basis::WorthQueryProductBranchLease,
    ordinary: bool,
) -> Result<WorthQueryApplicationSnapshotLease, WorthQueryApplicationCommitOutcome> {
    if !ordinary
        || admission.has_elevation_lifecycle_binding()
        || retained.product().has_same_selected_occurrence(&current)
        || retained.product().signal_basis().descriptor() != current.signal_basis().descriptor()
        || retained
            .product()
            .observation()
            .basis()
            .correspondence_basis()
            != current.observation().basis().correspondence_basis()
    {
        return Ok(retained);
    }
    let current_admission = acquire_current_lease(&retained, current.retained_clone())?;
    if let Some(support) = application.program_support.as_ref() {
        super::super::super::program_occurrence_gate::require_readmitted_program_matches(
            support,
            &retained,
            &current_admission,
        )
        .map_err(WorthQueryApplicationCommitOutcome::Denied)?;
    }
    admission
        .graph_work_mut()
        .readmit_mutation_lease(retained.product(), current_admission)
        .map_err(|_| denied(DenialStage::DecisionReadSet))?;
    let current_decision = acquire_current_lease(&retained, current)?;
    // No handler rerun or fact replacement: comparison uses the sealed values.
    Ok(current_decision)
}

fn acquire_current_lease(
    retained: &WorthQueryApplicationSnapshotLease,
    product: crate::basis::WorthQueryProductBranchLease,
) -> Result<WorthQueryApplicationSnapshotLease, WorthQueryApplicationCommitOutcome> {
    use crate::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLeaseDenial as Denial;
    WorthQueryApplicationSnapshotLease::acquire(
        retained.handle().clone(),
        std::sync::Arc::clone(&retained.layout),
        product,
    )
    .map_err(|denial| {
        WorthQueryApplicationCommitOutcome::Denied(match denial {
            crate::domain_computation::primary_graph::WorthQueryApplicationSnapshotLeaseDenial::Handle(denial) => denial.into(),
            Denial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots,
            } => WorthQueryApplicationCommitDenial::active_snapshot_capacity_exhausted(
                DenialStage::DecisionReadSet,
                maximum_active_snapshots,
            ),
            Denial::SnapshotIdentityExhausted => {
                WorthQueryApplicationCommitDenial::snapshot_identity_exhausted(
                    DenialStage::DecisionReadSet,
                )
            }
            Denial::ForeignRuntime => {
                WorthQueryApplicationCommitDenial::provider_rejected(DenialStage::DecisionReadSet)
            }
        })
    })
}

pub(super) fn validate_operation_currentness<Schema, Operation, Input, Scope>(
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    admission.validate_current_authority().map_err(|denial| {
        commit_outcome_from_authorization_denial(denial, DenialStage::DecisionReadSet)
    })
}

pub(super) fn validate_elevation_currentness<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    elevation_currentness: Option<WorthQueryElevationCommitCurrentness>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    if elevation_currentness
        .as_ref()
        .is_some_and(|currentness| !currentness.remains_current(&application.authorization_clock))
    {
        Err(denied(DenialStage::DecisionReadSet))
    } else {
        Ok(())
    }
}

/// A workflow step prepared before its instance's deadline still commits only
/// while the deadline lies ahead on the installed clock.
pub(super) fn validate_workflow_deadline<Schema>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    deadline: Option<u64>,
) -> Result<(), WorthQueryApplicationCommitOutcome> {
    deadline.map_or(Ok(()), |deadline| {
        super::super::super::super::workflow_deadline::ensure_before(
            &application.authorization_clock,
            deadline,
        )
        .map_err(|denial| {
            WorthQueryApplicationCommitOutcome::Denied(
                WorthQueryApplicationCommitDenial::workflow_settlement_denied(&denial),
            )
        })
    })
}
