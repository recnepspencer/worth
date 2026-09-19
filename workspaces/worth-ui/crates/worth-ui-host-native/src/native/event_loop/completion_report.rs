use super::{
    UiNativeClientPresentationAttribution, UiNativeClientShutdownObservation,
    UiNativeEventLoopApplication, UiNativeEventLoopClient, UiNativeEventLoopRunReport,
};

pub(super) struct UiNativeEventLoopCompletionEvidence {
    pub final_frame: crate::native::UiNativeRetainedFrameObservation,
    pub graphics: crate::native::UiNativeGraphicsObservation,
    pub client_attribution: Option<UiNativeClientPresentationAttribution>,
    pub peak_census: crate::native::UiNativeResourceCensus,
    pub terminal_census: crate::native::UiNativeResourceCensus,
    pub retained_frames: Vec<crate::native::UiNativeRetainedFrameObservation>,
    pub peak_text_pins: Box<[crate::native::text_atlas::UiNativeTextPinObservation]>,
    pub text_pin_frame_counts: Box<[u32]>,
    pub text_pin_frame_observations:
        Box<[Box<[crate::native::text_atlas::UiNativeTextPinObservation]>]>,
    pub text_atlas_model_frame_digests: Box<[[u8; 32]]>,
    pub text_atlas_plan_observations:
        Box<[crate::native::text_atlas::UiNativeTextAtlasPlanObservation]>,
    pub physical_signal_transition_observations:
        Box<[crate::native::physical_work_signal::UiNativePhysicalSignalTransitionObservation]>,
    pub physical_signal_transition_trace_complete: bool,
    pub physical_signal_lifecycle: crate::native::UiNativePhysicalSignalLifecycleObservation,
    pub input_observations: crate::native::UiNativeInputObservationReport,
    pub observation_history_complete: bool,
    pub text_atlas_transactions: u64,
    pub derived_state_reconstruction: Option<crate::UiNativeDerivedStateReconstructionObservation>,
    pub client_shutdown: Option<UiNativeClientShutdownObservation>,
    pub shutdown_overlap: super::UiNativeEventLoopShutdownOverlapObservation,
}

impl<Client: UiNativeEventLoopClient> UiNativeEventLoopApplication<Client> {
    pub(super) fn completed_report(
        self,
        evidence: UiNativeEventLoopCompletionEvidence,
    ) -> UiNativeEventLoopRunReport {
        let thread = self
            .thread_observation
            .expect("validated event-loop thread");
        UiNativeEventLoopRunReport {
            port_crossings: self
                .port_crossings
                .saturating_add(evidence.final_frame.port_crossings()),
            final_frame: evidence.final_frame,
            graphics: evidence.graphics,
            event_loop_thread: format!("{:?}", thread.thread).into_boxed_str(),
            event_loop_thread_matches_launch: thread.matches_launch,
            event_loop_thread_posture: self.thread_posture,
            client_attribution: evidence.client_attribution,
            readiness_signals: self.readiness_signals,
            redraw_turns: self.redraw_turns,
            idle_wait_turns: self.idle_wait_turns,
            coalesced_wakes: self.coalesced_wakes,
            peak_census: evidence.peak_census,
            terminal_census: evidence.terminal_census,
            retained_frames: evidence.retained_frames.into_boxed_slice(),
            peak_text_pins: evidence.peak_text_pins,
            text_pin_frame_counts: evidence.text_pin_frame_counts,
            text_pin_frame_observations: evidence.text_pin_frame_observations,
            text_atlas_model_frame_digests: evidence.text_atlas_model_frame_digests,
            text_atlas_plan_observations: evidence.text_atlas_plan_observations,
            physical_signal_transition_observations: evidence
                .physical_signal_transition_observations,
            physical_signal_transition_trace_complete: evidence
                .physical_signal_transition_trace_complete,
            physical_signal_lifecycle: evidence.physical_signal_lifecycle,
            input_observations: evidence.input_observations,
            observation_history_complete: evidence.observation_history_complete,
            text_atlas_transactions: evidence.text_atlas_transactions,
            derived_state_reconstruction: evidence.derived_state_reconstruction,
            client_shutdown: evidence.client_shutdown,
            shutdown_overlap: evidence.shutdown_overlap,
        }
    }
}
