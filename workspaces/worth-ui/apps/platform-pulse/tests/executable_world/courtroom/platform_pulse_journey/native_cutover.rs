use std::path::PathBuf;

use crate::installation::{CanonicalPlatformPulse, PulseInstallationPath};
use crate::product_process::{Closed, PulseExecutableWorld};
use crate::source_delta::{
    IntentRouteRemovalSourceDelta, PulseCausalActionCursor, PulseCausalActionManifest,
};

use super::open::complete_open_for_manifest;
use super::{JourneyCostInputs, PlatformPulseJourneyCost, PlatformPulseJourneyDeltas};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PulseNativeCutoverEvidence {
    installation_root: PathBuf,
    manifest_digest: [u8; 32],
    manifest_action_count: usize,
    lifecycle_envelopes: Vec<
        worth_ui_platform_pulse::observation_contract::PlatformPulseLifecycleObservationEnvelope,
    >,
    quiescence: QuiescenceEvidence,
    external_close: ExternalCloseEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PulseNativeCutoverVerdict {
    event_count: usize,
    process_id: u32,
    exit_poll_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ExternalCloseEvidence {
    close_request: crate::external_observation::NormalNativeCloseRequestObservation,
    successful_exit: crate::product_process::SuccessfulPlatformPulseExit,
    installation_cleanup: crate::installation::PulseInstallationCleanupEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QuiescenceEvidence {
    lifecycle_event_delta: usize,
    native_capture_count: u32,
    process_liveness_checks: u32,
    pixels_unchanged: bool,
}

pub(crate) struct CompletedPulseNativeCutoverRun {
    closed: PulseExecutableWorld<Closed>,
    cost: PlatformPulseJourneyCost,
    evidence: PulseNativeCutoverEvidence,
}

pub(super) fn complete(
    deltas: PlatformPulseJourneyDeltas,
    manifest: &PulseCausalActionManifest,
    installation_path: &PulseInstallationPath,
) -> CompletedPulseNativeCutoverRun {
    let route_removal =
        IntentRouteRemovalSourceDelta::from_checked_in(CanonicalPlatformPulse::checked_in())
            .expect("canonical Pulse contains exactly one typed action route");
    let mut cursor = manifest.cursor();
    let open = complete_open_for_manifest(deltas, &mut cursor, manifest, installation_path);
    let journey_started = open.recovered.native_journey_started();
    let open_source_actions = open.recovered.source_action_count();
    let first_publication = open.first_publication;
    let window_lookups = open.window_lookups;
    let open_captures = open.native_captures;
    let completed = open
        .recovered
        .complete_intent_journey_for_manifest(route_removal, &mut cursor)
        .unwrap_or_else(|failure| {
            panic!("cumulative native intent journey reaches its visible consequence: {failure}")
        });
    report_checkpoint("intent-complete", journey_started);
    let intent = completed.evidence();
    assert_complete_intent_evidence(intent);
    let source_action_count = open_source_actions.saturating_add(intent.source_action_count());
    let mut native_capture_count = open_captures
        .saturating_add(1)
        .saturating_add(intent.visible_posture_count());
    let expected_shutdown_sequence = intent.expected_shutdown_sequence();
    let recovered = completed.into_recovered();
    advance(&mut cursor, "begin-quiescent-interval");
    let (recovered, quiescence) = recovered
        .observe_quiescent(manifest.idle_interval())
        .unwrap_or_else(|failure| panic!("observe product quiescence: {failure}"));
    report_checkpoint("quiescence-complete", journey_started);
    advance(&mut cursor, "observe-quiescent-interval");
    assert!(quiescence.observed_for() >= manifest.idle_interval());
    assert_eq!(quiescence.lifecycle_event_delta(), 0);
    assert_eq!(quiescence.process_liveness_checks(), 2);
    assert!(quiescence.pixels_unchanged());
    native_capture_count = native_capture_count.saturating_add(quiescence.native_capture_count());
    advance(&mut cursor, "close-window");
    let closed = super::super::platform_pulse_cleanup::close_recovered_at_sequence(
        recovered,
        expected_shutdown_sequence,
    );
    report_checkpoint("close-complete", journey_started);
    advance(&mut cursor, "observe-exact-zero-shutdown");
    cursor
        .finish()
        .unwrap_or_else(|failure| panic!("finish Pulse causal manifest: {failure}"));
    let cleanup = closed.evidence();
    let evidence = PulseNativeCutoverEvidence {
        installation_root: installation_path.root().to_path_buf(),
        manifest_digest: manifest.digest(),
        manifest_action_count: manifest.action_count(),
        lifecycle_envelopes: cleanup.lifecycle_envelopes().to_vec(),
        quiescence: QuiescenceEvidence {
            lifecycle_event_delta: quiescence.lifecycle_event_delta(),
            native_capture_count: quiescence.native_capture_count(),
            process_liveness_checks: quiescence.process_liveness_checks(),
            pixels_unchanged: quiescence.pixels_unchanged(),
        },
        external_close: ExternalCloseEvidence {
            close_request: cleanup.close_request(),
            successful_exit: cleanup.successful_exit(),
            installation_cleanup: cleanup.installation_cleanup(),
        },
    };
    let cost = PlatformPulseJourneyCost::from_completed(
        JourneyCostInputs {
            first_publication,
            full_journey: journey_started.elapsed(),
            source_actions: source_action_count,
            native_captures: native_capture_count,
            window_lookups,
        },
        cleanup,
    );
    CompletedPulseNativeCutoverRun {
        closed,
        cost,
        evidence,
    }
}

fn report_checkpoint(phase: &str, journey_started: std::time::Instant) {
    eprintln!(
        "WORTH_UI_EXECUTABLE_WORLD_PHASE phase={phase} elapsed_ms={}",
        journey_started.elapsed().as_millis()
    );
}

fn advance(cursor: &mut PulseCausalActionCursor<'_>, action: &'static str) {
    cursor
        .advance(action)
        .unwrap_or_else(|failure| panic!("advance Pulse causal manifest: {failure}"));
}

fn assert_complete_intent_evidence(
    evidence: &crate::product_process::PlatformPulseIntentJourneyEvidence,
) {
    assert_eq!(evidence.native_activation_count(), 8);
    assert_eq!(evidence.source_action_count(), 7);
    assert_eq!(evidence.provider_start_count(), 3);
    assert_eq!(evidence.completion_count(), 2);
    assert_eq!(evidence.query_action_count(), 2);
    assert_eq!(evidence.visible_posture_count(), 10);
    assert!(evidence.minimum_changed_control_pixels() >= 9);
    assert_eq!(evidence.causal_pixel_count(), 2);
    assert!(evidence.minimum_causal_changed_control_pixels() >= 9);
    assert!(evidence
        .first_causal_trace()
        .outcome()
        .consequence_published());
    assert!(evidence.attempts_are_distinct());
}

impl CompletedPulseNativeCutoverRun {
    pub(crate) const fn evidence(&self) -> &PulseNativeCutoverEvidence {
        &self.evidence
    }

    pub(crate) fn closed(&self) -> &PulseExecutableWorld<Closed> {
        &self.closed
    }

    pub(crate) const fn cost(&self) -> PlatformPulseJourneyCost {
        self.cost
    }
}

impl PulseNativeCutoverEvidence {
    pub(crate) fn validate(&self) -> Result<PulseNativeCutoverVerdict, String> {
        if !self.installation_root.is_absolute()
            || self.manifest_digest == [0; 32]
            || self.manifest_action_count == 0
        {
            return Err("native cutover omitted installation or causal-manifest identity".into());
        }
        if self.lifecycle_envelopes.is_empty() {
            return Err("native cutover emitted no lifecycle evidence".into());
        }
        if self.quiescence.lifecycle_event_delta != 0
            || self.quiescence.native_capture_count != 2
            || self.quiescence.process_liveness_checks != 2
            || !self.quiescence.pixels_unchanged
        {
            return Err(format!("native quiescence drifted: {:?}", self.quiescence));
        }
        let close = self.external_close;
        if close.close_request.request_count() != 1
            || close.close_request.process_id() == 0
            || !close.successful_exit.status().success()
            || close.successful_exit.poll_count() == 0
            || !close.installation_cleanup.removed_owned_root()
        {
            return Err(format!("native close or cleanup drifted: {close:?}"));
        }
        Ok(PulseNativeCutoverVerdict {
            event_count: self.lifecycle_envelopes.len(),
            process_id: close.close_request.process_id(),
            exit_poll_count: close.successful_exit.poll_count(),
        })
    }
}

impl PulseNativeCutoverVerdict {
    pub(crate) const fn event_count(self) -> usize {
        self.event_count
    }

    pub(crate) const fn process_id(self) -> u32 {
        self.process_id
    }

    pub(crate) const fn exit_poll_count(self) -> u32 {
        self.exit_poll_count
    }
}
