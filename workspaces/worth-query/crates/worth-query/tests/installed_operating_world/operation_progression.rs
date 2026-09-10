use worth_proof::TransitionOutcome;
use worth_query::facade::{domain, installed, read};

use super::installed_operation_fixture::{
    configured_runtime, missing_read_execution_workspace, workspace, CountVertices,
    CountVerticesInput, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

mod failure_boundaries;

#[test]
fn public_bound_execution_projection_and_settlement_remain_one_chain() {
    let mut workspace = workspace("installed-progression", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    assert_eq!(
        bound.binding_counters(),
        domain::WorthQueryOperationBindingCounters {
            authority_checks: 1,
            operation_lookups: 1,
            required_domain_lookups: 0,
            graph_binding_lookups: 1,
            graph_participation_lookups: 0,
            graph_provider_contacts: 0,
            conditional_lowering_lookups: 1,
            conditional_lowerings_retained: 0,
            conditional_declarations_inspected: 0,
            conditional_workflow_stages_inspected: 0,
            conditional_lowering_checks: 0,
            graph_contract_checks: 0,
            graph_read_role_checks: 0,
            touched_graph_role_checks: 0,
            commit_graph_checks: 0,
            commit_authority_checks: 0,
            planning_steps: 1,
            authority_shape_admissions: 1,
            execution_authority_admissions: 1,
            commit_posture_classifications: 1,
            executor_route_lookups: 1,
            workflow_executor_route_lookups: 1,
            parallel_admission_route_lookups: 1,
        }
    );
    let binding_identity = bound.binding_identity().to_string();
    let consumer = bound.consumer_projection_contract().unwrap();

    let executed = bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(executed.receipt().binding_identity(), binding_identity);
    assert_eq!(executed.counters().executor_contacts, 1);
    let settled = executed
        .publish()
        .unwrap()
        .consume(consumer, read::project_facts().entity_identities())
        .unwrap()
        .settle()
        .unwrap();

    assert_eq!(
        settled.execution_receipt().binding_identity(),
        binding_identity
    );
    assert_eq!(
        settled.result_state(),
        domain::WorthQueryOperationResultState::Ready
    );
    assert_eq!(settled.counters().consumption_contacts, 1);
    assert!(!settled.publication_receipt().identity().is_empty());
    assert!(!settled.identity().is_empty());
}

#[test]
fn non_publishing_execution_is_a_terminal_typed_outcome() {
    let mut workspace = workspace("installed-terminal", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, CountVertices)
        .unwrap();
    let executed = bound
        .admit_execution_resources(
            CountVerticesInput { minimum: Some(0) },
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap();
    assert_eq!(*executed.output(), 1);
    assert_eq!(executed.receipt().output_identity(), "u64:1");
    assert_eq!(executed.counters().primary_read_contacts, 1);
    assert_eq!(executed.counters().publication_checks, 0);
}

#[test]
fn installed_parameter_contract_denies_before_graph_or_executor_work() {
    let workspace = workspace("installed-parameter-denial", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, CountVertices)
        .unwrap();
    let denial = match bound.admit_execution_resources(
        CountVerticesInput { minimum: None },
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &workspace,
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("missing required operation parameter did not produce an exact denial"),
    };
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::InputContract
    );
    assert_eq!(denial.counters().input_contract_checks, 1);
    assert_eq!(denial.counters().execution_contract_checks, 0);
    assert_eq!(denial.counters().resource_contract_lookups, 0);
    assert_eq!(denial.counters().support_snapshot_checks, 0);
    assert_eq!(denial.counters().provider_session_mints, 0);
}

#[test]
fn declared_primary_read_cannot_be_skipped_by_a_terminal_executor() {
    let mut workspace = missing_read_execution_workspace("installed-skipped-read").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, CountVertices)
        .unwrap();
    let denial = match bound
        .admit_execution_resources(
            CountVerticesInput { minimum: Some(0) },
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
    {
        TransitionOutcome::Failed(denial) => denial,
        _ => panic!("skipped primary read did not produce an execution failure"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryBoundExecutionDenialKind::UndeclaredFailureClass(
            domain::WorthQueryOperationFailureClass::Indeterminate,
        )
    );
    assert_eq!(denial.counters().primary_read_contacts, 0);
}

#[test]
fn foreign_workspace_denies_before_graph_or_executor_work() {
    let owner = workspace("installed-execution-owner", false).unwrap();
    let installed_domain = owner.domain(GeometryDomain).unwrap();
    let bound = owner
        .observe_operating_world(owner.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let foreign = workspace("installed-execution-foreign", false).unwrap();
    let denial = match bound.admit_execution_resources(
        ReadExecutionInput::default(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &foreign,
    ) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("foreign runtime did not produce an exact denial"),
    };
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::RuntimeAuthority(
            domain::WorthQueryDomainHandleDenialKind::ForeignRuntime,
        )
    );
    assert_eq!(denial.counters().runtime_authority_checks, 1);
    assert_eq!(denial.counters().input_contract_checks, 0);
    assert_eq!(denial.counters().resource_contract_lookups, 0);
    assert_eq!(denial.counters().provider_session_mints, 0);
}

#[test]
fn admitted_direct_attempt_becomes_stale_before_any_execution_work() {
    let mut workspace = configured_runtime()
        .controlled_workspace("installed-post-admission-stale")
        .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let admitted = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap()
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap();

    workspace.advance_domain_installation_generation().unwrap();

    let denial = match admitted.execute(&mut workspace) {
        TransitionOutcome::Stale(denial) => denial,
        _ => panic!("admitted attempt crossed an installation generation"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryBoundExecutionDenialKind::RuntimeAuthority(
            domain::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
        )
    );
    assert_eq!(denial.counters().runtime_authority_checks, 1);
    assert_eq!(denial.counters().graph_provider_contacts, 0);
    assert_eq!(denial.counters().executor_contacts, 0);
    assert_eq!(denial.counters().conditional_request_admission_checks, 0);
}

#[test]
fn equivalent_but_distinct_bound_contract_cannot_splice_the_chain() {
    let mut workspace = workspace("installed-progression-splice", false).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let first = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let second = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    assert_eq!(first.binding_identity(), second.binding_identity());
    let foreign_contract = second.consumer_projection_contract().unwrap();
    let published = first
        .admit_execution_resources(
            ReadExecutionInput::default(),
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .execute(&mut workspace)
        .unwrap()
        .publish()
        .unwrap();
    assert!(matches!(
        published.consume(foreign_contract, read::project_facts().entity_identities()),
        TransitionOutcome::Denied(domain::WorthQueryProgressionDenial::ConsumerContractMismatch)
    ));
}

#[test]
fn result_state_and_warning_postures_survive_through_settlement() {
    let states = [
        domain::WorthQueryOperationResultState::Ready,
        domain::WorthQueryOperationResultState::Advisory,
        domain::WorthQueryOperationResultState::Pending,
        domain::WorthQueryOperationResultState::Partial,
        domain::WorthQueryOperationResultState::Violation,
    ];
    for (index, state) in states.into_iter().enumerate() {
        let mut workspace = workspace(&format!("installed-posture-{index}"), false).unwrap();
        let installed_domain = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed_domain, ReadVertex)
            .unwrap();
        let consumer = bound.consumer_projection_contract().unwrap();
        let warning =
            domain::WorthQueryOperationExecutionWarning::Advisory(format!("posture-{index}"));
        let settled = bound
            .admit_execution_resources(
                ReadExecutionInput {
                    state,
                    warning: Some(warning.clone()),
                    failure: None,
                },
                crate::suite::installed_operation_fixture::execution_resource_request(),
                &workspace,
            )
            .unwrap()
            .execute(&mut workspace)
            .unwrap()
            .publish()
            .unwrap()
            .consume(consumer, read::project_facts().entity_identities())
            .unwrap()
            .settle()
            .unwrap();

        assert_eq!(settled.result_state(), state);
        assert_eq!(settled.warnings(), [warning]);
    }
}
