use super::{
    UiNativeClientPresentationAttribution, UiNativeClientShutdownObservation,
    UiNativeEventLoopRunReport, UiNativeEventLoopShutdownOverlapObservation,
    UiNativeGraphicsObservation, UiNativeInputObservationReport, UiNativePresentationObservation,
    UiNativeResourceCensus, UiNativeRetainedFrameObservation,
};

impl UiNativeEventLoopRunReport {
    pub fn presentation(&self) -> &UiNativePresentationObservation {
        &self.presentation
    }

    pub const fn terminal_census(&self) -> UiNativeResourceCensus {
        self.terminal_census
    }

    pub fn graphics(&self) -> &UiNativeGraphicsObservation {
        &self.graphics
    }

    pub fn event_loop_thread(&self) -> &str {
        &self.event_loop_thread
    }

    pub const fn event_loop_thread_matches_launch(&self) -> bool {
        self.event_loop_thread_matches_launch
    }

    pub const fn event_loop_thread_posture(&self) -> super::super::UiNativeEventLoopThreadPosture {
        self.event_loop_thread_posture
    }

    pub const fn client_attribution(&self) -> UiNativeClientPresentationAttribution {
        self.client_attribution
    }

    pub const fn readiness_signals(&self) -> u64 {
        self.readiness_signals
    }

    pub const fn redraw_turns(&self) -> u64 {
        self.redraw_turns
    }

    pub const fn idle_wait_turns(&self) -> u64 {
        self.idle_wait_turns
    }

    pub const fn coalesced_wakes(&self) -> u64 {
        self.coalesced_wakes
    }

    pub const fn peak_census(&self) -> UiNativeResourceCensus {
        self.peak_census
    }

    pub const fn port_crossings(&self) -> u8 {
        self.port_crossings
    }

    pub const fn client_shutdown(&self) -> Option<&UiNativeClientShutdownObservation> {
        self.client_shutdown.as_ref()
    }

    pub fn input_observations(&self) -> &UiNativeInputObservationReport {
        &self.input_observations
    }

    pub const fn shutdown_overlap(&self) -> UiNativeEventLoopShutdownOverlapObservation {
        self.shutdown_overlap
    }

    pub fn retained_frames(&self) -> &[UiNativeRetainedFrameObservation] {
        &self.retained_frames
    }

    #[doc(hidden)]
    pub fn peak_text_pins(&self) -> &[crate::native::text_atlas::UiNativeTextPinObservation] {
        &self.peak_text_pins
    }

    pub fn peak_text_layout_count(&self) -> usize {
        self.peak_text_pins
            .iter()
            .map(|pin| pin.layout_digest())
            .collect::<std::collections::BTreeSet<_>>()
            .len()
    }

    pub fn text_pin_frame_counts(&self) -> &[u32] {
        &self.text_pin_frame_counts
    }

    #[doc(hidden)]
    pub fn text_pin_frame_observations(
        &self,
    ) -> &[Box<[crate::native::text_atlas::UiNativeTextPinObservation]>] {
        &self.text_pin_frame_observations
    }

    pub fn text_atlas_model_frame_digests(&self) -> &[[u8; 32]] {
        &self.text_atlas_model_frame_digests
    }

    pub const fn observation_history_complete(&self) -> bool {
        self.observation_history_complete
    }

    pub const fn text_atlas_transactions(&self) -> u64 {
        self.text_atlas_transactions
    }

    pub const fn derived_state_reconstruction(
        &self,
    ) -> Option<crate::UiNativeDerivedStateReconstructionObservation> {
        self.derived_state_reconstruction
    }

    pub fn text_atlas_plan_observations(
        &self,
    ) -> &[crate::native::text_atlas::UiNativeTextAtlasPlanObservation] {
        &self.text_atlas_plan_observations
    }

    pub fn physical_signal_transition_observations(
        &self,
    ) -> &[crate::native::physical_work_signal::UiNativePhysicalSignalTransitionObservation] {
        &self.physical_signal_transition_observations
    }

    pub const fn physical_signal_transition_trace_complete(&self) -> bool {
        self.physical_signal_transition_trace_complete
    }

    pub const fn physical_signal_lifecycle(
        &self,
    ) -> crate::native::UiNativePhysicalSignalLifecycleObservation {
        self.physical_signal_lifecycle
    }
}
