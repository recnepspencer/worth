use worth_ui_native_platform::{UiNativeClientShutdownObservation, UiNativePlatformCloseReceipt};

pub(crate) fn publish(
    receipt: &UiNativePlatformCloseReceipt,
    shutdown: &UiNativeClientShutdownObservation,
) {
    println!("{}", evidence(receipt, shutdown));
}

fn evidence(
    receipt: &UiNativePlatformCloseReceipt,
    shutdown: &UiNativeClientShutdownObservation,
) -> serde_json::Value {
    let transitions = shutdown
        .presentation_transitions()
        .iter()
        .map(|transition| {
            serde_json::json!({
                "kind": format!("{:?}", transition.kind()),
                "attempt": transition.attempt(),
                "binding": transition.binding(),
            })
        })
        .collect::<Vec<_>>();
    let text_work = shutdown
        .text_presentation_work()
        .iter()
        .map(text_work_evidence)
        .collect::<Vec<_>>();
    let presentation = receipt.presentation();
    let attribution = receipt.client_attribution();
    let retained_frames = receipt
        .retained_frames()
        .iter()
        .map(|frame| {
            serde_json::json!({
                "frame": frame.frame(),
                "kind": format!("{:?}", frame.kind()),
                "baseline": frame.retained_baseline_rgba8(),
                "center": frame.retained_center_rgba8(),
                "presents": frame.cost().presents(),
                "presented_pixels": frame.cost().presented_pixels(),
                "translated_rows": frame.cost().translated_rows(),
                "translated_bytes": frame.cost().translated_bytes(),
                "cache_hits": frame.cost().native_resource_cache_hits(),
                "cache_misses": frame.cost().native_resource_cache_misses(),
                "draw_list_mutations": frame.cost().draw_list_mutations(),
                "intersecting_commands": frame.cost().intersecting_commands(),
                "replayed_commands": frame.cost().replayed_commands(),
                "rendered_pixels": frame.cost().rendered_pixels(),
                "gpu_writes": frame.cost().gpu_writes(),
            })
        })
        .collect::<Vec<_>>();
    let atlas_plans = receipt
        .text_atlas_plan_observations()
        .iter()
        .copied()
        .map(|plan| {
            serde_json::json!({
                "host_session": plan.host_session(),
                "attempt": plan.attempt(),
                "surface": plan.surface(),
                "binding": plan.binding(),
                "key_lookups": plan.key_lookups(),
                "hits": plan.hits(),
                "misses": plan.misses(),
                "page_probes": plan.page_probes(),
                "placement_probes": plan.placement_probes(),
                "eviction_candidates": plan.eviction_candidates(),
                "evictions": plan.evictions(),
                "staged_bytes": plan.staged_bytes(),
                "physical_staged_bytes": plan.physical_staged_bytes(),
                "peak_entries": plan.peak_entries(),
            })
        })
        .collect::<Vec<_>>();
    let physical_signal_transitions = receipt
        .physical_signal_transition_observations()
        .iter()
        .copied()
        .map(|transition| {
            serde_json::json!({
                "host_session": transition.host_session(),
                "attempt": transition.attempt(),
                "surface": transition.surface(),
                "host_surface": transition.host_surface(),
                "binding": transition.binding(),
                "request_sequence": transition.request_sequence(),
                "work": format!("{:?}", transition.work()),
                "origin": format!("{:?}", transition.origin()),
                "external_status": format!("{:?}", transition.external_status()),
                "settlement": format!("{:?}", transition.settlement()),
                "performed_transitions": transition.performed_transitions(),
                "performed_nodes": transition.performed_nodes(),
                "fact_revision": transition.fact_revision(),
                "read_scopes": transition.read_scopes(),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "schema": "worth-ui-native-phase-f-async-world-v1",
        "presentation_transitions": transitions,
        "presentation_transition_count": transitions.len(),
        "presentation_transition_trace_complete": shutdown.presentation_transition_trace_complete(),
        "text_presentation_work": text_work,
        "text_presentation_work_trace_complete": shutdown.text_presentation_work_trace_complete(),
        "text_atlas_plans": atlas_plans,
        "physical_signal_transitions": physical_signal_transitions,
        "physical_signal_transition_trace_complete": receipt.physical_signal_transition_trace_complete(),
        "closed_query_resources": shutdown.managed_semantic_resources_closed(),
        "query_close_complete": shutdown.managed_semantic_resources_complete(),
        "terminal_zero": receipt.terminal_census().is_zero(),
        "observation_history_complete": receipt.observation_history_complete(),
        "physical_signal_runtimes": receipt.peak_census().physical_signal_runtimes,
        "physical_signal_workers": receipt.peak_census().physical_signal_workers,
        "peak_text_layout_count": receipt.peak_text_layout_count(),
        "text_pin_frame_counts": receipt.text_pin_frame_counts(),
        "text_pin_frames": crate::native_phase_f_evidence::pin_frames(receipt),
        "retained_frame_intrinsic_glyphs": crate::native_phase_f_evidence::retained_frame_intrinsic_glyphs(receipt),
        "presentation": {
            "client_physical_size": presentation.client_physical_size(),
            "scale_factor_milli": presentation.scale_factor_milli(),
            "source": presentation.source_rgba8(),
            "retained_center": presentation.retained_center_rgba8(),
            "retained_baseline": presentation.retained_baseline_rgba8(),
            "frame": presentation.presented_frame(),
            "binding": presentation.binding_generation(),
            "attempt": presentation.presentation_attempt(),
            "alpha_glyphs": crate::native_phase_f_evidence::alpha_glyphs(receipt),
            "intrinsic_glyphs": crate::native_phase_f_evidence::intrinsic_glyphs(receipt),
            "glyph_transcript_digest": crate::native_phase_f_evidence::hex_digest(presentation.glyph_transcript_digest()),
            "intrinsic_glyph_transcript_digest": crate::native_phase_f_evidence::hex_digest(presentation.intrinsic_glyph_transcript_digest()),
        },
        "runtime_attribution": {
            "frame": attribution.frame(),
            "binding": attribution.binding(),
            "attempt": attribution.presentation_attempt(),
        },
        "retained_frames": retained_frames,
    })
}

fn text_work_evidence(
    work: &worth_ui_native_platform::UiNativeClientTextPresentationWorkObservation,
) -> serde_json::Value {
    let active_mechanics = work
        .active_mechanic_identities()
        .iter()
        .copied()
        .map(|mechanic| {
            serde_json::json!({
                "mounted_instance": mechanic.mounted_instance(),
                "semantic_slot": mechanic.semantic_slot(),
                "collection_row": mechanic.collection_row().map(crate::native_phase_f_evidence::hex_digest),
                "layout_digest": crate::native_phase_f_evidence::hex_digest(mechanic.layout_digest()),
                "raster_key_set_digest": crate::native_phase_f_evidence::hex_digest(mechanic.raster_key_set_digest()),
            })
        })
        .collect::<Vec<_>>();
    let binding_pin_identities = work
        .binding_pin_identities()
        .iter()
        .map(|[layout, raster_key]| {
            serde_json::json!({
                "layout": crate::native_phase_f_evidence::hex_digest(*layout),
                "raster_key": crate::native_phase_f_evidence::hex_digest(*raster_key),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "attempt": work.attempt(),
        "binding": work.binding(),
        "mounted_frame": work.mounted_frame(),
        "host_lineage": work.host_lineage(),
        "dpi_milli": work.dpi_milli(),
        "layout_count": work.layout_count(),
        "paint_span_count": work.paint_span_count(),
        "demand_batches": work.demand_batches(),
        "demand_records": work.demand_records(),
        "key_checks": work.key_checks(),
        "rasterized_glyphs": work.rasterized_glyphs(),
        "rasterized_texels": work.rasterized_texels(),
        "produced_bytes": work.produced_bytes(),
        "pin_additions": work.pin_additions(),
        "pin_releases": work.pin_releases(),
        "binding_pins": work.binding_pins(),
        "binding_pin_identities": binding_pin_identities,
        "removed_mechanics": work.removed_mechanics(),
        "active_mechanic_identity_digest": crate::native_phase_f_evidence::hex_digest(work.active_mechanic_identity_digest()),
        "active_mechanics": active_mechanics,
        "layout_set_digest": crate::native_phase_f_evidence::hex_digest(work.layout_set_digest()),
        "raster_key_set_digest": crate::native_phase_f_evidence::hex_digest(work.raster_key_set_digest()),
        "glyph_run_transcript_digest": crate::native_phase_f_evidence::hex_digest(work.glyph_run_transcript_digest()),
        "intrinsic_glyph_transcript_digest": crate::native_phase_f_evidence::hex_digest(work.intrinsic_glyph_transcript_digest()),
        "intrinsic_glyph_runs": work.intrinsic_glyph_runs(),
    })
}
