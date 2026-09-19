pub(super) fn evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> Option<serde_json::Value> {
    let shutdown = receipt.client_shutdown()?;
    let snapshot = receipt.visual_snapshot()?;
    let retained_frames = receipt
        .retained_frames()
        .iter()
        .map(|frame| {
            serde_json::json!({
                "frame": frame.frame(),
                "kind": format!("{:?}", frame.kind()),
                "retained_baseline_rgba8": frame.retained_baseline_rgba8(),
                "retained_center_rgba8": frame.retained_center_rgba8(),
                "client_physical_size": frame.presentation().map(|value| value.client_physical_size()),
                "presentation_attempt": frame.presentation().map(|value| value.presentation_attempt()),
            })
        })
        .collect::<Vec<_>>();
    let post_restore_presentation = retained_frames.last().cloned();
    Some(serde_json::json!({
        "schema": "worth-ui-native-phase8-evidence-v1",
        "retained_frames": retained_frames,
        "post_restore_presentation": post_restore_presentation,
        "presentation": {
            "frame": receipt.presentation().map(|value| value.presented_frame()),
            "attempt": receipt.presentation().map(|value| value.presentation_attempt()),
            "binding": receipt.presentation().map(|value| value.binding_generation()),
            "client_physical_size": receipt.presentation().map(|value| value.client_physical_size()),
        },
        "snapshot": {
            "affinity": snapshot.affinity(),
            "relation": format!("{:?}", snapshot.relation()),
            "client_physical_dimensions": snapshot.client_physical_dimensions(),
            "viewport_logical_dimension_bits": snapshot.viewport_logical_dimension_bits(),
            "scale_bits": snapshot.scale_bits(),
            "pixel_dimensions": snapshot.pixel_dimensions(),
            "pixel_byte_count": snapshot.pixel_bytes().len(),
        },
        "graphics_generations": {
            "device": receipt.graphics().device_generation(),
            "surface": receipt.graphics().surface_generation(),
        },
        "surface_suspension": {
            "count": receipt.graphics().surface_suspensions(),
            "targetless_count": receipt.graphics().targetless_surface_suspensions(),
        },
        "peak": {
            "surfaces": receipt.peak_census().surfaces,
            "devices": receipt.peak_census().devices,
            "queues": receipt.peak_census().queues,
            "retained_targets": receipt.peak_census().retained_targets,
            "readback_buffers": receipt.peak_census().readback_buffers,
        },
        "query_close_complete": shutdown.managed_semantic_resources_complete(),
        "intent_resources_empty": shutdown.intent_resources_empty(),
        "terminal_zero": receipt.terminal_census().is_zero(),
    }))
}
