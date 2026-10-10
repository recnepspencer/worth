use super::*;
use crate::application::WorthQuerySharedReadPinningBoundaryPosture;

#[test]
fn shared_read_phase_twelve_counter_sabotage_reopens_lock_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (workspace, _derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase12.lock-sabotage");
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));

    workspace
        .runtime
        .record_shared_read_hot_path_lock_for_certification();

    assert!(
        !pinning_phase_twelve_counters_are_closed(&workspace.runtime),
        "a perturbed hot-path lock counter must reopen Phase 12 posture"
    );
}

#[test]
fn shared_read_phase_twelve_counter_sabotage_reopens_pin_residue() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, _derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase12.pin-sabotage");
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));

    let held_read = workspace
        .shared_read_context(request_execution)
        .expect("held read context should mint");
    insert_task(request_execution, &mut workspace, "task-2", "Task Two");

    assert!(
        !pinning_phase_twelve_counters_are_closed(&workspace.runtime),
        "a real held retired generation pin must reopen Phase 12 posture"
    );
    drop(held_read);
    insert_task(request_execution, &mut workspace, "task-3", "Task Three");
    assert!(pinning_phase_twelve_counters_are_closed(&workspace.runtime));
}

#[test]
fn shared_read_phase_thirteen_closure_sabotage_reopens_boundary_posture() {
    let inventory = shared_read_pinning_inventory_evidence();
    let counters = WorthQuerySharedReadPinningCounterEvidence::new(0, 0, 0, 0, 0);
    let matrix = WorthQuerySharedReadPinningHostileMatrixEvidence::new(
        true,
        evidence_digest("shared-read-sabotage-matrix", []),
    );
    let portability = WorthQuerySharedReadPortabilityEvidence::proven(evidence_digest(
        "shared-read-sabotage-portability",
        [],
    ));
    let stale_denial = WorthQuerySharedReadStaleBasisDenialEvidence::proven(evidence_digest(
        "shared-read-sabotage-stale-denial",
        [],
    ));

    for certification in [
        WorthQuerySharedReadPinningCertification::from_evidence(
            inventory.with_missing_operation_for_sabotage(),
            matrix.clone(),
            portability.clone(),
            stale_denial.clone(),
            counters.clone(),
        ),
        WorthQuerySharedReadPinningCertification::from_evidence(
            inventory.clone(),
            matrix.clone(),
            portability.clone(),
            stale_denial.clone(),
            counters.with_unretired_pin_for_sabotage(),
        ),
        WorthQuerySharedReadPinningCertification::from_evidence(
            inventory.clone(),
            matrix.uncertified_for_sabotage(),
            portability.clone(),
            stale_denial.clone(),
            counters.clone(),
        ),
        WorthQuerySharedReadPinningCertification::from_evidence(
            inventory.clone(),
            matrix.clone(),
            portability.missing_for_sabotage(),
            stale_denial.clone(),
            counters.clone(),
        ),
        WorthQuerySharedReadPinningCertification::from_evidence(
            inventory.clone(),
            matrix.clone(),
            portability.clone(),
            stale_denial.missing_for_sabotage(),
            counters.clone(),
        ),
    ] {
        assert_ne!(
            certification.closure().posture(),
            WorthQuerySharedReadPinningBoundaryPosture::Closed,
            "any sabotaged Phase 13 evidence must reopen pinning boundary closure"
        );
    }
}
