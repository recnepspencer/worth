pub(super) fn native_phase2_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    serde_json::json!({
        "schema": "worth-ui-native-phase2-evidence-v1",
        "presentation": presentation_evidence(receipt),
        "runtime_attribution": attribution_evidence(receipt),
        "counters": counter_evidence(receipt),
        "graphics": graphics_evidence(receipt),
        "peak": peak_census_evidence(receipt),
        "terminal_census": terminal_census_evidence(receipt),
        "terminal_zero": receipt.terminal_census().is_zero(),
    })
}

fn presentation_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    let presentation = receipt.presentation();
    serde_json::json!({
        "presented_source": presentation.map(|value| value.source_rgba8()),
        "retained_center": presentation.map(|value| value.retained_center_rgba8()),
        "retained_baseline": presentation.map(|value| value.retained_baseline_rgba8()),
        "client_physical_size": presentation.map(|value| value.client_physical_size()),
        "scale_factor_milli": presentation.map(|value| value.scale_factor_milli()),
        "frame": presentation.map(|value| value.presented_frame()),
        "surface": presentation.map(|value| value.semantic_surface()),
        "binding": presentation.map(|value| value.binding_generation()),
        "mounted_instance": presentation.map(|value| value.mounted_instance()),
        "node_receipt": presentation.map(|value| value.node_receipt()),
        "presentation_attempt": presentation.map(|value| value.presentation_attempt()),
        "logical_bounds_milli": presentation.map(|value| value.logical_bounds_milli()),
        "order_ordinal": presentation.map(|value| value.order_ordinal()),
    })
}

fn attribution_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    let attribution = receipt.client_attribution();
    serde_json::json!({
        "frame": attribution.map(|value| value.frame()),
        "surface": attribution.map(|value| value.surface()),
        "binding": attribution.map(|value| value.binding()),
        "mounted_instance": attribution.map(|value| value.mounted_instance()),
        "node_receipt": attribution.map(|value| value.node_receipt()),
        "presentation_attempt": attribution.map(|value| value.presentation_attempt()),
        "authored_provenance_digest": attribution.map(|value| value.authored_provenance_digest()),
        "authored_semantic_identity_digest": attribution.map(|value| value.authored_semantic_identity_digest()),
    })
}

fn counter_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    let cost = receipt.final_frame().cost();
    serde_json::json!({
        "surface_acquisitions": cost.surface_acquisitions(),
        "queue_submissions": cost.queue_submissions(),
        "presents": cost.presents(),
        "render_passes": cost.render_passes(),
        "readiness_signals": receipt.readiness_signals(),
        "redraw_turns": receipt.redraw_turns(),
        "idle_wait_turns": receipt.idle_wait_turns(),
        "coalesced_wakes": receipt.coalesced_wakes(),
        "port_crossings": receipt.port_crossings(),
    })
}

fn graphics_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    serde_json::json!({
        "event_loop_thread": receipt.event_loop_thread(),
        "event_loop_thread_matches_launch": receipt.event_loop_thread_matches_launch(),
        "event_loop_thread_posture": receipt.event_loop_thread_posture().label(),
        "adapter": receipt.graphics().adapter_name(),
        "vendor": receipt.graphics().vendor(),
        "device": receipt.graphics().device(),
        "driver": receipt.graphics().driver(),
        "driver_info": receipt.graphics().driver_info(),
        "device_type": receipt.graphics().device_type(),
        "backend": receipt.graphics().backend(),
        "surface_format": receipt.graphics().surface_format(),
        "present_mode": receipt.graphics().present_mode(),
        "alpha_mode": receipt.graphics().alpha_mode(),
        "retained_format": receipt.graphics().retained_format(),
        "max_texture_dimension_2d": receipt.graphics().max_texture_dimension_2d(),
    })
}

fn peak_census_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    serde_json::Value::Object(
        receipt
            .peak_census()
            .entries()
            .map(|(class, count)| (class.to_owned(), serde_json::Value::from(count)))
            .collect::<serde_json::Map<_, _>>(),
    )
}

fn terminal_census_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    serde_json::Value::Object(
        receipt
            .terminal_census()
            .entries()
            .map(|(class, count)| (class.to_owned(), serde_json::Value::from(count)))
            .collect::<serde_json::Map<_, _>>(),
    )
}
