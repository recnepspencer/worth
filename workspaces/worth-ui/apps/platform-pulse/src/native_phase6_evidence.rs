/// Every batch the host retained must reach a typed runtime disposition, except
/// a pointer motion that retention folded into its predecessor: that submission
/// was retained, but the drain hands it over inside the predecessor's batch.
pub(super) fn ingress_settles_retained_batches(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> bool {
    let input = receipt.input_observations();
    let coalesced = input.coalesced_motion_batch_count();
    receipt.client_shutdown().is_some_and(|shutdown| {
        let counts = shutdown.observation_ingress().counts();
        counts[0] > 0
            && counts[4] == 0
            && coalesced <= input.retained_batch_count()
            && counts[..4].iter().sum::<u64>() + coalesced >= input.retained_batch_count()
    })
}

pub(super) fn native_phase6_evidence(
    receipt: &worth_ui_native_platform::UiNativePlatformCloseReceipt,
) -> serde_json::Value {
    let mut evidence = crate::native_phase2_evidence::native_phase2_evidence(receipt);
    let input = receipt.input_observations();
    let ingress = receipt
        .client_shutdown()
        .map(|shutdown| shutdown.observation_ingress().counts())
        .unwrap_or([0; 5]);
    if let serde_json::Value::Object(fields) = &mut evidence {
        fields.insert(
            "schema".to_owned(),
            serde_json::Value::String("worth-ui-native-phase6-evidence-v2".to_owned()),
        );
        fields.insert(
            "input".to_owned(),
            serde_json::json!({
                "retained_batches": input.retained_batch_count(),
                "coalesced_motion_batches": input.coalesced_motion_batch_count(),
                "retained_events": input.retained_event_count(),
                "first_sequence": input.first_retained_sequence(),
                "last_sequence": input.last_retained_sequence(),
                "family_counts": input.family_counts(),
                "profile_transitions": input.profile_transition_count(),
                "completed_presentations": input.completed_presentation_count(),
                "terminal_stop": input.terminal_stop().map(|stop| format!("{stop:?}")),
                "last_pointer_button": input.last_pointer_button().map(|button| {
                    serde_json::json!({
                        "sequence": button.sequence(),
                        "event_tick": button.event_tick(),
                        "x_subpixels": button.x_subpixels(),
                        "y_subpixels": button.y_subpixels(),
                        "coordinate_space": format!("{:?}", button.coordinate_space()),
                        "coordinate_unit": format!("{:?}", button.coordinate_unit()),
                    })
                }),
                "last_vertical_scroll": input.last_vertical_scroll().map(|scroll| {
                    serde_json::json!({
                        "sequence": scroll.sequence(),
                        "event_tick": scroll.event_tick(),
                        "x_subpixels": scroll.x_subpixels(),
                        "y_subpixels": scroll.y_subpixels(),
                    })
                }),
                "last_horizontal_scroll": input.last_horizontal_scroll().map(|scroll| {
                    serde_json::json!({
                        "sequence": scroll.sequence(),
                        "event_tick": scroll.event_tick(),
                        "x_subpixels": scroll.x_subpixels(),
                        "y_subpixels": scroll.y_subpixels(),
                    })
                }),
            }),
        );
        fields.insert(
            "runtime_ingress".to_owned(),
            serde_json::json!({
                "applied_batches": ingress[0],
                "duplicate_batches": ingress[1],
                "quarantined_batches": ingress[2],
                "denied_batches": ingress[3],
                "drain_denied": ingress[4],
                "typed_disposition_count": ingress[..4].iter().sum::<u64>(),
            }),
        );
    }
    evidence
}
