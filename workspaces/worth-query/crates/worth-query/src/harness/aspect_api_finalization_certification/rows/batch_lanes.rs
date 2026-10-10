use crate::runtime::{
    WorthQueryAuthorityLane, WorthQueryLiveView, WorthQueryPreviewOptions,
    WorthQueryUnrefinedLiveShape,
};
use crate::WorthQuerySessionLabel;

use super::super::digests::{inspection_digest, touched_aspect_digest};
use super::super::fixture::{
    stateful_bridge_task_runtime, task_live_view, CertificationTitleListMaintainer,
};
use super::super::{
    identity_id_touch, title_value_touch, ui_batch_summary_touch,
    AspectApiFinalizationCertificationBundle,
};
use super::authoring_values::string_aspect_value;

pub(super) fn authoritative_batch_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> AspectApiFinalizationCertificationBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.batch")
        .expect("workspace should open");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> =
        task_live_view(request_execution, &mut workspace, "tasks.cert-batch");
    let computed = workspace
        .computed::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "tasks.cert-batch-summary",
            |c| {
                c.depends_on_live(&live)
                    .reads([title_value_touch()])
                    .produces([ui_batch_summary_touch()])
            },
            CertificationTitleListMaintainer,
        )
        .expect("computed should declare");
    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(identity_id_touch(), string_aspect_value("task-1"))
                        .set_aspect(title_value_touch(), string_aspect_value("Buy milk"))
                })
                .insert("Task", |task| {
                    task.set_aspect(identity_id_touch(), string_aspect_value("task-2"))
                        .set_aspect(title_value_touch(), string_aspect_value("Buy bread"))
                })
        })
        .expect("batch should execute");
    let patches = workspace.observe(&live);
    let materialization = workspace
        .materialize_result(request_execution, &computed)
        .expect("aspect API certification materialization should execute");
    let state = workspace
        .state(request_execution, &receipt)
        .expect("state should resolve");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("inspection should resolve");

    AspectApiFinalizationCertificationBundle {
        mutation_surface_label: "workspace.batch".to_string(),
        authority_lane_label: receipt.authority_lane().to_string(),
        mutation_family_label: "batch".to_string(),
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
        receipt_digest: receipt.batch_digest().to_string(),
        state_digest: state
            .state_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        inspection_digest: inspection_digest(&inspection),
        touched_aspect_digest: touched_aspect_digest(receipt.admitted_touched_aspects()),
        affected_live_view_count: receipt.affected_live_view_targets().len(),
        affected_derived_view_count: receipt.affected_derived_view_targets().len(),
        routed_patch_count: patches.query_delivery_batches.len(),
        materialized_row_count: materialization.row_count(),
        preview_residue_count: 0,
    }
}

pub(super) fn preview_batch_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> AspectApiFinalizationCertificationBundle {
    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.preview-batch")
        .expect("workspace should open");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            WorthQuerySessionLabel::scoped_strs(
                "aspect-api-finalization-certification",
                ["preview-batch"],
            )
            .expect("preview label should build"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");
    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(identity_id_touch(), string_aspect_value("preview-task-1"))
                        .set_aspect(
                            title_value_touch(),
                            string_aspect_value("Preview title one"),
                        )
                })
                .insert("Task", |task| {
                    task.set_aspect(identity_id_touch(), string_aspect_value("preview-task-2"))
                        .set_aspect(
                            title_value_touch(),
                            string_aspect_value("Preview title two"),
                        )
                })
        })
        .expect("preview batch should stage");
    let outcome = preview.discard();
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("inspection should resolve");
    let state = workspace
        .state(request_execution, &receipt)
        .expect("state should resolve");

    AspectApiFinalizationCertificationBundle {
        mutation_surface_label: "preview.batch".to_string(),
        authority_lane_label: receipt.authority_lane().to_string(),
        mutation_family_label: "batch".to_string(),
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
        receipt_digest: receipt.batch_digest().to_string(),
        state_digest: state
            .state_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        inspection_digest: inspection_digest(&inspection),
        touched_aspect_digest: touched_aspect_digest(receipt.admitted_touched_aspects()),
        affected_live_view_count: receipt.affected_live_view_targets().len(),
        affected_derived_view_count: receipt.affected_derived_view_targets().len(),
        routed_patch_count: 0,
        materialized_row_count: 0,
        preview_residue_count: outcome.pending_write_intent_residue_count(),
    }
}

pub(super) fn mutation_surface_contract_lane(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> AspectApiFinalizationCertificationBundle {
    let workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("aspect-api.contract")
        .expect("workspace should open");
    let report = workspace.public_mutation_surface_report();
    let closeout = workspace.public_aspect_api_finalization_closeout();

    AspectApiFinalizationCertificationBundle {
        mutation_surface_label:
            "workspace.public_mutation_surface_report/public_aspect_api_finalization_closeout"
                .to_string(),
        authority_lane_label: WorthQueryAuthorityLane::AuthoritativeTruth.to_string(),
        mutation_family_label: "contract".to_string(),
        support_matrix_digest: workspace
            .public_support_matrix()
            .matrix_digest()
            .terminal_projection_for_reporting()
            .to_string(),
        mutation_surface_report_digest: report.report_digest().to_string(),
        closeout_digest: closeout.closeout_digest().to_string(),
        receipt_digest: report.report_digest().to_string(),
        state_digest: closeout.closeout_digest().to_string(),
        inspection_digest: crate::harness::certification::digest_parts(&[
            report.report_digest().to_string(),
            closeout.closeout_digest().to_string(),
        ]),
        touched_aspect_digest: crate::harness::certification::digest_parts(
            &closeout
                .preferred_stable_surfaces()
                .iter()
                .map(|surface| format!("surface:{surface}"))
                .collect::<Vec<_>>(),
        ),
        affected_live_view_count: report.preferred_stable_count(),
        affected_derived_view_count: report.lower_level_stable_count(),
        routed_patch_count: report.lower_level_stable_count(),
        materialized_row_count: report.support_gated_count(),
        preview_residue_count: 0,
    }
}
