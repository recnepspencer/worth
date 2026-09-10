use std::cell::RefCell;
use std::rc::Rc;

use crate::native::{UiNativeHostState, WorthUiNativeEventLoop, WorthUiNativeMechanicsAdapter};

/// The exact native mechanics half consumed by the runtime binding.
pub struct WorthUiPreparedNativeMechanics {
    adapter: WorthUiNativeMechanicsAdapter,
}

/// Effect-free qualified native mechanics preparation.
///
/// Preparing this value allocates no event loop, window, surface, adapter,
/// device, queue, or presentation target. `into_parts` consumes it into the
/// one adapter/driver pair that shares the later live native state.
pub struct WorthUiPreparedNativeHost {
    state: Rc<RefCell<UiNativeHostState>>,
    profile: super::UiNativePlatformProfileIdentity,
    event_loop_thread_posture: crate::native::UiNativeEventLoopThreadPosture,
}

impl WorthUiPreparedNativeHost {
    pub fn prepare_qualified() -> Self {
        Self {
            state: Rc::new(RefCell::new(UiNativeHostState::new())),
            profile: super::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2,
            event_loop_thread_posture:
                crate::native::UiNativeEventLoopThreadPosture::MainThreadRequired,
        }
    }

    /// Prepare the same qualified host with a bounded native-owner courtroom
    /// control. Available only to certification compositions.
    #[cfg(feature = "certification-support")]
    pub fn prepare_qualified_for_certification(plan: crate::UiNativeQualificationPlan) -> Self {
        let event_loop_thread_posture = plan.event_loop_thread_posture();
        Self {
            state: Rc::new(RefCell::new(UiNativeHostState::new_for_certification(plan))),
            profile: super::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2,
            event_loop_thread_posture,
        }
    }

    /// Consume the effect-free preparation into its exact mechanics adapter
    /// and event-loop owner. Neither part is independently constructible.
    pub fn into_parts(
        self,
        window: UiNativeWindowConfiguration,
    ) -> (WorthUiPreparedNativeMechanics, WorthUiNativeEventLoop) {
        debug_assert_eq!(
            self.profile,
            super::UiNativePlatformProfileIdentity::WORTH_UI_WINDOWS_DX12_V2
        );
        let adapter =
            WorthUiNativeMechanicsAdapter::from_preparation(Rc::clone(&self.state), self.profile);
        (
            WorthUiPreparedNativeMechanics { adapter },
            WorthUiNativeEventLoop::from_preparation(
                self.state,
                window,
                self.event_loop_thread_posture,
            ),
        )
    }
}

impl worth_ui_host_contract::WorthUiMeasurementHostAdapter for WorthUiPreparedNativeMechanics {
    fn observe_measurement(
        &self,
        request: &worth_ui_host_contract::UiHostMeasurementRequest,
    ) -> worth_ui_host_contract::UiHostMeasurementObservationValue {
        worth_ui_host_contract::WorthUiMeasurementHostAdapter::observe_measurement(
            &self.adapter,
            request,
        )
    }
}

impl worth_ui_host_contract::WorthUiHostMechanicsAdapter for WorthUiPreparedNativeMechanics {
    fn mechanical_host_contract(&self) -> worth_ui_host_contract::WorthUiHostContract {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::mechanical_host_contract(&self.adapter)
    }

    fn mechanical_protocol_contract(&self) -> worth_ui_host_contract::UiHostProtocolContract {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::mechanical_protocol_contract(
            &self.adapter,
        )
    }

    fn mechanical_capability_report(&self) -> worth_ui_host_contract::WorthUiHostCapabilityReport {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::mechanical_capability_report(
            &self.adapter,
        )
    }

    fn register_mechanical_host_session(
        &self,
        host_session_identity: u64,
    ) -> Result<(), worth_ui_host_contract::UiHostObservationSessionRegistrationDenial> {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::register_mechanical_host_session(
            &self.adapter,
            host_session_identity,
        )
    }

    fn mechanical_measurement_environment_report(
        &self,
    ) -> worth_ui_host_contract::UiHostMeasurementEnvironmentReport {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::mechanical_measurement_environment_report(
            &self.adapter,
        )
    }

    fn mechanical_visual_capture_capability(
        &self,
    ) -> worth_ui_host_contract::UiHostCaptureCapability {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::mechanical_visual_capture_capability(
            &self.adapter,
        )
    }

    fn drain_mechanical_host_observations(
        &self,
        host_session_identity: u64,
    ) -> Result<
        worth_ui_host_contract::UiHostObservationDrain,
        worth_ui_host_contract::UiHostObservationDrainDenial,
    > {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::drain_mechanical_host_observations(
            &self.adapter,
            host_session_identity,
        )
    }

    fn install_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::install_mechanical_input_recipient(
            &self.adapter,
            binding,
        )
    }

    fn clear_mechanical_input_recipient(
        &self,
        binding: worth_ui_host_contract::UiHostInputRecipientBindingReceipt,
    ) -> bool {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::clear_mechanical_input_recipient(
            &self.adapter,
            binding,
        )
    }

    fn perform_visual_capture(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureObservationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_visual_capture(
            &self.adapter,
            request,
        )
    }

    fn perform_visual_capture_cancellation(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> worth_ui_host_contract::UiHostCaptureCancellationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_visual_capture_cancellation(
            &self.adapter,
            request,
        )
    }

    fn perform_focus_placement(
        &self,
        request: worth_ui_host_contract::UiHostFocusPlacementRequest,
    ) -> worth_ui_host_contract::UiHostFocusPlacementAcknowledgement {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_focus_placement(
            &self.adapter,
            request,
        )
    }

    fn perform_mounted_surface_presentation(
        &self,
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> worth_ui_host_contract::UiHostSurfacePresentationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_mounted_surface_presentation(
            &self.adapter,
            view,
        )
    }

    fn perform_mounted_surface_completion(
        &self,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
    ) -> worth_ui_host_contract::UiHostSurfaceInFlightCompletion {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_mounted_surface_completion(
            &self.adapter,
            token,
        )
    }

    fn perform_mounted_surface_cancellation(
        &self,
        token: worth_ui_host_contract::UiHostPresentationCompletionToken,
        reason: worth_ui_host_contract::UiHostSurfaceStopReason,
    ) -> worth_ui_host_contract::UiHostSurfaceCancellationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_mounted_surface_cancellation(
            &self.adapter,
            token,
            reason,
        )
    }

    fn perform_surface_registration(
        &self,
        request: worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceRegistrationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_surface_registration(
            &self.adapter,
            request,
        )
    }

    fn perform_surface_deregistration(
        &self,
        request: worth_ui_host_contract::UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::perform_surface_deregistration(
            &self.adapter,
            request,
        )
    }

    fn release_mechanical_host_session(
        &self,
        host_session_identity: u64,
    ) -> worth_ui_host_contract::UiHostSessionReleaseOutcome {
        worth_ui_host_contract::WorthUiHostMechanicsAdapter::release_mechanical_host_session(
            &self.adapter,
            host_session_identity,
        )
    }
}

#[cfg(test)]
#[path = "prepared_host_tests.rs"]
mod tests;

/// Vendor-free window configuration consumed by the native event-loop owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiNativeWindowConfiguration {
    title: Box<str>,
    initial_logical_size: [u32; 2],
}

impl UiNativeWindowConfiguration {
    pub fn qualified(title: impl Into<Box<str>>, initial_logical_size: [u32; 2]) -> Self {
        Self {
            title: title.into(),
            initial_logical_size,
        }
    }

    pub(crate) fn title(&self) -> &str {
        &self.title
    }

    pub(crate) const fn initial_logical_size(&self) -> [u32; 2] {
        self.initial_logical_size
    }
}
