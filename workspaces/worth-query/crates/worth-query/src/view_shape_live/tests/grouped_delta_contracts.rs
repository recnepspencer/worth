use crate::live::{BridgeChangeSummary, BridgeFieldDelta};
use crate::view_shape::ViewShapeDescriptor;

use super::super::{
    admit_grouped_live_view, execute_grouped_live_view_shape_change, lower_view_shape_plan_to_live,
    materialize_authoritative_grouped_baseline,
    materialize_grouped_execution_surface_from_truth_view, ViewShapePatchFamily,
    ViewShapePatchPayload,
};
use super::grouped_truth_projection::{
    aspect_key, assert_grouped_delta_counters_are_debt_free, grouped_truth_view,
    grouped_truth_view_with_rows,
};
use super::grouped_truth_world::grouped_row;
use super::view_plan_world::{collection_canonical, planned_view, runtime_basis};

#[test]
fn grouped_delta_is_explicit_and_deterministic() {
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
        materialize_authoritative_grouped_baseline(&planned, basis.clone(), &grouped_execution)
            .unwrap();
    let member_key = baseline.desired_state().result().member_states()[0]
        .member_key()
        .to_string();
    let live = lower_view_shape_plan_to_live(&planned, basis, Some(baseline), None).unwrap();
    let change = BridgeChangeSummary::default()
        .with_field_delta(BridgeFieldDelta::new(
            "identity",
            "id",
            Some(member_key.as_str()),
            Some(member_key.as_str()),
        ))
        .with_field_delta(BridgeFieldDelta::new(
            "status",
            "lane",
            Some("todo"),
            Some("doing"),
        ))
        .with_membership_transition(true, true);
    let next_truth_view = grouped_truth_view_with_rows(
        &planned,
        &[
            grouped_row("task-1", "Ada", "doing"),
            grouped_row("task-2", "Bea", "doing"),
        ],
        "identity.id",
        None,
        resource_request,
    );
    let next_grouped_execution = materialize_grouped_execution_surface_from_truth_view(
        &planned,
        live.basis().clone(),
        &next_truth_view,
    )
    .unwrap();

    let grouped_live = admit_grouped_live_view(&live).unwrap();
    let first =
        execute_grouped_live_view_shape_change(grouped_live, &change, &next_grouped_execution)
            .unwrap();
    let second =
        execute_grouped_live_view_shape_change(grouped_live, &change, &next_grouped_execution)
            .unwrap();

    match first.patch_envelope().payload() {
        ViewShapePatchPayload::KanbanGroupMembershipPatch(delta) => {
            assert_eq!(
                delta.digest(),
                match second.patch_envelope().payload() {
                    ViewShapePatchPayload::KanbanGroupMembershipPatch(second_delta) =>
                        second_delta.digest(),
                    other => panic!("expected grouped delta payload, got {other:?}"),
                }
            );
            assert_eq!(delta.transitions().len(), 1);
            assert_eq!(delta.prior().result().lane_count(), 2);
            assert_eq!(delta.next().result().lane_count(), 1);
            assert_eq!(delta.next().result().row_count(), 2);
        }
        other => panic!("expected grouped delta payload, got {other:?}"),
    }
}

#[test]
fn grouped_churn_overrun_stays_on_grouped_membership_delta() {
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
        materialize_authoritative_grouped_baseline(&planned, basis.clone(), &grouped_execution)
            .unwrap();
    let member_key = baseline.desired_state().result().member_states()[0]
        .member_key()
        .to_string();
    let live = lower_view_shape_plan_to_live(&planned, basis, Some(baseline), None).unwrap();
    let next_truth_view = grouped_truth_view_with_rows(
        &planned,
        &[
            grouped_row("task-1", "Ada", "doing"),
            grouped_row("task-2", "Bea", "done"),
        ],
        "identity.id",
        None,
        resource_request,
    );
    let next_grouped_execution = materialize_grouped_execution_surface_from_truth_view(
        &planned,
        live.basis().clone(),
        &next_truth_view,
    )
    .unwrap();
    let grouped_live = admit_grouped_live_view(&live).unwrap();
    let execution = execute_grouped_live_view_shape_change(
        grouped_live,
        &BridgeChangeSummary::default()
            .with_field_delta(BridgeFieldDelta::new(
                "identity",
                "id",
                Some(member_key.as_str()),
                Some(member_key.as_str()),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "status",
                "lane",
                Some("todo"),
                Some("doing"),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "status",
                "lane",
                Some("doing"),
                Some("done"),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "profile",
                "display_name",
                Some("Ada"),
                Some("Ada Lovelace"),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "identity",
                "id",
                Some(member_key.as_str()),
                Some(member_key.as_str()),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "meta",
                "priority",
                Some("p1"),
                Some("p2"),
            )),
        &next_grouped_execution,
    )
    .unwrap();

    assert_eq!(
        execution.patch_envelope().patch_family(),
        Some(ViewShapePatchFamily::KanbanGroupMembershipPatch)
    );
    match execution.patch_envelope().payload() {
        ViewShapePatchPayload::KanbanGroupMembershipPatch(delta) => {
            assert_eq!(delta.transitions().len(), 2);
            assert_eq!(delta.next().result().lane_count(), 2);
        }
        other => panic!("expected grouped delta payload, got {other:?}"),
    }
    assert_grouped_delta_counters_are_debt_free(&execution, 2, 2);
}

#[test]
fn grouped_core_refresh_still_emits_grouped_semantics() {
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
        materialize_authoritative_grouped_baseline(&planned, basis.clone(), &grouped_execution)
            .unwrap();
    let live = lower_view_shape_plan_to_live(&planned, basis, Some(baseline), None).unwrap();
    let next_truth_view = grouped_truth_view(&planned, resource_request);
    let next_grouped_execution = materialize_grouped_execution_surface_from_truth_view(
        &planned,
        live.basis().clone(),
        &next_truth_view,
    )
    .unwrap();
    let mut change = BridgeChangeSummary::default();
    for _ in 0..128 {
        change = change.with_field_delta(BridgeFieldDelta::new(
            "profile",
            "display_name",
            Some("Ada"),
            Some("Ada Lovelace"),
        ));
    }

    let execution = execute_grouped_live_view_shape_change(
        admit_grouped_live_view(&live).unwrap(),
        &change,
        &next_grouped_execution,
    )
    .unwrap();

    assert_eq!(
        execution.patch_envelope().patch_family(),
        Some(ViewShapePatchFamily::KanbanGroupMembershipPatch)
    );
    match execution.patch_envelope().payload() {
        ViewShapePatchPayload::KanbanGroupMembershipPatch(delta) => {
            assert!(delta.transitions().is_empty());
            assert_eq!(delta.next().result().lane_count(), 2);
        }
        other => panic!("expected grouped delta payload, got {other:?}"),
    }
    assert_grouped_delta_counters_are_debt_free(&execution, 0, 2);
}

#[test]
fn grouped_delta_mixed_member_churn_stays_incremental() {
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
        materialize_authoritative_grouped_baseline(&planned, basis.clone(), &grouped_execution)
            .unwrap();
    let live = lower_view_shape_plan_to_live(&planned, basis, Some(baseline), None).unwrap();
    let next_truth_view = grouped_truth_view_with_rows(
        &planned,
        &[
            grouped_row("task-1", "Ada", "doing"),
            grouped_row("task-3", "Cy", "todo"),
        ],
        "identity.id",
        None,
        resource_request,
    );
    let next_grouped_execution = materialize_grouped_execution_surface_from_truth_view(
        &planned,
        live.basis().clone(),
        &next_truth_view,
    )
    .unwrap();

    let execution = execute_grouped_live_view_shape_change(
        admit_grouped_live_view(&live).unwrap(),
        &BridgeChangeSummary::default()
            .with_field_delta(BridgeFieldDelta::new(
                "identity",
                "id",
                Some("task-1"),
                Some("task-1"),
            ))
            .with_field_delta(BridgeFieldDelta::new(
                "status",
                "lane",
                Some("todo"),
                Some("doing"),
            ))
            .with_membership_transition(true, true),
        &next_grouped_execution,
    )
    .unwrap();

    assert_eq!(
        execution.patch_envelope().patch_family(),
        Some(ViewShapePatchFamily::KanbanGroupMembershipPatch)
    );
    match execution.patch_envelope().payload() {
        ViewShapePatchPayload::KanbanGroupMembershipPatch(delta) => {
            assert_eq!(delta.transitions().len(), 3);
            assert_eq!(delta.next().result().lane_count(), 2);
        }
        other => panic!("expected grouped delta payload, got {other:?}"),
    }
    assert_grouped_delta_counters_are_debt_free(&execution, 3, 2);
}
