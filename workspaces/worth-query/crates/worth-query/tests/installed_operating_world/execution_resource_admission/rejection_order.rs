use std::sync::atomic::Ordering;

use worth_proof::TransitionOutcome;
use worth_query::facade::installed;

use super::*;
use crate::suite::installed_operation_fixture::{
    artifact_move_workspace, bind_artifact_workflow, resource_admission_workspace, GeometryDomain,
    ReadExecutionInput, ReadFamily, ReadVertex,
};

#[test]
fn over_budget_direct_request_denies_before_session_or_executor_contact() {
    let contract_envelope = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("direct-chunk"),
    );
    let (workspace, contacts) = resource_admission_workspace(
        "resource-direct-rejection-order",
        contract([strategy("bounded", contract_envelope.clone())]),
        support(contract_envelope),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let request = request(
        installed::operation::WorthQuerySemanticScaleRequest::bounded(101),
        installed::operation::WorthQueryResourceLimitRequest::bounded(1),
        safe_point("direct-chunk"),
    );

    let TransitionOutcome::Denied(denial) =
        bound.admit_execution_resources(ReadExecutionInput::default(), request, &workspace)
    else {
        panic!("over-budget direct work must deny during resource admission")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::ResourceCeilingExceeded
    );
    assert_eq!(denial.counters().runtime_authority_checks, 1);
    assert_eq!(denial.counters().input_contract_checks, 1);
    assert_eq!(denial.counters().execution_contract_checks, 1);
    assert_eq!(denial.counters().resource_contract_lookups, 1);
    assert_eq!(denial.counters().support_snapshot_checks, 0);
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(contacts.load(Ordering::SeqCst), 0);
}

#[test]
fn over_budget_artifact_workflow_denies_before_artifact_allocation() {
    let (workspace, probe) = artifact_move_workspace("resource-artifact-rejection-order").unwrap();
    let bound = bind_artifact_workflow(&workspace);
    let request = installed::operation::WorthQueryExecutionResourceRequest::bounded(
        1_000_001,
        1,
        safe_point("fixture-chunk-boundary"),
    );

    let TransitionOutcome::Denied(denial) = bound.admit_workflow_resources(request, &workspace)
    else {
        panic!("over-budget artifact work must deny before workflow execution")
    };

    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::ResourceCeilingExceeded
    );
    assert_eq!(denial.counters().provider_session_mints, 0);
    assert_eq!(probe.allocations(), 0);
}

#[test]
fn named_semantic_axes_are_admitted_and_rejected_independently() {
    use installed::operation::WorthQuerySemanticScaleAxis as Axis;

    for (ordinal, axis) in [
        Axis::ModelSize,
        Axis::TouchedRegion,
        Axis::GraphValence,
        Axis::CandidateItems,
        Axis::OutputWidth,
        Axis::BatchWidth,
    ]
    .into_iter()
    .enumerate()
    {
        let contract_envelope = envelope(
            100,
            100,
            installed::operation::WorthQueryExecutionMode::Synchronous,
            None,
            safe_point("axis-chunk"),
        );
        let (workspace, contacts) = resource_admission_workspace(
            &format!("resource-axis-denial-{ordinal}"),
            contract([strategy("axis-bounded", contract_envelope.clone())]),
            support(contract_envelope),
        )
        .unwrap();
        let installed_domain = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed_domain, ReadVertex)
            .unwrap();
        let denied_request = request(
            installed::operation::WorthQuerySemanticScaleRequest::bounded(1).with(axis, 101),
            installed::operation::WorthQueryResourceLimitRequest::bounded(1),
            safe_point("axis-chunk"),
        );
        let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
            ReadExecutionInput::default(),
            denied_request,
            &workspace,
        ) else {
            panic!("{axis:?} alone must control its installed ceiling")
        };
        assert_eq!(
            denial.kind(),
            &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::ResourceCeilingExceeded
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 0);

        let contract_envelope = envelope(
            100,
            100,
            installed::operation::WorthQueryExecutionMode::Synchronous,
            None,
            safe_point("axis-chunk"),
        );
        let (workspace, _) = resource_admission_workspace(
            &format!("resource-axis-admit-{ordinal}"),
            contract([strategy("axis-bounded", contract_envelope.clone())]),
            support(contract_envelope),
        )
        .unwrap();
        let installed_domain = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed_domain, ReadVertex)
            .unwrap();
        let admitted = bound
            .admit_execution_resources(
                ReadExecutionInput::default(),
                request(
                    installed::operation::WorthQuerySemanticScaleRequest::bounded(1)
                        .with(axis, 100),
                    installed::operation::WorthQueryResourceLimitRequest::bounded(1),
                    safe_point("axis-chunk"),
                ),
                &workspace,
            )
            .unwrap();
        assert_eq!(admitted.resources().request().scale().get(axis), Some(100));
        for other in installed::operation::WorthQuerySemanticScaleAxis::ALL {
            if other != axis {
                assert_eq!(admitted.resources().request().scale().get(other), Some(1));
            }
        }
    }
}
