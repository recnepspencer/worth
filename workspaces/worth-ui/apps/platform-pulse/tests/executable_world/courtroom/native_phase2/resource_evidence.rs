pub(super) fn assert_exact_resource_evidence(evidence: &serde_json::Value) {
    let mut expected = PHASE_FIVE_RESOURCE_CLASSES
        .iter()
        .map(|field| ((*field).to_owned(), serde_json::Value::from(0)))
        .collect::<serde_json::Map<_, _>>();
    for (field, count) in [
        ("windows", 1),
        ("surfaces", 1),
        ("adapters", 1),
        ("devices", 1),
        ("queues", 1),
        ("retained_targets", 2),
        ("registrations", 1),
        ("readback_buffers", 1),
        ("pending_submissions", 1),
        ("event_wake_registrations", 1),
        ("application_drivers", 1),
        ("physical_signal_runtimes", 1),
        ("physical_signal_workers", 1),
        ("physical_signal_transition_observations", 1),
        ("retained_draw_lists", 1),
        ("presentation_epochs", 1),
        ("reconstruction_requirements", 1),
        ("retained_frame_observations", 1),
    ] {
        expected.insert(field.to_owned(), count.into());
    }
    assert_eq!(evidence["peak"], serde_json::Value::Object(expected));
    assert_eq!(evidence["terminal_zero"], true);
}

const PHASE_FIVE_RESOURCE_CLASSES: &[&str] = &[
    "alpha_atlas_pages",
    "color_atlas_pages",
    "atlas_staging_buffers",
    "text_atlas_plans",
    "text_atlas_reservations",
    "text_atlas_pins",
    "text_atlas_recoveries",
    "text_atlas_alpha_entries",
    "text_atlas_color_entries",
    "text_atlas_upload_submissions",
    "text_atlas_recovery_authorities",
    "text_atlas_in_flight_transactions",
    "physical_signal_runtimes",
    "physical_signal_workers",
    "physical_signal_pending_work",
    "physical_signal_wakes",
    "physical_signal_transition_observations",
    "pending_presentations",
    "pending_presentation_settlements",
    "retained_draw_lists",
    "presentation_epochs",
    "reconstruction_requirements",
    "text_pin_bindings",
    "pending_text_presentations",
    "retained_frame_observations",
    "text_pin_frame_observations",
    "text_atlas_plan_observations",
    "client_mounted_layouts",
    "client_raster_cache_entries",
];
