use super::*;
use crate::facade::{
    BridgeExecutionBasisReadmissionCleanupOutcome, BridgeExecutionBasisReadmissionDenialKind,
    BridgeExecutionBasisReadmissionOutcome,
};

#[test]
fn readmission_mints_fresh_signal_generation_and_commits_one_new_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&runtime, resource_request);
    let old_basis = yielded.basis_identity().as_str().to_owned();
    let old_request = yielded.basis_request_identity().to_owned();
    let preflight = exact_preflight(&runtime, yielded);
    let pending = match runtime.readmit_yielded_execution_basis(
        preflight,
        BridgeManagedExecutionIntent::new("query-operation-binding", "attempt-b"),
    ) {
        BridgeExecutionBasisReadmissionOutcome::Pending(pending) => pending,
        _ => panic!("fresh exact intent should become pending"),
    };
    assert_ne!(pending.fresh_request_identity(), old_request);
    assert_eq!(pending.counters().preflight_check_count(), 1);
    assert_eq!(pending.counters().signal_attempt_admission_count(), 1);
    assert_eq!(pending.counters().signal_attempt_check_count(), 1);
    assert_eq!(pending.counters().signal_queue_binding_count(), 1);

    let committed = runtime.commit_yielded_execution_basis_readmission(pending);
    assert_eq!(committed.counters().commit_count(), 1);
    let (basis, counters) = committed.into_parts();
    assert_eq!(counters.commit_count(), 1);
    assert_ne!(basis.identity().as_str(), old_basis);
    assert_eq!(
        basis.managed_intent().resource_attempt_identity(),
        "attempt-b"
    );
    assert_eq!(
        signal_status(&runtime, basis.request().request_handle()),
        ResourceInFlightStatus::Active
    );
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Completed)
        .expect("readmitted basis should complete");
}

#[test]
fn abort_returns_the_exact_yielded_basis_and_releases_provisional_ownership() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&runtime, resource_request);
    let yielded_basis = yielded.basis_identity().as_str().to_owned();
    let yielded_request = yielded.basis_request_identity().to_owned();
    let pending = pending_readmission(&runtime, yielded, "attempt-b");
    let returned = match pending.abort() {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => returned,
        _ => panic!("owner-thread provisional cleanup should complete"),
    };
    assert_eq!(returned.counters().abort_count(), 1);
    let (yielded, counters) = returned.into_parts();
    assert_eq!(counters.abort_count(), 1);
    assert_eq!(yielded.basis_identity().as_str(), yielded_basis);
    assert_eq!(yielded.basis_request_identity(), yielded_request);

    let pending = pending_readmission(&runtime, yielded, "attempt-b");
    let (basis, counters) = runtime
        .commit_yielded_execution_basis_readmission(pending)
        .into_parts();
    assert_eq!(counters.commit_count(), 1);
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .expect("retry after abort should own one live basis");
}

#[test]
fn foreign_thread_abort_returns_recovery_that_owner_thread_can_finish() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&runtime, resource_request);
    let yielded_basis = yielded.basis_identity().as_str().to_owned();
    let pending = pending_readmission(&runtime, yielded, "attempt-b");
    let recovery = std::thread::spawn(move || match pending.abort() {
        BridgeExecutionBasisReadmissionCleanupOutcome::RecoveryRequired(recovery) => recovery,
        _ => panic!("foreign-thread provisional cleanup must retain recovery authority"),
    })
    .join()
    .expect("foreign-thread cleanup probe should return recovery authority");
    assert_eq!(
        recovery.kind(),
        crate::facade::BridgeExecutionBasisReadmissionRecoveryKind::ProvisionalSignalCleanupFailed
    );
    assert!(recovery.detail().contains("belongs to thread"));
    let returned = match recovery.retry_cleanup() {
        BridgeExecutionBasisReadmissionCleanupOutcome::Complete(returned) => returned,
        _ => panic!("Signal owner thread should finish provisional cleanup"),
    };
    assert_eq!(returned.counters().abort_count(), 1);
    let (yielded, counters) = returned.into_parts();
    assert_eq!(counters.abort_count(), 1);
    assert_eq!(yielded.basis_identity().as_str(), yielded_basis);

    let pending = pending_readmission(&runtime, yielded, "attempt-b");
    let (basis, counters) = runtime
        .commit_yielded_execution_basis_readmission(pending)
        .into_parts();
    assert_eq!(counters.commit_count(), 1);
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .expect("cleanup recovery should preserve exact Bridge readmission authority");
}

#[test]
fn foreign_runtime_preflight_returns_the_untouched_yielded_authority() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let owner_runtime = runtime(BridgeRuntimePolicy::development());
    let foreign = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&owner_runtime, resource_request);
    let basis_identity = yielded.basis_identity().as_str().to_owned();
    let denial = match foreign.preflight_yielded_execution_basis(yielded, "query-operation-binding")
    {
        Err(denial) => denial,
        Ok(_) => panic!("foreign Bridge runtime should deny before fresh Signal work"),
    };
    assert_eq!(
        denial.kind(),
        BridgeExecutionBasisReadmissionDenialKind::ForeignRuntime
    );
    assert_eq!(denial.counters().preflight_check_count(), 1);
    assert_eq!(denial.counters().signal_attempt_admission_count(), 0);
    let returned = denial.into_returned_yielded();
    assert_eq!(returned.counters().preflight_check_count(), 1);
    let (yielded, counters) = returned.into_parts();
    assert_eq!(counters.preflight_check_count(), 1);
    assert_eq!(yielded.basis_identity().as_str(), basis_identity);
    let pending = pending_readmission(&owner_runtime, yielded, "attempt-b");
    let (basis, counters) = owner_runtime
        .commit_yielded_execution_basis_readmission(pending)
        .into_parts();
    assert_eq!(counters.commit_count(), 1);
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .expect("foreign-runtime denial should preserve retry authority");
}

#[test]
fn foreign_runtime_cannot_commit_an_owner_runtime_pending_readmission() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let owner_runtime = runtime(BridgeRuntimePolicy::development());
    let foreign_runtime = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&owner_runtime, resource_request);
    let pending = pending_readmission(&owner_runtime, yielded, "attempt-b");
    let commit = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        foreign_runtime.commit_yielded_execution_basis_readmission(pending)
    }));
    assert!(
        commit.is_err(),
        "foreign RuntimeBridge must not mint an active basis from owner pending authority"
    );
}

#[test]
fn reused_query_attempt_denies_before_signal_admission_and_preserves_retry() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let yielded = yielded_basis(&runtime, resource_request);
    let preflight = exact_preflight(&runtime, yielded);
    let denial = match runtime.readmit_yielded_execution_basis(
        preflight,
        BridgeManagedExecutionIntent::new("query-operation-binding", "attempt-a"),
    ) {
        BridgeExecutionBasisReadmissionOutcome::Denied(denial) => denial,
        _ => panic!("reused Query attempt must deny"),
    };
    assert_eq!(
        denial.kind(),
        BridgeExecutionBasisReadmissionDenialKind::AttemptIdentityReused
    );
    assert_eq!(denial.counters().signal_attempt_admission_count(), 0);
    let (yielded, denial_counters) = denial.into_returned_yielded().into_parts();
    assert_eq!(denial_counters.signal_attempt_admission_count(), 0);
    let pending = pending_readmission(&runtime, yielded, "attempt-b");
    let (basis, counters) = runtime
        .commit_yielded_execution_basis_readmission(pending)
        .into_parts();
    assert_eq!(counters.commit_count(), 1);
    basis
        .finalize(BridgeExecutionBasisTerminalDisposition::Cancelled)
        .expect("attempt-reuse denial should preserve retry authority");
}

fn yielded_basis(
    runtime: &RuntimeBridge,
    execution: worth_execution::ExecutionRequest<'_, '_>,
) -> crate::facade::BridgeYieldedExecutionBasis {
    runtime
        .admit_managed_execution_basis(
            managed_intent("attempt-a"),
            step_contract(),
            truth_basis("snapshot-a"),
            planned_truth_view(runtime, execution),
            execution,
        )
        .expect("initial managed basis should admit")
        .yield_execution_basis()
        .expect("owner-thread yield should release Signal and reservation")
}

fn pending_readmission(
    runtime: &RuntimeBridge,
    yielded: crate::facade::BridgeYieldedExecutionBasis,
    attempt: &str,
) -> crate::facade::BridgeExecutionBasisReadmissionPending {
    let preflight = exact_preflight(runtime, yielded);
    match runtime.readmit_yielded_execution_basis(
        preflight,
        BridgeManagedExecutionIntent::new("query-operation-binding", attempt),
    ) {
        BridgeExecutionBasisReadmissionOutcome::Pending(pending) => pending,
        _ => panic!("exact fresh attempt should become pending"),
    }
}

fn exact_preflight(
    runtime: &RuntimeBridge,
    yielded: crate::facade::BridgeYieldedExecutionBasis,
) -> crate::facade::BridgeYieldedExecutionBasisPreflight {
    match runtime.preflight_yielded_execution_basis(yielded, "query-operation-binding") {
        Ok(preflight) => preflight,
        Err(_) => panic!("exact yielded basis should pass preflight"),
    }
}
