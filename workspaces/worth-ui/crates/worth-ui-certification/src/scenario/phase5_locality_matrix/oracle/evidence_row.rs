use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use worth_ui_host_native::{
    UiNativeClientTextPresentationWorkObservation, UiNativePhysicalSignalTransitionObservation,
};

use super::Phase5LocalityEvidence;

pub(super) fn assemble(
    evidence: &Phase5LocalityEvidence,
    presentation: &worth_ui_host_native::UiNativePresentationObservation,
    completed: usize,
    work: &UiNativeClientTextPresentationWorkObservation,
    physical: &UiNativePhysicalSignalTransitionObservation,
) -> Value {
    let case = evidence.case();
    let timing = evidence.timing();
    let receipt = evidence.receipt();
    let shutdown = receipt
        .client_shutdown()
        .expect("adjudicated matrix evidence retains Query shutdown");
    json!({
        "retained": case.retained_size(),
        "retained_mechanics": case.retained_mechanics(),
        "retained_paragraphs": case.retained_paragraphs(),
        "axis": case.axis().label(),
        "target_index": case.target_index(),
        "world_elapsed_ms": evidence.world_elapsed_millis(),
        "event_loop_thread_posture": receipt.event_loop_thread_posture().label(),
        "event_loop_thread_matches_launch": receipt.event_loop_thread_matches_launch(),
        "timing_us": timing_row(&timing),
        "query_completed": completed,
        "authored_mounted_instances": {
            "count": shutdown.authored_mounted_instances().len(),
            "evidence_digest": digest_json(&json!(shutdown.authored_mounted_instances().iter().map(|row| (
                digest_hex(row.authored_semantic_identity_digest()), row.mounted_instance()
            )).collect::<Vec<_>>())),
        },
        "text": text_work_row(work),
        "text_work": shutdown.text_presentation_work().iter().map(text_work_row).collect::<Vec<_>>(),
        "atlas_plans": atlas_plan_rows(receipt.text_atlas_plan_observations()),
        "physical_signal": physical_signal_row(physical),
        "native": native_row(presentation),
        "terminal_zero": true,
    })
}

fn timing_row(timing: &super::super::execution::Phase5LocalityTimingView) -> Value {
    json!({
        "profile": timing.profile_micros,
        "platform_prepare": timing.platform_prepare_micros,
        "query_install": timing.query_install_micros,
        "fixture_materialization": timing.application.fixture_materialization_micros,
        "owner_installation": timing.application.owner_installation_micros,
        "builder_registration": timing.application.builder_registration_micros,
        "application_completion": timing.application.application_completion_micros,
        "native_run": timing.native_run_micros,
    })
}

fn atlas_plan_rows(plans: &[worth_ui_host_native::UiNativeTextAtlasPlanObservation]) -> Vec<Value> {
    plans.iter().map(|plan| json!({
        "attempt": plan.attempt(), "binding": plan.binding(),
        "key_lookups": plan.key_lookups(), "hits": plan.hits(), "misses": plan.misses(),
        "page_probes": plan.page_probes(), "placement_probes": plan.placement_probes(),
        "eviction_candidates": plan.eviction_candidates(), "evictions": plan.evictions(),
        "staged_bytes": plan.staged_bytes(), "physical_staged_bytes": plan.physical_staged_bytes(),
        "peak_entries": plan.peak_entries(),
    })).collect()
}

fn physical_signal_row(physical: &UiNativePhysicalSignalTransitionObservation) -> Value {
    json!({
        "host_session": physical.host_session(), "attempt": physical.attempt(),
        "surface": physical.surface(), "host_surface": physical.host_surface(),
        "binding": physical.binding(), "request_sequence": physical.request_sequence(),
        "work": format!("{:?}", physical.work()),
        "origin": format!("{:?}", physical.origin()),
        "external_status": format!("{:?}", physical.external_status()),
        "settlement": format!("{:?}", physical.settlement()),
        "performed_transitions": physical.performed_transitions(),
        "performed_nodes": physical.performed_nodes(), "fact_revision": physical.fact_revision(),
        "read_scopes": physical.read_scopes(),
    })
}

fn native_row(presentation: &worth_ui_host_native::UiNativePresentationObservation) -> Value {
    json!({
        "frame": presentation.presented_frame(),
        "attempt": presentation.presentation_attempt(),
        "binding": presentation.binding_generation(),
        "scale_factor_milli": presentation.scale_factor_milli(),
        "client_physical_size": presentation.client_physical_size(),
        "source_rgba8": presentation.source_rgba8(),
        "retained_baseline_rgba8": presentation.retained_baseline_rgba8(),
        "retained_center_rgba8": presentation.retained_center_rgba8(),
        "production_cost": production_cost(presentation.production_cost()),
        "cost": presentation_cost(presentation.cost()),
    })
}

fn digest_hex(digest: [u8; 32]) -> String {
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest_json(value: &Value) -> String {
    digest_hex(Sha256::digest(serde_json::to_vec(value).expect("evidence serializes")).into())
}

fn text_work_row(work: &UiNativeClientTextPresentationWorkObservation) -> Value {
    json!({
        "attempt": work.attempt(),
        "binding": work.binding(),
        "frame": work.mounted_frame(),
        "host_lineage": work.host_lineage(),
        "dpi_milli": work.dpi_milli(),
        "layouts": work.layout_count(),
        "paint_spans": work.paint_span_count(),
        "demand_records": work.demand_records(),
        "key_checks": work.key_checks(),
        "rasterized_glyphs": work.rasterized_glyphs(),
        "rasterized_texels": work.rasterized_texels(),
        "produced_bytes": work.produced_bytes(),
        "pin_additions": work.pin_additions(),
        "pin_releases": work.pin_releases(),
        "binding_pins": work.binding_pins(),
        "removed_mechanics": work.removed_mechanics(),
        "active_mechanic_identity_digest": digest_hex(work.active_mechanic_identity_digest()),
        "removed_mechanic_identity_digest": digest_hex(work.removed_mechanic_identity_digest()),
        "active_mechanic_identity_count": work.active_mechanic_identities().len(),
        "removed_mechanic_identity_count": work.removed_mechanic_identities().len(),
        "qualification": {
            "analyzed_bytes": work.analyzed_bytes(),
            "graphemes": work.graphemes(),
            "word_boundaries": work.word_boundaries(),
            "line_opportunities": work.line_opportunities(),
            "bidi_contexts": work.bidi_contexts(),
            "fallback_clusters": work.fallback_clusters(),
            "coverage_index_queries": work.coverage_index_queries(),
            "face_shape_attempts": work.face_shape_attempts(),
            "probed_glyphs": work.probed_glyphs(),
            "shaped_runs": work.shaped_runs(),
            "shaped_scalars": work.shaped_scalars(),
            "emitted_glyphs": work.emitted_glyphs(),
            "fitted_units": work.fitted_units(),
            "emitted_lines": work.emitted_lines(),
            "emitted_visual_runs": work.emitted_visual_runs(),
            "positioned_glyphs": work.positioned_glyphs(),
            "emitted_carets": work.emitted_carets(),
        },
    })
}

fn production_cost(cost: worth_ui_host_contract::UiMountedPresentationProductionCost) -> Value {
    json!({
        "source_instances": cost.source_instances(),
        "commands_considered": cost.commands_considered(),
        "command_index_lookups": cost.command_index_lookups(),
        "order_lookups": cost.order_lookups(),
        "retained_command_scans": cost.retained_command_scans(),
        "retained_command_clones": cost.retained_command_clones(),
        "projection_rows_materialized": cost.projection_rows_materialized(),
    })
}

fn presentation_cost(cost: worth_ui_host_contract::UiHostPresentationCostReport) -> Value {
    json!({
        "presented_surfaces": cost.presented_surfaces(),
        "translated_rows": cost.translated_rows(),
        "translated_bytes": cost.translated_bytes(),
        "cache_hits": cost.native_resource_cache_hits(),
        "cache_misses": cost.native_resource_cache_misses(),
        "asynchronous_handoffs": cost.asynchronous_handoffs(),
        "delta_rows_carried": cost.delta_rows_carried(),
        "draw_list_mutations": cost.draw_list_mutations(),
        "order_mutations": cost.order_mutations(),
        "order_index_lookups": cost.order_index_lookups(),
        "order_index_node_touches": cost.order_index_node_touches(),
        "order_index_rotations": cost.order_index_rotations(),
        "order_index_high_water": cost.order_index_high_water(),
        "damage_regions": cost.logical_damage_regions(),
        "logical_damage_pixels": cost.logical_damage_pixels(),
        "retained_command_scans": cost.retained_command_scans(),
        "retained_command_clones": cost.retained_command_clones(),
        "damage_index_probes": cost.damage_index_probes(),
        "damage_index_stored_records": cost.damage_index_stored_records(),
        "damage_index_high_water": cost.damage_index_high_water(),
        "damage_region_command_checks": cost.damage_region_command_checks(),
        "intersecting_commands": cost.intersecting_commands(),
        "replayed_commands": cost.replayed_commands(),
        "cleared_pixels": cost.cleared_pixels(),
        "rendered_pixels": cost.rendered_pixels(),
        "gpu_writes": cost.gpu_writes(),
        "render_passes": cost.render_passes(),
        "surface_copies": cost.surface_copies(),
        "surface_acquisitions": cost.surface_acquisitions(),
        "queue_submissions": cost.queue_submissions(),
        "presents": cost.presents(),
        "presented_pixels": cost.presented_pixels(),
    })
}
