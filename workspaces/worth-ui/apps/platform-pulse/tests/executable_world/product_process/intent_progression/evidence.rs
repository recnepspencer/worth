use crate::adjudication::VisibleControlPixelChange;
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentAttemptObservationReference;
use worth_ui_platform_pulse::observation_contract::PlatformPulseIntentCausalTraceObservation;

#[derive(Debug)]
pub(crate) struct PlatformPulseIntentJourneyEvidence {
    native_activation_count: u32,
    source_action_count: u32,
    provider_start_count: u32,
    completion_count: u32,
    query_action_count: u32,
    visible_posture_count: u32,
    causal_pixel_count: u32,
    minimum_changed_control_pixels: usize,
    minimum_causal_changed_control_pixels: usize,
    final_lifecycle_sequence: u64,
    first_attempt: PlatformPulseIntentAttemptObservationReference,
    second_attempt: PlatformPulseIntentAttemptObservationReference,
    cancelled_attempt: PlatformPulseIntentAttemptObservationReference,
    first_causal_trace: PlatformPulseIntentCausalTraceObservation,
}

#[derive(Debug)]
pub(crate) struct PlatformPulseIntentCausalPulseEvidence {
    trace: PlatformPulseIntentCausalTraceObservation,
    changed_control_pixels: usize,
    final_lifecycle_sequence: u64,
}

#[derive(Default)]
pub(super) struct PlatformPulseIntentJourneyEvidenceBuilder {
    native_activation_count: u32,
    source_action_count: u32,
    provider_start_count: u32,
    completion_count: u32,
    query_action_count: u32,
    visible_posture_count: u32,
    causal_pixel_count: u32,
    minimum_changed_control_pixels: Option<usize>,
    minimum_causal_changed_control_pixels: Option<usize>,
    final_lifecycle_sequence: u64,
    first_attempt: Option<PlatformPulseIntentAttemptObservationReference>,
    second_attempt: Option<PlatformPulseIntentAttemptObservationReference>,
    cancelled_attempt: Option<PlatformPulseIntentAttemptObservationReference>,
    pending_causal_trace: Option<PlatformPulseIntentCausalTraceObservation>,
    first_causal_trace: Option<PlatformPulseIntentCausalTraceObservation>,
}

impl PlatformPulseIntentJourneyEvidenceBuilder {
    pub(super) fn record_native_activation(&mut self) {
        self.native_activation_count += 1;
    }

    pub(super) fn record_source_action(&mut self) {
        self.source_action_count += 1;
    }

    pub(super) fn record_provider_start(&mut self) {
        self.provider_start_count += 1;
    }

    pub(super) fn record_completion(&mut self) {
        self.completion_count += 1;
    }

    pub(super) fn record_query_action(&mut self) {
        self.query_action_count += 1;
    }

    pub(super) fn record_visible_change(&mut self, change: VisibleControlPixelChange) {
        self.visible_posture_count += 1;
        self.minimum_changed_control_pixels = Some(
            self.minimum_changed_control_pixels
                .map_or(change.differing_pixels(), |minimum| {
                    minimum.min(change.differing_pixels())
                }),
        );
    }

    pub(super) fn record_causal_trace(
        &mut self,
        trace: PlatformPulseIntentCausalTraceObservation,
    ) -> bool {
        if self.pending_causal_trace.is_some() {
            return false;
        }
        self.pending_causal_trace = Some(trace);
        true
    }

    pub(super) fn record_causal_visible_change(
        &mut self,
        change: VisibleControlPixelChange,
    ) -> bool {
        let Some(trace) = self.pending_causal_trace.take() else {
            return false;
        };
        self.record_visible_change(change);
        self.causal_pixel_count += 1;
        self.minimum_causal_changed_control_pixels = Some(
            self.minimum_causal_changed_control_pixels
                .map_or(change.differing_pixels(), |minimum| {
                    minimum.min(change.differing_pixels())
                }),
        );
        if self.first_causal_trace.is_none() {
            self.first_causal_trace = Some(trace);
        }
        true
    }

    pub(super) fn record_sequence(&mut self, sequence: u64) {
        self.final_lifecycle_sequence = sequence;
    }

    pub(super) fn record_first_attempt(
        &mut self,
        attempt: PlatformPulseIntentAttemptObservationReference,
    ) {
        self.first_attempt = Some(attempt);
    }

    pub(super) fn record_second_attempt(
        &mut self,
        attempt: PlatformPulseIntentAttemptObservationReference,
    ) {
        self.second_attempt = Some(attempt);
    }

    pub(super) fn record_cancelled_attempt(
        &mut self,
        attempt: PlatformPulseIntentAttemptObservationReference,
    ) {
        self.cancelled_attempt = Some(attempt);
    }

    pub(super) fn finish(self) -> Option<PlatformPulseIntentJourneyEvidence> {
        if self.pending_causal_trace.is_some() {
            return None;
        }
        Some(PlatformPulseIntentJourneyEvidence {
            native_activation_count: self.native_activation_count,
            source_action_count: self.source_action_count,
            provider_start_count: self.provider_start_count,
            completion_count: self.completion_count,
            query_action_count: self.query_action_count,
            visible_posture_count: self.visible_posture_count,
            causal_pixel_count: self.causal_pixel_count,
            minimum_changed_control_pixels: self.minimum_changed_control_pixels?,
            minimum_causal_changed_control_pixels: self.minimum_causal_changed_control_pixels?,
            final_lifecycle_sequence: self.final_lifecycle_sequence,
            first_attempt: self.first_attempt?,
            second_attempt: self.second_attempt?,
            cancelled_attempt: self.cancelled_attempt?,
            first_causal_trace: self.first_causal_trace?,
        })
    }

    pub(super) fn finish_causal_pulse(self) -> Option<PlatformPulseIntentCausalPulseEvidence> {
        if self.pending_causal_trace.is_some() || self.causal_pixel_count != 1 {
            return None;
        }
        Some(PlatformPulseIntentCausalPulseEvidence {
            trace: self.first_causal_trace?,
            changed_control_pixels: self.minimum_causal_changed_control_pixels?,
            final_lifecycle_sequence: self.final_lifecycle_sequence,
        })
    }
}

impl PlatformPulseIntentCausalPulseEvidence {
    pub(crate) fn trace(&self) -> &PlatformPulseIntentCausalTraceObservation {
        &self.trace
    }

    pub(crate) fn changed_control_pixels(&self) -> usize {
        self.changed_control_pixels
    }

    pub(crate) fn expected_shutdown_sequence(&self) -> u64 {
        self.final_lifecycle_sequence + 1
    }
}

impl PlatformPulseIntentJourneyEvidence {
    pub(crate) fn native_activation_count(&self) -> u32 {
        self.native_activation_count
    }

    pub(crate) fn source_action_count(&self) -> u32 {
        self.source_action_count
    }

    pub(crate) fn provider_start_count(&self) -> u32 {
        self.provider_start_count
    }

    pub(crate) fn completion_count(&self) -> u32 {
        self.completion_count
    }

    pub(crate) fn query_action_count(&self) -> u32 {
        self.query_action_count
    }

    pub(crate) fn visible_posture_count(&self) -> u32 {
        self.visible_posture_count
    }

    pub(crate) fn minimum_changed_control_pixels(&self) -> usize {
        self.minimum_changed_control_pixels
    }

    pub(crate) fn causal_pixel_count(&self) -> u32 {
        self.causal_pixel_count
    }

    pub(crate) fn minimum_causal_changed_control_pixels(&self) -> usize {
        self.minimum_causal_changed_control_pixels
    }

    pub(crate) fn first_causal_trace(&self) -> &PlatformPulseIntentCausalTraceObservation {
        &self.first_causal_trace
    }

    pub(crate) fn expected_shutdown_sequence(&self) -> u64 {
        self.final_lifecycle_sequence + 1
    }

    pub(crate) fn attempts_are_distinct(&self) -> bool {
        self.first_attempt != self.second_attempt
            && self.first_attempt != self.cancelled_attempt
            && self.second_attempt != self.cancelled_attempt
    }
}
