use worth_query::facade::domain;

use super::installed_operation_fixture::{
    canonical_bundle, conflicting_workspace, lowering_mismatch_workspace,
    mismatched_cost_workspace, mismatched_determinism_workspace, mismatched_read_plan_workspace,
    operation_identity_contract, semantic_drift_workspace, unsupported_direct_effect_workspace,
    workspace, GeometryDomain, ReadFamily, ReadVertex, ReadVertexLookalike,
};

struct DriftTrigger;
impl domain::WorthQueryOnDemandTriggerFamily for DriftTrigger {
    const PORTABLE_IDENTITY: &'static str = "worth.tests.triggers.operation-drift";
}

#[test]
fn declaration_order_converges_to_one_installed_operation_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let direct = workspace(request_execution, "operation-order-direct", false).unwrap();
    let reversed = workspace(request_execution, "operation-order-reversed", true).unwrap();
    let direct_receipt = direct.domain_installation_receipt(GeometryDomain).unwrap();
    let reversed_receipt = reversed
        .domain_installation_receipt(GeometryDomain)
        .unwrap();
    assert_eq!(
        direct_receipt.package_identity(),
        reversed_receipt.package_identity()
    );

    let direct_domain = direct.domain(GeometryDomain).unwrap();
    let reversed_domain = reversed.domain(GeometryDomain).unwrap();
    let direct_world = direct
        .observe_operating_world(direct.current_world())
        .unwrap();
    let reversed_world = reversed
        .observe_operating_world(reversed.current_world())
        .unwrap();
    let direct_operation = direct_world
        .family(ReadFamily)
        .bind(&direct_domain, ReadVertex)
        .unwrap();
    let reversed_operation = reversed_world
        .family(ReadFamily)
        .bind(&reversed_domain, ReadVertex)
        .unwrap();
    assert_eq!(
        direct_operation.definition(),
        reversed_operation.definition()
    );
    assert_eq!(
        (
            direct_operation.binding_counters().authority_checks,
            direct_operation.binding_counters().operation_lookups,
            direct_operation.binding_counters().graph_binding_lookups,
        ),
        (1, 1, 1)
    );
}

#[test]
fn executor_lowering_family_must_match_installed_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial = match lowering_mismatch_workspace(request_execution, "operation-lowering-mismatch")
    {
        Ok(_) => panic!("foreign lowering family must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor lowering family disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_precompiled_read_must_match_installed_canonical_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial =
        match mismatched_read_plan_workspace(request_execution, "operation-read-plan-mismatch") {
            Ok(_) => panic!("executor-authored semantic plan drift must not install"),
            Err(denial) => denial,
        };
    assert!(
        denial
            .message()
            .contains("executor read declaration disagrees with installed canonical semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_cost_must_match_installed_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial = match mismatched_cost_workspace(request_execution, "operation-cost-mismatch") {
        Ok(_) => panic!("executor cost drift must not install"),
        Err(denial) => denial,
    };
    assert!(
        denial
            .message()
            .contains("executor cost contract disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn executor_determinism_must_match_installed_semantics() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial =
        match mismatched_determinism_workspace(request_execution, "operation-determinism-mismatch")
        {
            Ok(_) => panic!("executor determinism drift must not install"),
            Err(denial) => denial,
        };
    assert!(
        denial
            .message()
            .contains("executor determinism disagrees with installed semantics"),
        "unexpected denial: {}",
        denial.message()
    );
}

#[test]
fn direct_primary_effect_requires_a_query_owned_execution_door() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial =
        match unsupported_direct_effect_workspace(request_execution, "unsupported-direct-effect") {
            Ok(_) => panic!("direct primary effect installed without an execution door"),
            Err(denial) => denial,
        };
    assert!(denial
        .message()
        .contains("direct operation declares effects without a Query-owned execution door"));
}

#[test]
fn rebuilt_execution_index_preserves_operation_authority_and_exact_shape() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = workspace(request_execution, "operation-index-rebuild", false).unwrap();
    let report = world.verify_domain_execution_index_rebuild();
    assert!(report.is_equivalent());
    assert_eq!(report.active_identity(), report.rebuilt_identity());
    assert_eq!(report.domain_operation_count(), 2);
    assert_eq!(report.operation_graph_participation_count(), 0);
    assert_eq!(report.operation_required_domain_count(), 0);

    let installed_domain = world.domain(GeometryDomain).unwrap();
    let operating_world = world
        .observe_operating_world(world.current_world())
        .unwrap();
    let operation = operating_world
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    assert_eq!(operation.definition().identity().name(), "read-vertex");
}

#[test]
fn one_field_semantic_drift_rejects_the_package_atomically() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial = match conflicting_workspace(request_execution, "operation-semantic-conflict") {
        Ok(_) => panic!("conflicting operation meaning must reject the package"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        worth_query::facade::consumer_kit::WorthQueryTestBackendErrorKind::DomainInstallationFailed
    );
    assert!(denial.message().contains("ConflictingDomainOperation"));
}

#[test]
fn marker_lookalike_misses_one_index_without_later_work() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = workspace(request_execution, "operation-marker-lookalike", false).unwrap();
    let domain = world.domain(GeometryDomain).unwrap();
    let operating_world = world
        .observe_operating_world(world.current_world())
        .unwrap();
    let denial = match operating_world
        .family(ReadFamily)
        .bind(&domain, ReadVertexLookalike)
    {
        Ok(_) => panic!("lookalike operation marker must not bind"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::OperationNotInstalled
    );
    assert_eq!(
        (
            denial.counters().authority_checks,
            denial.counters().operation_lookups,
            denial.counters().planning_steps,
        ),
        (1, 1, 0)
    );
}

#[test]
fn foreign_domain_authority_denies_before_operation_lookup() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let owner = workspace(request_execution, "operation-owner", false).unwrap();
    let foreign = workspace(request_execution, "operation-foreign", false).unwrap();
    let foreign_domain = foreign.domain(GeometryDomain).unwrap();
    let operating_world = owner
        .observe_operating_world(owner.current_world())
        .unwrap();
    let denial = match operating_world
        .family(ReadFamily)
        .bind(&foreign_domain, ReadVertex)
    {
        Ok(_) => panic!("foreign domain authority must not bind"),
        Err(denial) => denial,
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryOperationBindingDenialKind::DomainAuthority
    );
    assert_eq!(denial.counters().operation_lookups, 0);
    assert_eq!(denial.counters().planning_steps, 0);
}

#[path = "operation_definition/every_downstream_semantic_role_participates_in_atomic_installati.rs"]
mod every_downstream_semantic_role_participates_in_atomic_installati;
