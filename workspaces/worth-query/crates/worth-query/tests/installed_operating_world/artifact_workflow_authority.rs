use worth_proof::TransitionOutcome;
use worth_query::facade::domain;

use super::installed_operation_fixture::{
    artifact_controlled_workspace, artifact_move_workspace, artifact_workspace_without_support,
    bind_artifact_workflow, move_intent, GeometryDomain, ReadFamily, WorkflowRead,
};

#[test]
fn artifact_package_requires_explicit_runtime_version_support() {
    let error = match artifact_workspace_without_support("artifact-support-denial") {
        Ok(_) => panic!("artifact contract without runtime version support installed"),
        Err(error) => error,
    };
    assert!(error.message().contains("UnsupportedArtifactVersion"));
    assert!(error
        .message()
        .contains("WORTH.tests.artifact-workflow.candidates:1:1"));
}

#[test]
fn foreign_provider_is_denied_and_its_resource_is_disposed_exactly_once() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-provider-denial").unwrap();
    let outcome = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("reject-provider"), &mut workspace);

    assert!(matches!(
        outcome,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(
        probe.denials(),
        vec![domain::WorthQueryArtifactDenialKind::ProviderFamilyMismatch]
    );
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 1);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn retained_production_admission_denies_in_a_later_run_before_provider_projection() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-retained-admission").unwrap();
    let retained = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("retain-admission"), &mut workspace);
    assert!(matches!(
        retained,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));

    let rejected = bind_artifact_workflow(&workspace)
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(move_intent("reuse-retained-admission"), &mut workspace);
    assert!(matches!(
        rejected,
        TransitionOutcome::Denied(_) | TransitionOutcome::Failed(_)
    ));
    assert_eq!(
        probe.denials(),
        vec![domain::WorthQueryArtifactDenialKind::RunMismatch]
    );
    assert_eq!(probe.allocations(), 1);
    assert_eq!(probe.projection_calls(), 0);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn stale_installation_generation_denies_artifact_transfer_before_consumer_access() {
    let (mut workspace, probe) =
        artifact_controlled_workspace("artifact-stale-generation").unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "produce",
            domain::WorthQueryWorkflowValue::Text("produce".into()),
            &mut workspace,
        )
        .unwrap();
    workspace.advance_domain_installation_generation().unwrap();

    let denial = match run.advance_with_artifact("consume", "produce", &mut workspace) {
        TransitionOutcome::Stale(denial) => denial,
        _ => panic!("stale artifact transfer did not return stale authority evidence"),
    };
    assert!(matches!(
        denial.kind(),
        domain::WorthQueryWorkflowAdvanceDenialKind::RuntimeAuthority(
            domain::WorthQueryDomainHandleDenialKind::StaleInstallationGeneration,
        )
    ));
    assert_eq!(denial.counters().stage_executor_contacts, 1);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn foreign_runtime_denies_artifact_progression_before_consumer_access() {
    let (mut owner, probe) = artifact_move_workspace("artifact-runtime-owner").unwrap();
    let installed = owner.domain(GeometryDomain).unwrap();
    let run = owner
        .observe_operating_world(owner.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &owner,
        )
        .unwrap()
        .start_workflow(&mut owner)
        .unwrap()
        .advance(
            "produce",
            domain::WorthQueryWorkflowValue::Text("retain-observer-lease".into()),
            &mut owner,
        )
        .unwrap();
    let (mut foreign, foreign_probe) = artifact_move_workspace("artifact-runtime-foreign").unwrap();

    let denial = match run.advance_with_artifact("consume", "produce", &mut foreign) {
        TransitionOutcome::Denied(denial) => denial,
        _ => panic!("foreign runtime did not deny artifact progression"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::RuntimeAuthority(
            domain::WorthQueryDomainHandleDenialKind::ForeignRuntime,
        )
    );
    assert_eq!(denial.counters().stage_executor_contacts, 1);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 1);
    assert_eq!(foreign_probe.allocations(), 0);
    let lease = probe
        .take_escaped_lease()
        .expect("producer retained an observer lease");
    let snapshot = lease.owner_snapshot();
    assert_eq!(snapshot.owner_count(), 0);
    assert_eq!(snapshot.lease_count(), 0);
    assert_eq!(snapshot.lifecycle_generation(), 3);
    assert_eq!(snapshot.counters().transfer_admissions, 0);
    assert_eq!(snapshot.counters().provider_disposals, 1);
    assert!(snapshot.is_disposed());
    drop(lease);
    assert_eq!(probe.disposals(), 1);
}

#[test]
fn undeclared_predecessor_denies_before_artifact_transfer_or_consumer_access() {
    let (mut workspace, probe) = artifact_move_workspace("artifact-stage-mismatch").unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "produce",
            domain::WorthQueryWorkflowValue::Text("produce".into()),
            &mut workspace,
        )
        .unwrap();

    let denial = match run.advance_with_artifact("consume", "other-producer", &mut workspace) {
        TransitionOutcome::Failed(denial) => denial,
        _ => panic!("undeclared predecessor did not deny artifact transfer"),
    };
    assert_eq!(
        denial.kind(),
        &domain::WorthQueryWorkflowAdvanceDenialKind::PredecessorAuthorityMissing(
            "other-producer".into(),
        )
    );
    assert_eq!(denial.counters().stage_executor_contacts, 1);
    assert_eq!(probe.borrow_observations(), 0);
    assert_eq!(probe.disposals(), 1);
}
