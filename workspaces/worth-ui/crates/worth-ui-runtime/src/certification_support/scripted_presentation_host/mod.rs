use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};

use crate::facade::mounted::{
    UiHostSurfaceCancellationOutcome, UiHostSurfaceInFlightCompletion,
    UiHostSurfacePresentationMode, UiHostSurfacePresentationOutcome, UiMountedCompletedEffects,
    UiMountedEffectFamily,
};
use worth_ui_host_contract::WorthUiHostCapabilityReport;

mod adapter;
mod measurement_adapter;
mod visual_capture_script;

use visual_capture_script::ScriptedVisualCapture;

#[derive(Clone, Default)]
pub struct ScriptedPresentationHost {
    state: Arc<Mutex<ScriptedPresentationState>>,
    observation_retention: Arc<worth_ui_host_contract::UiHostObservationRetention>,
}

enum ScriptedPresentationStart {
    Outcome(UiHostSurfacePresentationOutcome),
    InFlight {
        completions: VecDeque<ScriptedSurfaceCompletion>,
        cancellation: UiHostSurfaceCancellationOutcome,
    },
}

pub enum ScriptedSurfaceCompletion {
    Pending,
    RejectedBeforeEffects(worth_ui_host_contract::UiHostSurfacePresentationDenial),
    Presented(worth_ui_host_contract::UiMountedSurfacePresentationCompletion),
    Superseded(worth_ui_host_contract::UiMountedSurfacePresentationSupersession),
    PresentationIndeterminate,
}

struct ScriptedPresentationState {
    contract: worth_ui_host_contract::WorthUiHostContract,
    protocol: worth_ui_host_contract::UiHostProtocolContract,
    capabilities: WorthUiHostCapabilityReport,
    presentations: VecDeque<ScriptedPresentationStart>,
    completions: BTreeMap<u64, VecDeque<ScriptedSurfaceCompletion>>,
    cancellations: BTreeMap<u64, UiHostSurfaceCancellationOutcome>,
    token_sessions: BTreeMap<u64, u64>,
    registrations: BTreeMap<
        worth_ui_host_contract::UiHostSurfaceIdentity,
        worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
    >,
    indeterminate_next_registration: bool,
    wrong_next_deregistration_receipt: bool,
    cancellation_calls: Vec<u64>,
    presentation_calls: usize,
    last_filled_rect_colors: Vec<worth_ui_host_contract::UiMountedRgba8>,
    last_presentation_correlation:
        Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    next_physical_presentation_sequence: u64,
    last_focus_placement: Option<worth_ui_host_contract::UiHostFocusPlacementRequest>,
    requested_portal_overlay_commands:
        HashSet<worth_ui_host_contract::UiMountedPaintCommandIdentity>,
    requested_portal_overlay_counts: Vec<usize>,
    reconstruction_portal_overlay_counts: Vec<usize>,
    input_recipient: Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt>,
    viewport_extent: [f32; 2],
    viewport_measurement_calls: usize,
    viewport_environment_generation: u64,
    font_environment_generation: u64,
    adapter_environment_generation: u64,
    queued_observation: Option<crate::facade::observation_report::UiHostObservationBatch>,
    queued_measurement: Option<(
        crate::facade::measurement_exchange::WorthUiHostMeasurementIngress,
        crate::facade::measurement_exchange::UiHostMeasurementCompletion,
    )>,
    observation_events: Vec<&'static str>,
    visual_capture_capability: worth_ui_host_contract::UiHostCaptureCapability,
    visual_captures: VecDeque<ScriptedVisualCapture>,
    visual_capture_calls: Vec<worth_ui_host_contract::UiHostVisualCaptureRequest>,
    visual_cancellation_outcome: worth_ui_host_contract::UiHostCaptureCancellationOutcome,
    visual_cancellation_calls: Vec<worth_ui_host_contract::UiHostVisualCaptureRequest>,
}

impl Default for ScriptedPresentationState {
    fn default() -> Self {
        Self {
            contract: worth_ui_host_contract::WorthUiHostContract::headless(),
            protocol: worth_ui_host_contract::UiHostProtocolContract::current(),
            capabilities: recording_capabilities(),
            presentations: VecDeque::new(),
            completions: BTreeMap::new(),
            cancellations: BTreeMap::new(),
            token_sessions: BTreeMap::new(),
            registrations: BTreeMap::new(),
            indeterminate_next_registration: false,
            wrong_next_deregistration_receipt: false,
            cancellation_calls: Vec::new(),
            presentation_calls: 0,
            last_filled_rect_colors: Vec::new(),
            last_presentation_correlation: None,
            next_physical_presentation_sequence: 1,
            last_focus_placement: None,
            requested_portal_overlay_commands: HashSet::new(),
            requested_portal_overlay_counts: Vec::new(),
            reconstruction_portal_overlay_counts: Vec::new(),
            input_recipient: None,
            viewport_extent: [800.0, 600.0],
            viewport_measurement_calls: 0,
            viewport_environment_generation: 1,
            font_environment_generation: 1,
            adapter_environment_generation: 1,
            queued_observation: None,
            queued_measurement: None,
            observation_events: Vec::new(),
            visual_capture_capability: worth_ui_host_contract::UiHostCaptureCapability::Unsupported,
            visual_captures: VecDeque::new(),
            visual_capture_calls: Vec::new(),
            visual_cancellation_outcome:
                worth_ui_host_contract::UiHostCaptureCancellationOutcome::CancelledBeforeReadback,
            visual_cancellation_calls: Vec::new(),
        }
    }
}

pub fn recorded_effects() -> UiMountedCompletedEffects {
    UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::RecordedProjection])
}

pub const fn scripted_presentation_epoch() -> worth_ui_host_contract::UiHostPresentationEpoch {
    worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(1)
}

fn scripted_presentation_cost() -> worth_ui_host_contract::UiHostPresentationCostReport {
    worth_ui_host_contract::UiHostPresentationCostReport::from_adapter(
        worth_ui_host_contract::UiHostPresentationCostInput {
            presented_surfaces: 1,
            translated_rows: 0,
            translated_bytes: 0,
            native_resource_cache_hits: 0,
            native_resource_cache_misses: 0,
            asynchronous_handoffs: 0,
            ..Default::default()
        },
    )
}

pub fn presented_completion() -> ScriptedSurfaceCompletion {
    ScriptedSurfaceCompletion::Presented(
        worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::RecordOnly,
            scripted_presentation_epoch(),
            recorded_effects(),
            scripted_presentation_cost(),
        ),
    )
}

impl ScriptedPresentationHost {
    pub fn native_display() -> Self {
        let host = Self::default();
        {
            let mut state = host.state.lock().unwrap();
            state.contract = worth_ui_host_contract::WorthUiHostContract::native();
            state.capabilities = WorthUiHostCapabilityReport::available(vec![
                worth_ui_host_contract::WorthUiHostCapability::NativePaint,
                worth_ui_host_contract::WorthUiHostCapability::ViewportObservation,
                worth_ui_host_contract::WorthUiHostCapability::DpiObservation,
            ]);
        }
        host
    }

    pub fn set_protocol(&self, protocol: worth_ui_host_contract::UiHostProtocolContract) {
        self.state.lock().unwrap().protocol = protocol;
    }

    pub fn set_capabilities(&self, capabilities: WorthUiHostCapabilityReport) {
        self.state.lock().unwrap().capabilities = capabilities;
    }

    pub fn push_presented(&self) {
        self.push_presentation(UiHostSurfacePresentationOutcome::Presented(
            worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::RecordOnly,
                scripted_presentation_epoch(),
                recorded_effects(),
                scripted_presentation_cost(),
            ),
        ));
    }

    pub fn push_native_display_presented(&self) {
        self.push_presentation(UiHostSurfacePresentationOutcome::Presented(
            worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                scripted_presentation_epoch(),
                UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::NativePaint]),
                scripted_presentation_cost(),
            ),
        ));
    }

    pub fn push_native_display_settled_without_effects(&self) {
        self.push_presentation(UiHostSurfacePresentationOutcome::Presented(
            worth_ui_host_contract::UiMountedSurfacePresentationCompletion::new(
                UiHostSurfacePresentationMode::NativeDisplay,
                scripted_presentation_epoch(),
                UiMountedCompletedEffects::new(Vec::new()),
                worth_ui_host_contract::UiHostPresentationCostReport::default(),
            ),
        ));
    }

    pub fn push_rejected(&self) {
        self.push_presentation(UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
            worth_ui_host_contract::UiHostSurfacePresentationDenial::AdapterDeclined,
        ));
    }

    pub fn push_presentation(&self, outcome: UiHostSurfacePresentationOutcome) {
        self.state
            .lock()
            .unwrap()
            .presentations
            .push_back(ScriptedPresentationStart::Outcome(outcome));
    }

    pub fn push_in_flight(
        &self,
        completions: Vec<ScriptedSurfaceCompletion>,
        cancellation: UiHostSurfaceCancellationOutcome,
    ) {
        self.state
            .lock()
            .unwrap()
            .presentations
            .push_back(ScriptedPresentationStart::InFlight {
                completions: completions.into_iter().collect(),
                cancellation,
            });
    }

    pub fn presentation_calls(&self) -> usize {
        self.state.lock().unwrap().presentation_calls
    }

    pub fn last_filled_rect_colors(&self) -> Vec<worth_ui_host_contract::UiMountedRgba8> {
        self.state.lock().unwrap().last_filled_rect_colors.clone()
    }

    pub fn last_presentation_correlation(
        &self,
    ) -> Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation> {
        self.state.lock().unwrap().last_presentation_correlation
    }

    pub fn last_focus_placement(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostFocusPlacementRequest> {
        self.state.lock().unwrap().last_focus_placement
    }

    pub fn input_recipient(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostInputRecipientBindingReceipt> {
        self.state.lock().unwrap().input_recipient
    }

    pub fn reconstruction_portal_overlay_counts(&self) -> Vec<usize> {
        self.state
            .lock()
            .unwrap()
            .reconstruction_portal_overlay_counts
            .clone()
    }

    pub fn requested_portal_overlay_counts(&self) -> Vec<usize> {
        self.state
            .lock()
            .unwrap()
            .requested_portal_overlay_counts
            .clone()
    }

    pub fn set_viewport_extent(&self, extent: [f32; 2]) {
        self.state.lock().unwrap().viewport_extent = extent;
    }

    pub fn viewport_measurement_calls(&self) -> usize {
        self.state.lock().unwrap().viewport_measurement_calls
    }

    pub fn return_indeterminate_next_registration(&self) {
        self.state.lock().unwrap().indeterminate_next_registration = true;
    }

    pub fn return_wrong_next_deregistration_receipt(&self) {
        self.state.lock().unwrap().wrong_next_deregistration_receipt = true;
    }

    pub fn native_registration_count(&self) -> usize {
        self.state.lock().unwrap().registrations.len()
    }

    pub fn native_in_flight_count(&self) -> usize {
        self.state.lock().unwrap().token_sessions.len()
    }

    pub fn advance_viewport_environment(&self) {
        let mut state = self.state.lock().unwrap();
        state.viewport_environment_generation = state
            .viewport_environment_generation
            .checked_add(1)
            .expect("scripted viewport generation capacity");
    }

    pub fn advance_font_environment(&self) {
        let mut state = self.state.lock().unwrap();
        state.font_environment_generation = state
            .font_environment_generation
            .checked_add(1)
            .expect("scripted font generation capacity");
    }

    pub fn cancellation_calls(&self) -> Vec<u64> {
        self.state.lock().unwrap().cancellation_calls.clone()
    }

    pub fn enqueue_observation_during_next_presentation(
        &self,
        batch: crate::facade::observation_report::UiHostObservationBatch,
    ) {
        self.state.lock().unwrap().queued_observation = Some(batch);
    }

    pub fn enqueue_observation_for_next_drain(
        &self,
        batch: crate::facade::observation_report::UiHostObservationBatch,
    ) {
        self.observation_retention
            .retain(batch)
            .expect("scripted observation fits adapter retention");
    }

    pub fn pending_observation_batch_count(&self) -> usize {
        self.observation_retention.pending_batch_count()
    }

    pub fn enqueue_measurement_during_next_presentation(
        &self,
        ingress: crate::facade::measurement_exchange::WorthUiHostMeasurementIngress,
        completion: crate::facade::measurement_exchange::UiHostMeasurementCompletion,
    ) {
        self.state.lock().unwrap().queued_measurement = Some((ingress, completion));
    }

    pub fn observation_events(&self) -> Vec<&'static str> {
        self.state.lock().unwrap().observation_events.clone()
    }
}

fn recording_capabilities() -> WorthUiHostCapabilityReport {
    WorthUiHostCapabilityReport::available(vec![
        worth_ui_host_contract::WorthUiHostCapability::MountedFrameRecording,
    ])
}
