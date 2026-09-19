use super::*;

pub(in crate::courtroom) struct OpenPlatformPulseJourney {
    pub(super) recovered: PulseExecutableWorld<Published<FinalRecovered>>,
    pub(super) first_publication: Duration,
    pub(super) native_captures: u32,
    pub(super) window_lookups: u32,
}

pub(in crate::courtroom) fn complete_open(
    deltas: PlatformPulseJourneyDeltas,
) -> OpenPlatformPulseJourney {
    complete_open_with_causal_actions(deltas, None, NATIVE_INITIALIZATION_DEADLINE, None)
}

pub(super) fn complete_open_for_manifest(
    deltas: PlatformPulseJourneyDeltas,
    cursor: &mut crate::source_delta::PulseCausalActionCursor<'_>,
    manifest: &crate::source_delta::PulseCausalActionManifest,
    installation_path: &crate::installation::PulseInstallationPath,
) -> OpenPlatformPulseJourney {
    assert_eq!(manifest.transition_deadline(), TRANSITION_DEADLINE);
    complete_open_with_causal_actions(
        deltas,
        Some(cursor),
        manifest.first_frame_deadline(),
        Some(installation_path),
    )
}

fn complete_open_with_causal_actions(
    deltas: PlatformPulseJourneyDeltas,
    mut cursor: Option<&mut crate::source_delta::PulseCausalActionCursor<'_>>,
    first_frame_deadline: Duration,
    installation_path: Option<&crate::installation::PulseInstallationPath>,
) -> OpenPlatformPulseJourney {
    let report_cost = cursor.is_some();
    advance(&mut cursor, &["launch"]);
    let initial = launch_initial(deltas.canonical, first_frame_deadline, installation_path);
    let journey_started = initial.native_journey_started();
    advance(&mut cursor, &["observe-first-frame"]);
    let first_publication = initial.launch_to_first_publication();
    report_checkpoint(report_cost, "first-publication", journey_started);
    let mut native_captures = initial.evidence().capture_count();
    let window_lookups = initial.evidence().client_area().window_lookup_count();
    let input_reached = super::super::platform_pulse_input::reach_native_input_observed(
        initial,
        Instant::now() + TRANSITION_DEADLINE,
        |step| advance(&mut cursor, &[native_input_manifest_action(step)]),
    );
    native_captures += input_reached.evidence().capture_count();
    advance(&mut cursor, &["publish-query-v1"]);
    let first_current = publish_first_current(input_reached);
    advance(&mut cursor, &["observe-query-v1"]);
    native_captures += first_current.query_evidence().pixels().capture_count();
    let (visualized, visual_captures) =
        publish_visual_identity(first_current, cursor.as_deref_mut());
    native_captures += visual_captures;
    advance(&mut cursor, &["publish-query-v2"]);
    let second_current = publish_second_current(visualized);
    advance(&mut cursor, &["observe-query-v2"]);
    native_captures += second_current.query_evidence().pixels().capture_count();
    advance(&mut cursor, &["edit-green-source"]);
    let green = publish_green(second_current, deltas.green);
    advance(&mut cursor, &["observe-green-successor"]);
    native_captures += green.evidence().capture_count();
    advance(&mut cursor, &["edit-malformed-source"]);
    let preserved = preserve_green(green, deltas.malformed);
    advance(&mut cursor, &["observe-predecessor-preserved"]);
    report_checkpoint(report_cost, "visual-source-denied", journey_started);
    native_captures += preserved.evidence().capture_count();
    advance(&mut cursor, &["edit-canonical-blue-recovery"]);
    let recovered = recover_blue(preserved, deltas.recovery);
    advance(&mut cursor, &["observe-blue-recovery"]);
    report_checkpoint(report_cost, "visual-source-recovered", journey_started);
    native_captures += recovered.evidence().capture_count();
    advance(&mut cursor, &["edit-revision-schema"]);
    let stopped = stop_on_revision_schema(recovered, deltas.revision_schema);
    advance(&mut cursor, &["observe-schema-stop"]);
    report_checkpoint(report_cost, "revision-schema-denied", journey_started);
    native_captures += stopped.evidence().replacement().capture_count();
    advance(&mut cursor, &["edit-status-schema-recovery"]);
    let recovered = recover_status_schema(stopped, deltas.status_schema_recovery);
    advance(&mut cursor, &["observe-status-schema-recovery"]);
    report_checkpoint(report_cost, "revision-schema-recovered", journey_started);
    native_captures += recovered.evidence().replacement().capture_count();
    OpenPlatformPulseJourney {
        recovered,
        first_publication,
        native_captures,
        window_lookups,
    }
}

fn report_checkpoint(enabled: bool, phase: &str, journey_started: Instant) {
    if enabled {
        eprintln!(
            "WORTH_UI_EXECUTABLE_WORLD_PHASE phase={phase} elapsed_ms={}",
            journey_started.elapsed().as_millis()
        );
    }
}

fn native_input_manifest_action(
    step: crate::product_process::NativeInputCausalStep,
) -> &'static str {
    match step {
        crate::product_process::NativeInputCausalStep::PointerDelivered => {
            "deliver-preintent-pointer"
        }
        crate::product_process::NativeInputCausalStep::PointerObserved => {
            "observe-preintent-pointer"
        }
        crate::product_process::NativeInputCausalStep::KeyboardDelivered => {
            "deliver-preintent-keyboard"
        }
        crate::product_process::NativeInputCausalStep::KeyboardObserved => {
            "observe-preintent-keyboard"
        }
    }
}

fn advance(
    cursor: &mut Option<&mut crate::source_delta::PulseCausalActionCursor<'_>>,
    actions: &[&'static str],
) {
    let Some(cursor) = cursor.as_deref_mut() else {
        return;
    };
    for action in actions {
        cursor
            .advance(action)
            .unwrap_or_else(|failure| panic!("advance Pulse causal manifest: {failure}"));
    }
}

impl OpenPlatformPulseJourney {
    pub(in crate::courtroom) fn into_recovered(
        self,
    ) -> PulseExecutableWorld<Published<FinalRecovered>> {
        self.recovered
    }
}
