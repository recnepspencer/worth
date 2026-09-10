mod appearance_mount;
mod identity;
mod inspection;
mod interaction;
mod layout_reconstruction;
mod motion_sampling;
mod occurrence_projection;
mod projection;
mod publication;
mod raster_cache_reconstruction;
mod replacement;
mod selection_binding;
#[cfg(test)]
mod test_support;
pub use selection_binding::UiMountedSelectionBindingDenial;

pub(crate) use motion_sampling::UiMountedMotionSampleSettlement;
pub(crate) use publication::{
    UiMountedHostObservationTransition, UiMountedObservationValidationBasis,
    UiMountedPublicationTransition,
};
pub(crate) use replacement::{
    UiMountedGraphReplacementAdmission, UiMountedGraphReplacementInFlight,
    UiMountedGraphReplacementPreparation, UiMountedGraphReplacementPresentation,
    UiMountedGraphReplacementSuccessor,
};

use std::collections::BTreeMap;

use worth_ui_host_contract::UiMountedPresentationAttemptIdentity;

/// Mounted lifecycle authority retained by one active application session.
pub(crate) struct WorthUiMountedSessionState {
    identity: super::UiMountedIdentityState,
    occurrence_geometry: super::UiMountedOccurrenceGeometryState,
    retention: super::UiMountedFrameRetentionCoordinator,
    presentation: super::UiMountedPresentationCoordinator,
    motion_sampling: super::presentation::motion_sampling::UiMountedMotionSampler,
    selection_bindings: super::selection_binding::UiMountedSelectionBindings,
    publication_reservations:
        BTreeMap<UiMountedPresentationAttemptIdentity, super::UiMountedFramePublicationCandidate>,
    reconciliation_reservations: BTreeMap<
        UiMountedPresentationAttemptIdentity,
        super::UiMountedFrameReconciliationCandidate,
    >,
}

impl WorthUiMountedSessionState {
    pub(crate) fn new(
        host_session: crate::facade::WorthUiHostSessionIdentity,
        retention_budget: super::UiMountedFrameRetentionBudget,
        presentation_async: Option<
            crate::native_platform::text_presentation::UiPresentationAsyncRuntime,
        >,
    ) -> Result<Self, super::UiMountedIdentityDenial> {
        Ok(Self {
            identity: super::UiMountedIdentityState::new(host_session)?,
            occurrence_geometry: Default::default(),
            retention: super::UiMountedFrameRetentionCoordinator::with_budget(retention_budget),
            presentation: super::UiMountedPresentationCoordinator::new(presentation_async),
            motion_sampling: Default::default(),
            selection_bindings: Default::default(),
            publication_reservations: BTreeMap::new(),
            reconciliation_reservations: BTreeMap::new(),
        })
    }

    pub(crate) fn has_active_presentation_attempt(&self) -> bool {
        self.presentation.has_active_attempt()
    }

    pub(crate) fn observation_basis_admission_ready(&self) -> bool {
        self.retention.observation_basis_admission_ready()
    }

    pub(crate) fn place_semantic_focus(
        &mut self,
        basis: super::UiMountedFocusPlacementRequestBasis,
        supported: bool,
        host: crate::facade::UiHostEffectPort<'_>,
    ) -> Result<
        worth_ui_host_contract::UiHostFocusPlacementAcknowledgement,
        super::UiMountedFocusPlacementDenial,
    > {
        self.presentation
            .place_semantic_focus(basis, supported, host)
    }

    pub(crate) fn reconcile_focus_placement(
        &mut self,
        observation: worth_ui_host_contract::UiHostFocusPlacementObservation,
    ) -> Result<
        super::UiFocusHostPlacementReconciliationReceipt,
        super::UiFocusHostPlacementReconciliationDenial,
    > {
        self.presentation.reconcile_focus_placement(observation)
    }

    pub(crate) fn shutdown_focus_placement(&mut self) -> super::UiFocusHostPlacementShutdownReport {
        self.presentation.shutdown_focus_placement()
    }

    pub(crate) fn shutdown_presentation(
        &mut self,
        host: &crate::facade::WorthUiHostSessionAuthority,
    ) -> (
        super::UiMountedPresentationShutdownReport,
        Vec<super::UiMountedPresentationOutcome>,
        Option<crate::native_platform::text_presentation::UiPresentationAsyncTerminalCleanup>,
    ) {
        let _ = self.presentation.cancel_motion_sample(host.effect_port());
        self.selection_bindings.clear();
        self.presentation.shutdown(host.effect_port())
    }

    pub(crate) fn assert_shutdown_resolved(&self) {
        assert!(
            self.publication_reservations.is_empty(),
            "shutdown resolves every retained mounted publication reservation"
        );
        assert!(
            self.reconciliation_reservations.is_empty(),
            "shutdown resolves every retained mounted reconciliation reservation"
        );
    }

    pub(crate) const fn native_client_resource_peaks(&self) -> [usize; 2] {
        [
            self.identity.peak_qualified_layouts(),
            self.presentation.peak_raster_cache_entries(),
        ]
    }
}
