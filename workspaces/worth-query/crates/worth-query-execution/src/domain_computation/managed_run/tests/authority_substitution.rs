use super::*;
use crate::domain_computation::{
    WorthQueryManagedDirectRunAdmissionFailureKind, WorthQueryManagedTruthReadRequest,
};

#[test]
fn same_runtime_foreign_bridge_adapter_denies_before_lower_authority_admission() {
    let runtime = query_runtime();
    let plan = admitted_plan("foreign-bridge-adapter", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact operation should start");
    let context = causal_fixture::source_profile_substitution_context();
    let request = WorthQueryManagedTruthReadRequest::from_relational_basis(
        context.descriptor.clone(),
        worth_runtime_bridge::facade::SnapshotReadPacket::new(vec![]),
    );

    let rejection = match runtime
        .managed_run_admission(&context.foreign_bridge, &context.relational)
        .admit_direct(&operation, attempt, request)
    {
        Ok(_) => panic!("same-runtime foreign adapter joined a Relational lease"),
        Err(rejection) => rejection,
    };
    assert_eq!(
        rejection.kind(),
        WorthQueryManagedDirectRunAdmissionFailureKind::ManagedAuthorityJoin
    );
    assert!(rejection.detail().contains("authoritative adapter"));

    let attempt = rejection.into_resource_attempt();
    let request = WorthQueryManagedTruthReadRequest::from_relational_basis(
        context.descriptor,
        worth_runtime_bridge::facade::SnapshotReadPacket::new(vec![]),
    );
    let admitted = runtime
        .managed_run_admission(&context.exact_bridge, &context.relational)
        .admit_direct(&operation, attempt, request)
        .expect("the exact adapter should admit the untouched resource attempt");
    let cleanup = admitted
        .start()
        .terminal(WorthQueryManagedRunTerminalKind::Cancelled)
        .cleanup()
        .expect("owner-thread cleanup should release the exact admitted run");
    assert!(cleanup.inspection().resources_released());
    assert_eq!(cleanup.inspection().released_reservation_count(), 1);
}

#[test]
fn foreign_query_runtime_denies_before_resource_or_lower_basis_checks() {
    let owner = query_runtime();
    let foreign = query_runtime();
    let plan = admitted_plan("foreign-query-runtime", 8);
    let operation = direct_authority(&owner, &plan);
    let attempt = owner
        .start_direct_resource_attempt(&operation, plan)
        .expect("owner runtime should start its exact attempt");
    let lower = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );

    let rejection =
        match foreign.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("foreign Query runtime admitted another runtime's run"),
            Err(rejection) => rejection,
        };
    let denial = rejection.denial();
    assert_eq!(
        denial.kind(),
        WorthQueryManagedRunDenialKind::ForeignQueryRuntime
    );
    assert_eq!(denial.counters().query_runtime_check_count(), 1);
    assert_eq!(denial.counters().resource_attempt_check_count(), 0);
    assert_eq!(denial.counters().bridge_source_check_count(), 0);
}

#[test]
fn stale_installation_generation_denies_before_resource_or_lower_basis_checks() {
    let mut runtime = query_runtime();
    let plan = admitted_plan("stale-managed-run", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("current operation should start before successor installation");
    runtime
        .commit_successor_installation(Arc::new(
            runtime.installed_packages().successor_generation(),
        ))
        .expect("successor installation should commit");
    let lower = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );

    let rejection =
        match runtime.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("stale installed operation admitted a managed run"),
            Err(rejection) => rejection,
        };
    let denial = rejection.denial();
    assert_eq!(
        denial.kind(),
        WorthQueryManagedRunDenialKind::StaleInstallationGeneration
    );
    assert_eq!(denial.counters().resource_attempt_check_count(), 0);
}

#[test]
fn independently_valid_resource_attempt_cannot_substitute_for_the_operation() {
    let runtime = query_runtime();
    let operation_plan = admitted_plan("managed-operation-a", 8);
    let operation = direct_authority(&runtime, &operation_plan);
    let attempt_plan = admitted_plan("managed-operation-b", 8);
    let attempt_operation = direct_authority(&runtime, &attempt_plan);
    let attempt = runtime
        .start_direct_resource_attempt(&attempt_operation, attempt_plan)
        .expect("independently valid operation should start its own attempt");
    let lower = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );

    let rejection =
        match runtime.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("foreign resource attempt substituted for the operation"),
            Err(rejection) => rejection,
        };
    {
        let denial = rejection.denial();
        assert_eq!(
            denial.kind(),
            WorthQueryManagedRunDenialKind::ResourceAttemptMismatch
        );
        assert_eq!(denial.counters().resource_attempt_check_count(), 1);
        assert_eq!(denial.counters().bridge_source_check_count(), 0);
    }
    let returned_attempt = rejection.into_resource_attempt();
    assert_eq!(
        returned_attempt.binding_authority().binding_identity(),
        attempt_operation.binding_identity()
    );
    assert_eq!(
        returned_attempt
            .release()
            .capacity()
            .released_reservation_count(),
        1
    );
}

#[test]
fn bridge_attempt_for_a_different_run_intent_cannot_substitute() {
    let runtime = query_runtime();
    let plan = admitted_plan("bridge-intent-substitution", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact operation should start");
    let lower = causal_fixture::causal_lower_execution_basis(
        "independently-valid-operation",
        attempt.attempt_identity().as_str(),
    );

    let rejection =
        match runtime.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("Bridge authority for another run intent substituted"),
            Err(rejection) => rejection,
        };
    assert_eq!(
        rejection.denial().kind(),
        WorthQueryManagedRunDenialKind::BridgeManagedIntentMismatch
    );
    assert_eq!(rejection.denial().counters().bridge_intent_check_count(), 1);
    assert_eq!(rejection.denial().counters().bridge_source_check_count(), 0);
}

#[test]
fn independently_valid_relational_runtime_cannot_substitute_for_bridge_source() {
    let runtime = query_runtime();
    let plan = admitted_plan("foreign-relational-runtime", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact operation should start");
    let bridge_owner = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );
    let foreign_relational = causal_fixture::causal_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );

    let rejection = match runtime.admit_direct_run(
        &operation,
        attempt,
        bridge_owner.bridge,
        foreign_relational.relational,
    ) {
        Ok(_) => panic!("foreign Relational runtime substituted for Bridge source"),
        Err(rejection) => rejection,
    };
    let denial = rejection.denial();
    assert_eq!(
        denial.kind(),
        WorthQueryManagedRunDenialKind::ForeignRelationalRuntime
    );
    assert_eq!(denial.counters().bridge_source_check_count(), 1);
    assert_eq!(denial.counters().relational_basis_check_count(), 0);
}

#[test]
fn independently_valid_snapshot_lease_cannot_substitute_within_one_runtime() {
    let runtime = query_runtime();
    let plan = admitted_plan("mismatched-relational-snapshot", 8);
    let operation = direct_authority(&runtime, &plan);
    let attempt = runtime
        .start_direct_resource_attempt(&operation, plan)
        .expect("exact operation should start");
    let lower = causal_fixture::mismatched_snapshot_lower_execution_basis(
        operation.binding_identity(),
        attempt.attempt_identity().as_str(),
    );

    let rejection =
        match runtime.admit_direct_run(&operation, attempt, lower.bridge, lower.relational) {
            Ok(_) => panic!("different active snapshot lease substituted for Bridge truth"),
            Err(rejection) => rejection,
        };
    let denial = rejection.denial();
    assert_eq!(
        denial.kind(),
        WorthQueryManagedRunDenialKind::RelationalSnapshotMismatch
    );
    assert_eq!(denial.counters().relational_basis_check_count(), 1);
    assert_eq!(denial.counters().semantic_basis_check_count(), 0);
}
