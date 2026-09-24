use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;

use crate::facade::host::{
    UiHostAdapterSessionAuthority, UiHostSessionReleaseOutcome, UiHostSessionReleaseReceipt,
    WorthUiOperationalHostAdapter,
};
use worth_ui_host_contract::{
    UiHostMeasurementEnvironmentReport, UiHostMeasurementObservationValue,
    UiHostMeasurementRequest, UiHostPresentationCostInput, UiHostPresentationCostReport,
    UiHostPresentationEpoch, UiHostSurfacePresentationDenial, UiHostSurfacePresentationMode,
    UiHostSurfacePresentationOutcome, UiHostSurfaceRegistrationDenial,
    UiHostSurfaceRegistrationRequest, UiMountedCompletedEffects, UiMountedEffectFamily,
    UiMountedFrameConsumptionView, UiViewportExtentObservation, WorthUiHostCapability,
    WorthUiHostCapabilityReport, WorthUiHostContract, WorthUiMeasurementHostAdapter,
};

type NativeSurfaceRegistrations =
    BTreeMap<worth_ui_host_contract::UiHostSurfaceIdentity, UiHostSurfaceRegistrationRequest>;

#[derive(Clone, Default)]
pub(super) struct NativeIdentityTraceHost {
    registrations: Rc<RefCell<NativeSurfaceRegistrations>>,
    presentation_calls: Rc<Cell<usize>>,
}

impl WorthUiMeasurementHostAdapter for NativeIdentityTraceHost {
    fn observe_measurement(
        &self,
        request: &UiHostMeasurementRequest,
    ) -> UiHostMeasurementObservationValue {
        match request.family() {
            worth_ui_host_contract::UiMeasurementRequestFamily::ViewportExtent => {
                UiHostMeasurementObservationValue::ViewportExtent(UiViewportExtentObservation {
                    width: 1_000.0,
                    height: 700.0,
                })
            }
            family => panic!("identity-trace host received unsupported measurement: {family:?}"),
        }
    }
}

impl WorthUiOperationalHostAdapter for NativeIdentityTraceHost {
    fn operational_host_contract(&self) -> WorthUiHostContract {
        WorthUiHostContract::native()
    }

    fn operational_capability_report(&self) -> WorthUiHostCapabilityReport {
        WorthUiHostCapabilityReport::available(vec![
            WorthUiHostCapability::ViewportObservation,
            WorthUiHostCapability::NativePaint,
        ])
    }

    fn measurement_environment_report(&self) -> UiHostMeasurementEnvironmentReport {
        UiHostMeasurementEnvironmentReport::new(Some(1), None, None, None)
    }

    fn register_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceRegistrationOutcome {
        if request.host_session_identity() != authority.host_session_identity() {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        let mut registrations = self.registrations.borrow_mut();
        if registrations
            .insert(request.host_surface_identity(), request)
            .is_some()
        {
            return worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        worth_ui_host_contract::UiHostSurfaceRegistrationOutcome::RegisteredKnownEmpty
    }

    fn deregister_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        request: UiHostSurfaceRegistrationRequest,
    ) -> worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome {
        if request.host_session_identity() != authority.host_session_identity()
            || self
                .registrations
                .borrow_mut()
                .remove(&request.host_surface_identity())
                != Some(request)
        {
            return worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::RejectedBeforeEffects(
                UiHostSurfaceRegistrationDenial::ForeignRegistration,
            );
        }
        worth_ui_host_contract::UiHostSurfaceDeregistrationOutcome::Deregistered(
            worth_ui_host_contract::UiHostSurfaceDeregistrationReceipt::from_runtime(
                request.host_session_identity(),
                request.host_surface_identity(),
            ),
        )
    }

    fn present_mounted_surface(
        &self,
        authority: &UiHostAdapterSessionAuthority,
        view: &UiMountedFrameConsumptionView<'_>,
    ) -> UiHostSurfacePresentationOutcome {
        if !authority.admits_mounted_presentation(view)
            || view.requirement().presentation_mode()
                != UiHostSurfacePresentationMode::NativeDisplay
            || !self.registration_matches(view)
        {
            return UiHostSurfacePresentationOutcome::RejectedBeforeEffects(
                UiHostSurfacePresentationDenial::SurfaceBindingChanged,
            );
        }
        self.presentation_calls
            .set(self.presentation_calls.get() + 1);
        UiHostSurfacePresentationOutcome::Presented(view.acknowledge_presented(
            UiHostSurfacePresentationMode::NativeDisplay,
            UiHostPresentationEpoch::issued_by_host(view.attempt().diagnostic_value()),
            UiMountedCompletedEffects::new(performed_effects(view)),
            UiHostPresentationCostReport::from_adapter(UiHostPresentationCostInput {
                presented_surfaces: 1,
                translated_rows: 0,
                translated_bytes: 0,
                native_resource_cache_hits: 0,
                native_resource_cache_misses: 0,
                asynchronous_handoffs: 0,
                ..Default::default()
            }),
        ))
    }

    fn release_host_session(
        &self,
        authority: &UiHostAdapterSessionAuthority,
    ) -> UiHostSessionReleaseOutcome {
        let mut registrations = self.registrations.borrow_mut();
        let before = registrations.len();
        registrations.retain(|_, request| {
            request.host_session_identity() != authority.host_session_identity()
        });
        UiHostSessionReleaseOutcome::Released(UiHostSessionReleaseReceipt::released(
            authority.host_session_identity(),
            before - registrations.len(),
        ))
    }
}

fn performed_effects(view: &UiMountedFrameConsumptionView<'_>) -> Vec<UiMountedEffectFamily> {
    match view.presentation_work() {
        worth_ui_host_contract::UiMountedPresentationWorkView::Initial(initial) => {
            initial_effects(initial.projection())
        }
        worth_ui_host_contract::UiMountedPresentationWorkView::Delta(delta) => {
            (!delta.changes().is_empty() || !delta.order().is_empty() || !delta.damage().is_empty())
                .then_some(UiMountedEffectFamily::NativePaint)
                .into_iter()
                .collect()
        }
        worth_ui_host_contract::UiMountedPresentationWorkView::Reconstruction(work) => {
            initial_effects(work.projection())
        }
        worth_ui_host_contract::UiMountedPresentationWorkView::Sample(sample) => {
            (!sample.changes().is_empty() || !sample.damage().is_empty())
                .then_some(UiMountedEffectFamily::NativePaint)
                .into_iter()
                .collect()
        }
        worth_ui_host_contract::UiMountedPresentationWorkView::Unchanged(_) => Vec::new(),
    }
}

fn initial_effects(
    projection: &worth_ui_host_contract::UiMountedProjectionView,
) -> Vec<UiMountedEffectFamily> {
    let mut effects = projection.authored_native_effects().to_vec();
    effects.push(UiMountedEffectFamily::NativePaint);
    effects.sort_unstable();
    effects.dedup();
    effects
}

impl NativeIdentityTraceHost {
    pub(super) fn presentation_calls(&self) -> usize {
        self.presentation_calls.get()
    }

    fn registration_matches(&self, view: &UiMountedFrameConsumptionView<'_>) -> bool {
        let requirement = view.requirement();
        self.registrations
            .borrow()
            .get(&requirement.host_surface())
            .is_some_and(|registration| {
                registration.host_session_identity() == view.host_session_identity()
                    && registration.semantic_surface_identity() == requirement.semantic_surface()
                    && registration.binding_generation() == requirement.binding()
                    && registration.presentation_mode() == requirement.presentation_mode()
            })
    }
}
