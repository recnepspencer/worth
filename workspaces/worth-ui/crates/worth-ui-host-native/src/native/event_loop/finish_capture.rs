use super::{
    UiNativeClientPresentationAttribution, UiNativeEventLoopApplication, UiNativeEventLoopClient,
};

pub(super) struct UiNativeEventLoopFinishCapture {
    pub final_frame: Option<crate::native::UiNativeRetainedFrameObservation>,
    pub graphics: Option<crate::native::UiNativeGraphicsObservation>,
    pub client_attribution: Option<UiNativeClientPresentationAttribution>,
    pub effect_posture: crate::native::UiNativeEffectPosture,
    pub host_peak_census: crate::native::UiNativeResourceCensus,
    pub retained_frames: Vec<crate::native::UiNativeRetainedFrameObservation>,
    pub peak_text_pins: Box<[crate::native::text_atlas::UiNativeTextPinObservation]>,
    pub input_observations: crate::native::UiNativeInputObservationReport,
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
    pub observation_history_complete: bool,
    pub text_atlas_transactions: u64,
    pub derived_state_reconstruction: Option<crate::UiNativeDerivedStateReconstructionObservation>,
}

pub(super) fn capture<Client: UiNativeEventLoopClient>(
    application: &UiNativeEventLoopApplication<Client>,
) -> UiNativeEventLoopFinishCapture {
    let state = application.shared.borrow();
    UiNativeEventLoopFinishCapture {
        final_frame: state.last_retained_frame.clone(),
        graphics: state.presentation_access().as_ref().map(|access| {
            crate::native::UiNativeGraphicsObservation::from_presentation_access(access)
        }),
        client_attribution: application
            .client
            .as_ref()
            .zip(
                state
                    .last_retained_frame
                    .as_ref()
                    .and_then(|frame| frame.presentation()),
            )
            .and_then(|(client, observed)| client.presentation_attribution(observed)),
        effect_posture: state.lifecycle.effect_posture(),
        host_peak_census: state.compiler_total_peak(),
        retained_frames: state.retained_frame_observations.clone(),
        peak_text_pins: state.peak_text_pins.clone(),
        input_observations: state.lifecycle.input_report(),
        text_pin_frame_counts: state.text_pin_frame_counts.clone().into_boxed_slice(),
        text_pin_frame_observations: state.text_pin_frame_observations.clone().into_boxed_slice(),
        text_atlas_model_frame_digests: state
            .text_atlas_model_frame_digests
            .clone()
            .into_boxed_slice(),
        text_atlas_plan_observations: state
            .text_atlas_plan_observations
            .clone()
            .into_boxed_slice(),
        physical_signal_transition_observations: state
            .physical_signal
            .transition_observations()
            .to_vec()
            .into_boxed_slice(),
        physical_signal_transition_trace_complete: state
            .physical_signal
            .transition_observation_trace_complete(),
        physical_signal_lifecycle: state.physical_signal.lifecycle_observation(),
        observation_history_complete: !state.observation_history_overflowed,
        text_atlas_transactions: state.text_atlas.committed_transactions(),
        derived_state_reconstruction: state.certified_derived_state_reconstruction(),
    }
}
