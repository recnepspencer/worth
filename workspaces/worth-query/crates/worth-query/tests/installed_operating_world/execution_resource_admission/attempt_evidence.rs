use std::sync::atomic::Ordering;

use worth_query::facade::{domain, installed};

use super::*;
use crate::suite::installed_operation_fixture::{
    execution_resource_request, resource_admission_workspace, workflow_workspace, GeometryDomain,
    ReadExecutionInput, ReadFamily, ReadVertex, WorkflowRead,
};

#[test]
fn changed_request_mints_a_new_plan_and_session_without_mutating_prior_admission() {
    let installed_envelope = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("identity-chunk"),
    );
    let (mut workspace, contacts) = resource_admission_workspace(
        "resource-attempt-identity",
        contract([strategy("bounded", installed_envelope.clone())]),
        support(installed_envelope),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let first_bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let second_bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let first = first_bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            request(
                installed::operation::WorthQuerySemanticScaleRequest::bounded(5),
                installed::operation::WorthQueryResourceLimitRequest::bounded(5),
                safe_point("identity-chunk"),
            ),
            &workspace,
        )
        .unwrap();
    let second = second_bound
        .admit_execution_resources(
            ReadExecutionInput::default(),
            request(
                installed::operation::WorthQuerySemanticScaleRequest::bounded(5).with(
                    installed::operation::WorthQuerySemanticScaleAxis::TouchedRegion,
                    6,
                ),
                installed::operation::WorthQueryResourceLimitRequest::bounded(5),
                safe_point("identity-chunk"),
            ),
            &workspace,
        )
        .unwrap();

    assert_ne!(
        first.resources().request_identity(),
        second.resources().request_identity()
    );
    assert_ne!(first.resources().identity(), second.resources().identity());
    assert_eq!(
        first.resources().envelope_identity(),
        second.resources().envelope_identity()
    );
    assert_ne!(
        first.provider_session_identity(),
        second.provider_session_identity()
    );
    assert_ne!(
        first.provider_session_attempt_identity(),
        second.provider_session_attempt_identity()
    );
    assert_eq!(
        first
            .resources()
            .request()
            .scale()
            .get(installed::operation::WorthQuerySemanticScaleAxis::TouchedRegion),
        Some(5)
    );
    assert_eq!(
        second
            .resources()
            .request()
            .scale()
            .get(installed::operation::WorthQuerySemanticScaleAxis::TouchedRegion),
        Some(6)
    );

    let first_plan_identity = first.resources().identity().to_owned();
    let first_request_identity = first.resources().request_identity().to_owned();
    let first_session_identity = first.provider_session_identity().to_owned();
    let first_attempt_identity = first.provider_session_attempt_identity().to_owned();
    let executed = first.execute(&mut workspace).unwrap();
    let evidence = executed.receipt().execution_resources();
    assert_eq!(executed.resources().identity(), first_plan_identity);
    assert_eq!(executed.provider_session_identity(), first_session_identity);
    assert_eq!(evidence.admission_identity(), first_plan_identity);
    assert_eq!(evidence.request_identity(), first_request_identity);
    assert_eq!(evidence.provider_session_identity(), first_session_identity);
    assert_eq!(
        evidence.provider_session_attempt_identity(),
        first_attempt_identity
    );
    assert_ne!(
        evidence.provider_session_attempt_identity(),
        executed.resources().identity()
    );
    assert_eq!(contacts.load(Ordering::SeqCst), 1);
}

#[test]
fn workflow_stage_receipts_retain_stage_local_plans_and_one_attempt_session() {
    let mut workspace = workflow_workspace("resource-workflow-attempt-evidence").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let admitted = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(execution_resource_request(), &workspace)
        .unwrap();
    let session_identity = admitted.provider_session_identity().to_owned();
    let run = admitted.start_workflow(&mut workspace).unwrap();
    assert_eq!(
        run.operation_resource_evidence()
            .provider_session_identity(),
        session_identity
    );
    let trace = run
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance(
            "left",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "right",
            domain::WorthQueryWorkflowValue::Text("start".into()),
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap();

    assert_eq!(trace.resources().counters().runtime_authority_checks, 1);
    assert_eq!(trace.resources().counters().provider_session_mints, 1);
    assert_eq!(trace.resources().counters().resource_contract_lookups, 5);
    assert_eq!(
        trace
            .resources()
            .operation()
            .counters()
            .runtime_authority_checks,
        1
    );
    for (_, plan) in trace.resources().stages() {
        assert_eq!(plan.counters().runtime_authority_checks, 0);
        assert_eq!(plan.counters().provider_session_mints, 0);
    }
    for receipt in trace.stage_receipts() {
        let plan = trace.resources().stage(receipt.stage_identity()).unwrap();
        let evidence = receipt.execution_resources();
        assert_eq!(evidence.admission_identity(), plan.identity());
        assert_eq!(evidence.request_identity(), plan.request_identity());
        assert_eq!(evidence.envelope_identity(), plan.envelope_identity());
        assert_eq!(evidence.provider_session_identity(), session_identity);
    }
}
