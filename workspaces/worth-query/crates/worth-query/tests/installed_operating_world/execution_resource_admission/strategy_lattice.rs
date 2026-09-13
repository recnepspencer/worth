use worth_proof::TransitionOutcome;
use worth_query::facade::installed;

use super::*;
use crate::suite::conditional_node_contract::dependency;
use crate::suite::installed_operation_fixture::{
    conditional_workflow_workspace, execution_resource_request, resource_admission_workspace,
    GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex, WorkflowRead,
};

#[test]
fn asynchronous_strategy_and_live_capacity_shortfall_defer_with_distinct_causes() {
    let asynchronous = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Asynchronous,
        None,
        safe_point("async-chunk"),
    );
    let (workspace, _) = resource_admission_workspace(
        "resource-async-required",
        contract([strategy("async", asynchronous.clone())]),
        support(asynchronous),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let TransitionOutcome::Deferred(async_denial) = bound.admit_execution_resources(
        ReadExecutionInput::default(),
        request(
            installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
            installed::operation::WorthQueryResourceLimitRequest::bounded(1),
            safe_point("async-chunk"),
        ),
        &workspace,
    ) else {
        panic!("an async-only installed strategy must defer a sync-only request")
    };
    assert_eq!(
        async_denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::AsyncExecutionRequired
    );
    assert_eq!(async_denial.counters().provider_session_mints, 0);

    let installed_envelope = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("pressure-chunk"),
    );
    let live_support = envelope(
        10,
        10,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("pressure-chunk"),
    );
    let (workspace, _) = resource_admission_workspace(
        "resource-backpressured",
        contract([strategy("wide", installed_envelope)]),
        support(live_support),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let TransitionOutcome::Deferred(pressure_denial) = bound.admit_execution_resources(
        ReadExecutionInput::default(),
        request(
            installed::operation::WorthQuerySemanticScaleRequest::bounded(5),
            installed::operation::WorthQueryResourceLimitRequest::bounded(5),
            safe_point("pressure-chunk"),
        ),
        &workspace,
    ) else {
        panic!("matching capability with insufficient live capacity must backpressure")
    };
    assert_eq!(
        pressure_denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::Backpressured
    );
    assert_eq!(pressure_denial.counters().provider_session_mints, 0);
}

#[test]
fn named_degradation_is_explicit_and_cannot_satisfy_exact_support_silently() {
    let exact = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("degraded-chunk"),
    );
    let degraded = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        Some(installed::operation::WorthQueryExecutionDegradation::PartialResult),
        safe_point("degraded-chunk"),
    );
    let (workspace, _) = resource_admission_workspace(
        "resource-named-degradation",
        contract([
            strategy("exact", exact),
            strategy("retained-progress", degraded.clone()),
        ]),
        support(degraded),
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
                installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
                installed::operation::WorthQueryResourceLimitRequest::bounded(1),
                safe_point("degraded-chunk"),
            )
            .allow_degradation(installed::operation::WorthQueryExecutionDegradation::PartialResult),
            &workspace,
        )
        .unwrap();

    assert_eq!(
        admitted.resources().posture(),
        installed::operation::WorthQueryExecutionResourceAdmissionPosture::Degraded
    );
    assert_eq!(
        admitted.resources().strategy().as_str(),
        "retained-progress"
    );
    assert_eq!(
        admitted.resources().envelope().degradation(),
        Some(installed::operation::WorthQueryExecutionDegradation::PartialResult)
    );
    assert_eq!(admitted.resources().counters().support_snapshot_checks, 2);
}

#[test]
fn provider_access_allocator_and_safe_point_mismatches_never_fallback() {
    use installed::operation::WorthQueryExecutionResourceAdmissionDenialKind as Kind;

    for (ordinal, provider, access, allocator, expected) in [
        (
            0,
            "other-provider",
            ACCESS,
            ALLOCATOR,
            Kind::DifferentProviderRequired,
        ),
        (
            1,
            PROVIDER,
            "other-access",
            ALLOCATOR,
            Kind::DifferentAccessProductRequired,
        ),
        (
            2,
            PROVIDER,
            ACCESS,
            "other-arena",
            Kind::DifferentAllocatorRequired,
        ),
    ] {
        let installed_envelope = envelope(
            100,
            100,
            installed::operation::WorthQueryExecutionMode::Synchronous,
            None,
            safe_point("mismatch-chunk"),
        );
        let (workspace, _) = resource_admission_workspace(
            &format!("resource-provider-mismatch-{ordinal}"),
            contract([strategy("required", installed_envelope.clone())]),
            support_with_requirements(installed_envelope, provider, access, allocator),
        )
        .unwrap();
        let installed_domain = workspace.domain(GeometryDomain).unwrap();
        let bound = workspace
            .observe_operating_world(workspace.current_world())
            .unwrap()
            .family(ReadFamily)
            .bind(&installed_domain, ReadVertex)
            .unwrap();
        let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
            ReadExecutionInput::default(),
            request(
                installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
                installed::operation::WorthQueryResourceLimitRequest::bounded(1),
                safe_point("mismatch-chunk"),
            ),
            &workspace,
        ) else {
            panic!("provider requirement mismatch must deny")
        };
        assert_eq!(denial.kind(), &expected);
        assert_eq!(denial.counters().provider_session_mints, 0);
    }

    let installed_envelope = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("required-safe-point"),
    );
    let support_envelope = envelope(
        100,
        100,
        installed::operation::WorthQueryExecutionMode::Synchronous,
        None,
        safe_point("other-safe-point"),
    );
    let (workspace, _) = resource_admission_workspace(
        "resource-safe-point-mismatch",
        contract([strategy("required", installed_envelope)]),
        support(support_envelope),
    )
    .unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let bound = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, ReadVertex)
        .unwrap();
    let TransitionOutcome::Denied(denial) = bound.admit_execution_resources(
        ReadExecutionInput::default(),
        request(
            installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
            installed::operation::WorthQueryResourceLimitRequest::bounded(1),
            safe_point("required-safe-point"),
        ),
        &workspace,
    ) else {
        panic!("a provider safe-point mismatch must deny")
    };
    assert_eq!(denial.kind(), &Kind::CancellationSafePointUnsupported);
}

#[test]
fn workflow_operation_and_stage_support_snapshots_include_only_causal_graph_roles() {
    let dependency = dependency(domain::WorthQuerySemanticLocality::SourceRecord);
    let conditional = domain::WorthQueryPortableConditionalNodeDeclaration::declare(
        "resource-stage-local-support",
        domain::WorthQueryConditionalNodeRole::WorkflowStage,
    )
    .dependencies([dependency.clone()])
    .outputs([
        domain::WorthQueryConditionalNodeOutput::WorkflowStageOutput {
            contract: domain::WorthQueryWorkflowValueContract::Projection,
        },
    ])
    .required_context([domain::WorthQueryConditionalNodeContext::WorkflowRun])
    .evaluation(
        domain::WorthQueryConditionalEvaluationCondition::aspect_filtered([dependency]).unwrap(),
        domain::WorthQueryConditionalTrigger::DependencyChange,
    )
    .comparison(
        domain::WorthQueryComparatorRequirement::ExactCanonicalValue,
        domain::WorthQueryOutputEquivalenceRequirement::ExactCanonicalValue,
    )
    .artifact_policy(
        domain::WorthQueryArtifactReuseEquivalence::NotReusable,
        domain::WorthQueryMaintenancePosture::LazyUntilObserved,
        domain::WorthQueryArtifactPosture::Ephemeral,
    )
    .output_relationship(domain::WorthQueryOutputRelationship::IsWorkflowStageOutput)
    .finish()
    .unwrap();
    let workspace =
        conditional_workflow_workspace("resource-stage-local-support", conditional).unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let admitted = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(execution_resource_request(), &workspace)
        .unwrap();

    assert!(admitted
        .resources()
        .operation()
        .support_snapshot()
        .graph_providers()
        .is_empty());
    assert!(admitted
        .resources()
        .stage("start")
        .unwrap()
        .support_snapshot()
        .graph_providers()
        .is_empty());
    let publish = admitted.resources().stage("publish").unwrap();
    assert_eq!(publish.support_snapshot().graph_providers().len(), 1);
    assert_eq!(publish.support_snapshot().graph_providers()[0].0, "model");
}
