mod admission_affinity;
mod basis_denials;
mod branch_registry;
mod cancellation;
mod cell_posture_outcomes;
mod cell_progress;
mod close_cleanup;
mod definition_custody;
mod exact_cell_contracts;
mod exact_retirement_contracts;
mod fork_contracts;
pub(in crate::branch::owner_services) mod fork_sharing;
mod forked_transactions;
mod issuance_capability;
mod kernel_topology;
mod lifecycle;
mod managed_reference;
mod operation_control;
mod output_retention;
mod output_retention_cleanup;
mod progress_bound;
mod registry_capacity;
mod retention_lifecycle;
mod retention_preflight;
mod retirement_cleanup;
mod retirement_cost_accounting;
mod retirement_lineage;
mod retirement_planning;
mod retirement_receipt_oracle;
mod retirement_retention_fence;
mod root_destruction;
pub(super) mod runtime_root;
mod snapshot_identity;
mod snapshot_reservation_ordering;

fn with_movement_permit(operation: impl FnOnce(&super::SignalOwnerMovementPermit<'_>)) {
    let source = super::SignalOwnerCancellationSource::new();
    let token = source.token();
    let permit = token
        .preflight_movement()
        .expect("the test cancellation source remains open");
    operation(&permit);
}

#[cfg(feature = "test-operation-control")]
#[test]
fn test_operation_control_build_preserves_kernel_authority_boundary() {
    use super::super::SignalOwnerOperationBoundary;

    let boundary = SignalOwnerOperationBoundary::TargetCellAdmission;
    assert_eq!(boundary, SignalOwnerOperationBoundary::TargetCellAdmission);
}
