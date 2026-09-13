use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use worth_ui_host_contract::{
    UiHostMeasurementObservationValue, UiHostMeasurementRequest, UiHostProtocolNegotiation,
    UiHostSessionReleaseOutcome, UiHostSessionReleaseReceipt, UiHostSurfacePresentationDenial,
    UiHostSurfacePresentationMode, UiHostSurfacePresentationOutcome,
    UiHostSurfaceRegistrationDenial, UiHostSurfaceRegistrationRequest, UiMountedCompletedEffects,
    UiMountedEffectFamily, UiMountedFrameConsumptionView, UiMountedPresentationProductionCost,
    UiMountedSurfacePresentationCompletion, UiViewportExtentObservation,
    WorthUiHostCapabilityReport, WorthUiHostContract, WorthUiHostMechanicsAdapter,
    WorthUiMeasurementHostAdapter,
};

use super::headless_measurement::UiHeadlessMeasurementEnvironment;
use super::{UiHeadlessMountedFrameTranscript, UiHeadlessRecorderCapacity};
mod capability;
mod input_focus;
mod presentation;
mod recorded_frame;
mod retained_command_store;
mod retained_order;
mod retained_presentation;
mod sample_observation;

use presentation::{add_order_cost, apply_work, work_cost};
use recorded_frame::{materialize_frames, UiHeadlessRecordedFrame};
use retained_command_store::UiHeadlessRetainedCommandStore;
use retained_order::UiHeadlessRetainedOrder;
pub(crate) use retained_presentation::UiHeadlessRetainedPresentation;
pub use sample_observation::UiHeadlessPresentationSampleObservation;

#[derive(Clone)]
pub struct WorthUiHeadlessRecorder {
    pub(super) state: Rc<RefCell<WorthUiHeadlessRecorderState>>,
}

pub(super) struct WorthUiHeadlessRecorderState {
    capacity: UiHeadlessRecorderCapacity,
    measurement: UiHeadlessMeasurementEnvironment,
    registrations:
        BTreeMap<worth_ui_host_contract::UiHostSurfaceIdentity, UiHostSurfaceRegistrationRequest>,
    transcripts: VecDeque<UiHeadlessRecordedFrame>,
    transcript_checkpoints: BTreeMap<
        worth_ui_host_contract::UiSurfaceBindingGeneration,
        UiHeadlessMountedFrameTranscript,
    >,
    pub(super) retained_presentations: BTreeMap<
        worth_ui_host_contract::UiSurfaceBindingGeneration,
        UiHeadlessRetainedPresentation,
    >,
    input_recipients: BTreeMap<u64, worth_ui_host_contract::UiHostInputRecipientBindingReceipt>,
    pub(super) semantic_focus:
        BTreeMap<u64, worth_ui_host_contract::UiHostFocusPlacementAcknowledgement>,
    latest_production_cost: Option<UiMountedPresentationProductionCost>,
}

impl WorthUiHeadlessRecorder {
    pub fn new(capacity: UiHeadlessRecorderCapacity) -> Self {
        Self::with_measurement_environment(
            capacity,
            UiHeadlessMeasurementEnvironment::unsupported(),
        )
    }

    pub fn with_viewport_extent(
        capacity: UiHeadlessRecorderCapacity,
        viewport: UiViewportExtentObservation,
    ) -> Self {
        Self::with_measurement_environment(
            capacity,
            UiHeadlessMeasurementEnvironment::fixed_viewport(viewport),
        )
    }

    pub fn with_viewport_extent_and_dpi(
        capacity: UiHeadlessRecorderCapacity,
        viewport: UiViewportExtentObservation,
        dpi_scale: worth_ui_host_contract::UiDpiScaleFactorObservation,
    ) -> Self {
        Self::with_measurement_environment(
            capacity,
            UiHeadlessMeasurementEnvironment::fixed_viewport_and_dpi(viewport, dpi_scale),
        )
    }

    fn with_measurement_environment(
        capacity: UiHeadlessRecorderCapacity,
        measurement: UiHeadlessMeasurementEnvironment,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(WorthUiHeadlessRecorderState {
                capacity,
                measurement,
                registrations: BTreeMap::new(),
                transcripts: VecDeque::new(),
                transcript_checkpoints: BTreeMap::new(),
                retained_presentations: BTreeMap::new(),
                input_recipients: BTreeMap::new(),
                semantic_focus: BTreeMap::new(),
                latest_production_cost: None,
            })),
        }
    }

    pub fn latest_production_cost(&self) -> Option<UiMountedPresentationProductionCost> {
        self.state.borrow().latest_production_cost
    }

    pub fn semantic_focus_placement(
        &self,
        host_session: u64,
    ) -> Option<worth_ui_host_contract::UiHostFocusPlacementAcknowledgement> {
        self.state
            .borrow()
            .semantic_focus
            .get(&host_session)
            .copied()
    }

    pub fn observed_transcripts(&self) -> Box<[UiHeadlessMountedFrameTranscript]> {
        let state = self.state.borrow();
        let mut checkpoints = state.transcript_checkpoints.clone();
        materialize_frames(state.transcripts.iter().cloned(), &mut checkpoints)
            .expect("admitted headless records must reconstruct exactly")
    }

    pub fn drain_transcripts(&self) -> Box<[UiHeadlessMountedFrameTranscript]> {
        let mut state = self.state.borrow_mut();
        let records = state.transcripts.drain(..).collect::<Vec<_>>();
        materialize_frames(records, &mut state.transcript_checkpoints)
            .expect("admitted headless records must reconstruct exactly")
    }

    fn validate_registration(
        &self,
        request: UiHostSurfaceRegistrationRequest,
    ) -> Result<(), UiHostSurfaceRegistrationDenial> {
        let agreement = match self.mechanical_protocol_contract().negotiate() {
            UiHostProtocolNegotiation::Compatible(agreement) => agreement,
            UiHostProtocolNegotiation::Incompatible(_) => {
                return Err(UiHostSurfaceRegistrationDenial::ForeignRegistration);
            }
        };
        let capabilities = self.mechanical_capability_report();
        if request.protocol() != agreement
            || request.capability_generation().as_u64() == 0
            || request.capability_profile_digest() != capabilities.profile_identity_digest()
        {
            return Err(UiHostSurfaceRegistrationDenial::ForeignRegistration);
        }
        let state = self.state.borrow();
        if state.registrations.len() >= state.capacity.surface_bindings() {
            return Err(UiHostSurfaceRegistrationDenial::CapacityExceeded);
        }
        if state
            .registrations
            .contains_key(&request.host_surface_identity())
        {
            return Err(UiHostSurfaceRegistrationDenial::ForeignRegistration);
        }
        Ok(())
    }

    fn validate_presented_view(
        &self,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> Result<UiHeadlessRecorderCapacity, UiHostSurfacePresentationDenial> {
        if view.requirement().presentation_mode() != UiHostSurfacePresentationMode::RecordOnly {
            return Err(
                UiHostSurfacePresentationDenial::UnsupportedPresentationMode(
                    view.requirement().presentation_mode(),
                ),
            );
        }
        let live_protocol = match self.mechanical_protocol_contract().negotiate() {
            UiHostProtocolNegotiation::Compatible(agreement) => agreement,
            UiHostProtocolNegotiation::Incompatible(denial) => {
                return Err(UiHostSurfacePresentationDenial::Protocol(denial));
            }
        };
        let live_profile = self
            .mechanical_capability_report()
            .profile_identity_digest();
        let state = self.state.borrow();
        let records_transcript = view.appearance_work().is_some()
            || matches!(
                view.presentation_work(),
                worth_ui_host_contract::UiMountedPresentationWorkView::Initial(_)
                    | worth_ui_host_contract::UiMountedPresentationWorkView::Delta(_)
                    | worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(_)
            );
        if records_transcript && state.transcripts.len() >= state.capacity.retained_frames() {
            return Err(UiHostSurfacePresentationDenial::CapacityExceeded);
        }
        let requirement = view.requirement();
        let registration = state
            .registrations
            .get(&requirement.host_surface())
            .ok_or(UiHostSurfacePresentationDenial::SurfaceBindingChanged)?;
        if view.host_session_identity() != registration.host_session_identity() {
            return Err(UiHostSurfacePresentationDenial::SurfaceBindingChanged);
        }
        if view.protocol() != registration.protocol() || live_protocol != registration.protocol() {
            return Err(UiHostSurfacePresentationDenial::ProtocolChanged);
        }
        if view.capability_generation() != registration.capability_generation()
            || requirement.capability_generation() != registration.capability_generation()
        {
            return Err(UiHostSurfacePresentationDenial::CapabilityGenerationChanged);
        }
        if view.capability_profile_digest() != registration.capability_profile_digest()
            || requirement.capability_profile_digest() != registration.capability_profile_digest()
            || live_profile != registration.capability_profile_digest()
        {
            return Err(UiHostSurfacePresentationDenial::CapabilityProfileChanged);
        }
        if requirement.semantic_surface() != registration.semantic_surface_identity()
            || requirement.binding() != registration.binding_generation()
            || requirement.presentation_mode() != registration.presentation_mode()
            || view.surface() != registration.semantic_surface_identity()
            || view.binding() != registration.binding_generation()
        {
            return Err(UiHostSurfacePresentationDenial::SurfaceBindingChanged);
        }
        Ok(state.capacity)
    }
}

impl Default for WorthUiHeadlessRecorder {
    fn default() -> Self {
        Self::new(UiHeadlessRecorderCapacity::production_default())
    }
}

impl WorthUiMeasurementHostAdapter for WorthUiHeadlessRecorder {
    fn observe_measurement(
        &self,
        request: &UiHostMeasurementRequest,
    ) -> UiHostMeasurementObservationValue {
        self.state.borrow().measurement.observe(request)
    }
}

impl WorthUiHostMechanicsAdapter for WorthUiHeadlessRecorder {
    fn mechanical_host_contract(&self) -> WorthUiHostContract {
        WorthUiHostContract::headless()
    }

    fn mechanical_capability_report(&self) -> WorthUiHostCapabilityReport {
        capability::report(self)
    }

    fn mechanical_measurement_environment_report(
        &self,
    ) -> worth_ui_host_contract::UiHostMeasurementEnvironmentReport {
        self.state.borrow().measurement.report()
    }

    fn install_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        input_focus::install_recipient(self, binding)
    }

    fn clear_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        input_focus::clear_recipient(self, binding)
    }

    fn perform_focus_placement(
        &self,
        request: worth_ui_host_contract::UiHostFocusPlacementRequest,
    ) -> worth_ui_host_contract::UiHostFocusPlacementAcknowledgement {
        super::solicited_effect::focus_placement::place(self, request)
    }

    fn perform_surface_registration(
        &self,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceRegistrationOutcome {
        if let Err(denial) = self.validate_registration(request) {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                denial,
            );
        }
        self.state
            .borrow_mut()
            .registrations
            .insert(request.host_surface_identity(), request);
        worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RegisteredKnownEmpty
    }

    fn perform_surface_deregistration(
        &self,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome {
        let mut state = self.state.borrow_mut();
        if state.registrations.get(&request.host_surface_identity()) != Some(&request) {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        state.registrations.remove(&request.host_surface_identity());
        state
            .retained_presentations
            .remove(&request.binding_generation());
        state
            .transcript_checkpoints
            .remove(&request.binding_generation());
        worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::Deregistered(
            worth_ui_host_contract::UiHostSurfaceDeregistrationReceipt::from_runtime(
                request.host_session_identity(),
                request.host_surface_identity(),
            ),
        )
    }

    fn perform_mounted_surface_presentation(
        &self,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> UiHostSurfacePresentationOutcome {
        let capacity = match self.validate_presented_view(view) {
            Ok(capacity) => capacity,
            Err(denial) => {
                return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial);
            }
        };
        let binding = view.binding();
        let adapter_cost = match work_cost(view) {
            Ok(cost) => cost,
            Err(denial) => {
                return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial);
            }
        };
        let mut state = self.state.borrow_mut();
        let mut retained = state.retained_presentations.remove(&binding);
        let (recorded, order_cost) = match apply_work(view, capacity, &mut retained) {
            Ok(recorded) => recorded,
            Err(denial) => {
                if let Some(retained) = retained {
                    state.retained_presentations.insert(binding, retained);
                }
                return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(denial);
            }
        };
        let adapter_cost = add_order_cost(adapter_cost, order_cost);
        state.latest_production_cost =
            Some(presentation::production_cost(view.presentation_work()));
        if let Some(recorded) = recorded {
            state.transcripts.push_back(recorded);
        }
        let epoch = worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(
            view.attempt().diagnostic_value(),
        );
        let mut retained = retained.expect("admitted work must retain current presentation state");
        retained.epoch = Some(epoch);
        state.retained_presentations.insert(binding, retained);
        UiHostSurfacePresentationOutcome::Presented(UiMountedSurfacePresentationCompletion::new(
            UiHostSurfacePresentationMode::RecordOnly,
            epoch,
            UiMountedCompletedEffects::new(vec![UiMountedEffectFamily::RecordedProjection]),
            adapter_cost,
        ))
    }

    fn release_mechanical_host_session(
        &self,
        host_session_identity: u64,
    ) -> UiHostSessionReleaseOutcome {
        let mut state = self.state.borrow_mut();
        state.semantic_focus.remove(&host_session_identity);
        let released_bindings = state
            .registrations
            .values()
            .filter(|request| request.host_session_identity() == host_session_identity)
            .map(|request| request.binding_generation())
            .collect::<Vec<_>>();
        let retained_before = state.registrations.len();
        state
            .registrations
            .retain(|_, request| request.host_session_identity() != host_session_identity);
        state.input_recipients.remove(&host_session_identity);
        for binding in released_bindings {
            state.retained_presentations.remove(&binding);
            state.transcript_checkpoints.remove(&binding);
        }
        UiHostSessionReleaseOutcome::Released(UiHostSessionReleaseReceipt::released(
            host_session_identity,
            retained_before - state.registrations.len(),
        ))
    }
}
