//! Real native input translation and retention with scripted rendering only.
use super::*;
use worth_ui_host_contract::*;
use worth_ui_host_native::UiNativeLifecycleProtocol;

pub(super) struct ScriptedPresentationNativeInput {
    pub(super) protocol: UiNativeLifecycleProtocol,
    pub(super) pending: BTreeMap<u64, (UiMountedFrameIdentity, UiSurfaceBindingGeneration)>,
}

impl ScriptedPresentationHost {
    pub fn begin_native_input_recovery(
        &self,
    ) -> Option<worth_ui_host_native::UiNativeInputRecoveryGrant> {
        self.native_input
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .protocol
            .begin_input_retention_recovery()
    }

    pub fn complete_native_input_recovery(
        &self,
        acknowledgement: worth_ui_host_native::UiNativeInputRecoveryAcknowledgement,
    ) -> bool {
        self.native_input
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .protocol
            .complete_input_retention_recovery(acknowledgement)
    }
    pub fn native_display_with_native_input() -> Self {
        let mut host = Self::native_display();
        let mut protocol = UiNativeLifecycleProtocol::new();
        protocol.install_initial_profile(1.0, [800, 600]);
        host.native_input = Some(Arc::new(Mutex::new(ScriptedPresentationNativeInput {
            protocol,
            pending: BTreeMap::new(),
        })));
        host
    }

    pub fn observe_native_window_event(
        &self,
        event: &winit::event::WindowEvent,
        tick: u64,
        pointer: Option<winit::dpi::PhysicalPosition<f64>>,
    ) -> worth_ui_host_native::UiNativeLifecycleTransition {
        self.native_input
            .as_ref()
            .expect("native input fixture installed")
            .lock()
            .unwrap()
            .protocol
            .observe_window_event_at(event, tick, pointer)
    }

    pub fn native_input_report(&self) -> worth_ui_host_native::UiNativeInputObservationReport {
        self.native_input
            .as_ref()
            .expect("native input fixture installed")
            .lock()
            .unwrap()
            .protocol
            .report()
    }

    pub(super) fn record_native_input_presentation(
        &self,
        view: &UiMountedFrameConsumptionView<'_>,
        outcome: &UiHostSurfacePresentationOutcome,
    ) {
        let Some(input) = &self.native_input else {
            return;
        };
        let mut input = input.lock().unwrap();
        match outcome {
            UiHostSurfacePresentationOutcome::Presented(completion) => {
                input.protocol.record_completed_presentation(
                    view.protocol(),
                    view.host_session_identity(),
                    UiHostObservationPresentationBasis::new(
                        view.requirement().host_surface(),
                        view.frame(),
                        view.binding(),
                        completion.epoch(),
                    ),
                );
            }
            UiHostSurfacePresentationOutcome::InFlight(token) => {
                let identity = token.diagnostic_value();
                input.protocol.remember_pending_presentation(
                    view.protocol(),
                    view.host_session_identity(),
                    view.requirement().host_surface(),
                    view.binding(),
                    identity,
                );
                assert!(input
                    .pending
                    .insert(identity, (view.frame(), view.binding()))
                    .is_none());
            }
            _ => {}
        }
    }

    pub(super) fn settle_native_input_presentation(
        &self,
        identity: u64,
        completion: Option<&UiMountedSurfacePresentationCompletion>,
    ) {
        let Some(input) = &self.native_input else {
            return;
        };
        let mut input = input.lock().unwrap();
        let (frame, binding) = input
            .pending
            .remove(&identity)
            .expect("pending native input context");
        if let Some(completion) = completion {
            input.protocol.complete_pending_presentation(
                frame,
                binding,
                completion.epoch(),
                identity,
            );
        } else {
            input
                .protocol
                .abandon_pending_presentation(binding, Some(identity));
        }
    }
}
