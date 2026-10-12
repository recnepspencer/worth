use crate::runtime::{WorthQueryAspectTouch, WorthQueryLiveView, WorthQueryUnrefinedLiveShape};

use super::super::digests::{inspection_digest, touched_aspect_digest};
use super::super::fixture::{stateful_bridge_task_runtime, task_live_view};
use super::super::{
    description_value_touch, identity_id_touch, title_value_touch,
    AspectApiFinalizationCertificationBundle,
};
use super::authoring_values::string_aspect_value;
use super::receipt_digest::write_receipt_digest;

pub(super) fn authoritative_crud_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    entity_id: &str,
    initial_title: &str,
    renamed_title: &str,
) -> AspectApiFinalizationCertificationBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.crud")
        .expect("workspace should open");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> =
        task_live_view(request_execution, &mut workspace, "tasks.cert-crud");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(identity_id_touch(), string_aspect_value(entity_id))
                .set_aspect(title_value_touch(), string_aspect_value(initial_title))
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);
    let update = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| task.set_aspect(title_value_touch(), string_aspect_value(renamed_title)),
        )
        .expect("rename should execute");
    let _patches = workspace.observe(&live);
    let delete = workspace
        .delete_with(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| task.touch(title_value_touch()),
        )
        .expect("delete should execute");
    let state = workspace
        .state(request_execution, &delete)
        .expect("state should resolve");
    let inspection = workspace
        .inspect(request_execution, &delete)
        .expect("inspection should resolve");

    AspectApiFinalizationCertificationBundle {
        mutation_surface_label: "workspace.insert/update/delete".to_string(),
        authority_lane_label: delete.authority_lane().to_string(),
        mutation_family_label: delete.mutation_family().to_string(),
        support_matrix_digest: workspace
            .public_support_matrix()
            .matrix_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        mutation_surface_report_digest: workspace
            .public_mutation_surface_report()
            .report_digest()
            .to_string(),
        closeout_digest: workspace
            .public_aspect_api_finalization_closeout()
            .closeout_digest()
            .to_string(),
        receipt_digest: write_receipt_digest(&delete),
        state_digest: state
            .state_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        inspection_digest: inspection_digest(&inspection),
        touched_aspect_digest: touched_aspect_digest(update.deltas()[0].admitted_touched_aspects()),
        affected_live_view_count: delete.affected_live_view_targets().len(),
        affected_derived_view_count: delete.affected_derived_view_targets().len(),
        routed_patch_count: 1,
        materialized_row_count: 0,
        preview_residue_count: 0,
    }
}

pub(super) fn clear_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    aspect_touch: WorthQueryAspectTouch,
) -> AspectApiFinalizationCertificationBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.clear")
        .expect("workspace should open");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> =
        task_live_view(request_execution, &mut workspace, "tasks.cert-clear");
    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(identity_id_touch(), string_aspect_value("task-1"))
                .set_aspect(title_value_touch(), string_aspect_value("Buy milk"))
                .set_aspect(description_value_touch(), string_aspect_value("whole milk"))
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);

    let receipt = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| task.clear(aspect_touch),
        )
        .expect("clear should execute");
    let patches = workspace.observe(&live);
    let state = workspace
        .state(request_execution, &receipt)
        .expect("state should resolve");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("inspection should resolve");

    AspectApiFinalizationCertificationBundle {
        mutation_surface_label: "workspace.update.clear".to_string(),
        authority_lane_label: receipt.authority_lane().to_string(),
        mutation_family_label: receipt.mutation_family().to_string(),
        support_matrix_digest: workspace
            .public_support_matrix()
            .matrix_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        mutation_surface_report_digest: workspace
            .public_mutation_surface_report()
            .report_digest()
            .to_string(),
        closeout_digest: workspace
            .public_aspect_api_finalization_closeout()
            .closeout_digest()
            .to_string(),
        receipt_digest: write_receipt_digest(&receipt),
        state_digest: state
            .state_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        inspection_digest: inspection_digest(&inspection),
        touched_aspect_digest: touched_aspect_digest(
            receipt.deltas()[0].admitted_touched_aspects(),
        ),
        affected_live_view_count: receipt.affected_live_view_targets().len(),
        affected_derived_view_count: receipt.affected_derived_view_targets().len(),
        routed_patch_count: patches.query_delivery_batches.len(),
        materialized_row_count: workspace.read(request_execution, &live).len(),
        preview_residue_count: 0,
    }
}
