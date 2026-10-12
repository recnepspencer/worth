use super::*;

#[test]
fn runtime_state_snapshot_is_digest_bound_to_basis_shape_lane_and_state() {
    let ready = WorthQueryRuntimeStateSnapshot::ready(
        runtime_state_snapshot_basis_label_identity(&runtime_state_snapshot_test_subject_identity(
            "basis:current",
        )),
        runtime_state_snapshot_result_shape_label_identity(
            &runtime_state_snapshot_test_subject_identity("shape:table"),
        ),
        WorthQueryAuthorityLane::AuthoritativeTruth,
        "sync runtime-backed rows are ready",
    );
    let pending = WorthQueryRuntimeStateSnapshot::deferred(
        WorthQueryRuntimeStateKind::Pending,
        runtime_state_snapshot_basis_label_identity(&runtime_state_snapshot_test_subject_identity(
            "basis:current",
        )),
        runtime_state_snapshot_result_shape_label_identity(
            &runtime_state_snapshot_test_subject_identity("shape:table"),
        ),
        WorthQueryAuthorityLane::BridgeExternalState,
        "async/resource family is deferred",
    );

    assert_ne!(ready.state_digest(), pending.state_digest());
    assert!(pending.explanation().contains("deferred"));
}

#[test]
fn runtime_workspace_states_basis_lifecycle_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("task.basis-state-workspace")
        .expect("task runtime should open a named workspace");
    let current = crate::basis_lifecycle::basis_lifecycle()
        .current_head()
        .observe()
        .expect("current-head observation should admit");
    let branch = crate::basis_lifecycle::basis_lifecycle()
        .branch_head("branch:state-1", true)
        .observe()
        .expect("branch-head observation should admit");

    let current_state = workspace
        .state(request_execution, &current)
        .expect("basis capability should snapshot");
    let branch_state = workspace
        .state(request_execution, &branch)
        .expect("branch basis capability should snapshot");

    assert_eq!(current_state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(branch_state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(
        branch_state.authority_lane(),
        WorthQueryAuthorityLane::BranchLocalTruth
    );
}

#[test]
fn runtime_workspace_inspection_surfaces_basis_lifecycle_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime =
        stateful_bridge_task_runtime_with_domain(request_execution, test_runtime_basis_package());
    let installed = runtime
        .domain(TestRuntimeBasisDomain)
        .expect("test basis domain should be installed");
    let world_basis = installed
        .declarations(&runtime, TestRuntimeBasisContext)
        .expect("installed test basis context should admit")
        .retained_world_basis();
    let workspace = runtime
        .workspace("task.basis-inspection-workspace")
        .expect("task runtime should open a named workspace");
    let current = crate::basis_lifecycle::basis_lifecycle()
        .current_head()
        .observe()
        .expect("current-head observation should admit");
    let capability_inspection = workspace
        .inspect(request_execution, &current)
        .expect("basis capability should inspect");
    let world_inspection = workspace
        .inspect(request_execution, &world_basis)
        .expect("retained world basis should inspect");

    match capability_inspection {
        WorthQueryInspection::BasisLifecycle(inspection) => {
            assert_eq!(inspection.subject_label(), "scoped_observation_basis");
            assert_eq!(inspection.state_kind(), WorthQueryRuntimeStateKind::Ready);
            assert_eq!(
                inspection.authority_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
            assert_eq!(
                inspection
                    .family()
                    .expect("family should be present")
                    .as_str(),
                "current_head"
            );
        }
        other => panic!("expected basis lifecycle inspection, got {other:?}"),
    }

    match world_inspection {
        WorthQueryInspection::BasisLifecycle(inspection) => {
            assert_eq!(inspection.subject_label(), "admitted_world_basis");
            assert_eq!(inspection.state_kind(), WorthQueryRuntimeStateKind::Ready);
            assert_eq!(
                inspection.support_digest(),
                Some(world_basis.support_snapshot_digest())
            );
            assert_eq!(
                inspection.shape_digest(),
                runtime_state_snapshot_result_shape_label_identity(world_basis.handle_identity())
                    .as_str()
            );
        }
        other => panic!("expected admitted world basis inspection, got {other:?}"),
    }
}
