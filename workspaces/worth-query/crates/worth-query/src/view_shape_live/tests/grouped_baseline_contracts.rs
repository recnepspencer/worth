use crate::basis::{
    resolve_snapshot_basis, BasisAuthorityFamily, BasisResolutionMode, ExecutionBasisIntent,
    ResolvedSnapshotIdentity, SnapshotLineageClass,
};
use crate::view_shape::ViewShapeDescriptor;
use worth_foundational::facade::prepare_aspect_value_identity_basis;

use super::super::{
    materialize_authoritative_grouped_baseline,
    materialize_grouped_execution_surface_from_truth_view, ViewShapeLiveFailureClass,
};
use super::grouped_truth_projection::{
    aspect_key, grouped_truth_view, grouped_truth_view_with_rows,
};
use super::grouped_truth_world::grouped_row;
use super::view_plan_world::{collection_canonical, planned_view, runtime_basis};

#[test]
fn grouped_baseline_is_derived_from_authoritative_execution_bindings() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let planned = planned_view(
        &collection_canonical(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(planned.validated().query().schema_basis().clone());
    let truth_view = grouped_truth_view(&planned, resource_request);
    let grouped_execution =
        materialize_grouped_execution_surface_from_truth_view(&planned, basis.clone(), &truth_view)
            .unwrap();
    let baseline =
        materialize_authoritative_grouped_baseline(&planned, basis, &grouped_execution).unwrap();

    assert_eq!(
        grouped_execution
            .grouped_planning()
            .native_grouping_aspect_key(),
        &aspect_key("status")
    );
    assert_eq!(grouped_execution.member_rows().len(), 2);
    assert_eq!(
        grouped_execution.member_rows()[0]
            .lane()
            .native_grouping_aspect_key(),
        &aspect_key("status")
    );
    assert_eq!(
        grouped_execution.truth_view_evidence_identity().as_str(),
        crate::view_shape_live::grouped_execution::bridge_grouped_truth_view_digest_evidence_identity(
            truth_view.digest(),
        )
        .as_str()
    );
    assert_eq!(baseline.desired_state().result().row_count(), 2);
    assert_eq!(baseline.desired_state().result().lane_count(), 2);
    let expected_member_key = prepare_aspect_value_identity_basis(
        &crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("task-1"),
    );
    let expected_lane_key = prepare_aspect_value_identity_basis(
        &crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value("todo"),
    );
    assert_eq!(
        baseline.desired_state().result().member_states()[0].member_key(),
        expected_member_key.as_str()
    );
    assert_eq!(
        baseline.desired_state().result().member_states()[0]
            .lane()
            .lane_key(),
        expected_lane_key.as_str()
    );
}

#[test]
fn grouped_baseline_rejects_mismatched_grouped_execution_surface() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let planned = planned_view(
        &collection_canonical(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(planned.validated().query().schema_basis().clone());
    let truth_view = grouped_truth_view(&planned, resource_request);
    let grouped_execution =
        materialize_grouped_execution_surface_from_truth_view(&planned, basis.clone(), &truth_view)
            .unwrap();
    let other_plan = planned_view(
        &collection_canonical(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("profile")),
    );
    let error = materialize_authoritative_grouped_baseline(&other_plan, basis, &grouped_execution)
        .unwrap_err();

    assert_eq!(
        error.failure_class(),
        &ViewShapeLiveFailureClass::GroupedBaselineMismatch
    );
}

#[test]
fn grouped_execution_rejects_truth_view_with_mismatched_identity_binding() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let planned = planned_view(
        &collection_canonical(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let basis = runtime_basis(planned.validated().query().schema_basis().clone());
    let wrong_truth_view = grouped_truth_view_with_rows(
        &planned,
        &[grouped_row("task-1", "Ada", "todo")],
        "profile.display_name",
        None,
        resource_request,
    );

    let error =
        materialize_grouped_execution_surface_from_truth_view(&planned, basis, &wrong_truth_view)
            .unwrap_err();

    assert_eq!(
        error.failure_class(),
        &ViewShapeLiveFailureClass::GroupedBaselineMismatch
    );
}

#[test]
fn grouped_execution_rejects_truth_view_with_mismatched_snapshot_identity() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let planned = planned_view(
        &collection_canonical(),
        ViewShapeDescriptor::kanban_grouped(aspect_key("status")),
    );
    let wrong_basis = resolve_snapshot_basis(
        ExecutionBasisIntent::new(
            BasisAuthorityFamily::Runtime,
            SnapshotLineageClass::CurrentHead,
            false,
        ),
        ResolvedSnapshotIdentity::new(
            BasisAuthorityFamily::Runtime,
            None,
            crate::memory_workspace::admit_external_snapshot_label("snapshot-b")
                .evidence_identity(),
            planned.validated().query().schema_basis().clone(),
            SnapshotLineageClass::CurrentHead,
        ),
        BasisResolutionMode::RuntimeDirect,
    )
    .unwrap();
    let truth_view = grouped_truth_view(&planned, resource_request);

    let error =
        materialize_grouped_execution_surface_from_truth_view(&planned, wrong_basis, &truth_view)
            .unwrap_err();

    assert_eq!(
        error.failure_class(),
        &ViewShapeLiveFailureClass::GroupedBaselineMismatch
    );
    assert!(error.message().contains("snapshot"));
}
