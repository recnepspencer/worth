struct UiHostAdapterSessionGrant;

impl worth_proof::AuthorityMarker for UiHostAdapterSessionGrant {}

/// Concrete runtime-owned capability required at every host-effect boundary.
///
/// Adapter implementations may inspect the admitted session identity, but only
/// the Worth UI runtime can construct this capability.
pub struct UiHostAdapterSessionAuthority {
    host_session_identity: u64,
    _authority: worth_proof::AuthorityWitness<UiHostAdapterSessionGrant>,
    presentation_lease_gate: crate::mounting::presentation::UiMountedPresentationLeaseGate,
    surface_effect_rounds: std::cell::Cell<u64>,
}

impl UiHostAdapterSessionAuthority {
    pub(crate) fn activate(host_session_identity: u64) -> Self {
        Self {
            host_session_identity,
            _authority: worth_proof::AuthorityWitness::from_authority_marker(
                UiHostAdapterSessionGrant,
            ),
            presentation_lease_gate: Default::default(),
            surface_effect_rounds: std::cell::Cell::new(0),
        }
    }

    /// Records the host's answer to one surface presentation. A presented or
    /// in-flight answer began effects, and the host drops any retry wake it
    /// held for an earlier frame it rejected before effects.
    pub(crate) fn record_surface_presentation(
        &self,
        outcome: &crate::facade::mounted::UiHostSurfacePresentationOutcome,
    ) {
        use crate::facade::mounted::UiHostSurfacePresentationOutcome as Outcome;
        if matches!(outcome, Outcome::Presented(_) | Outcome::InFlight(_)) {
            self.surface_effect_rounds
                .set(self.surface_effect_rounds.get().wrapping_add(1));
        }
    }

    /// How many surface presentations the host has begun effects for.
    pub(crate) fn surface_effect_rounds(&self) -> u64 {
        self.surface_effect_rounds.get()
    }

    pub fn host_session_identity(&self) -> u64 {
        self.host_session_identity
    }

    pub fn admits_mounted_presentation(
        &self,
        view: &worth_ui_host_contract::UiMountedFrameConsumptionView<'_>,
    ) -> bool {
        view.host_session_identity() == self.host_session_identity
            && self.presentation_lease_gate.admits(view)
    }

    pub fn admits_mounted_completion_token(
        &self,
        token: &worth_ui_host_contract::UiHostPresentationCompletionToken,
    ) -> bool {
        self.presentation_lease_gate.admits_token(token)
    }

    pub(crate) fn issued_mounted_completion(
        &self,
        completion: &worth_ui_host_contract::UiMountedSurfacePresentationCompletion,
    ) -> bool {
        self.presentation_lease_gate.admits_completion(completion)
    }

    pub fn admits_visual_capture(
        &self,
        request: worth_ui_host_contract::UiHostVisualCaptureRequest,
    ) -> bool {
        request.host_session_identity() == self.host_session_identity
    }

    pub(crate) fn claim_mounted_presentation_lease(
        &self,
    ) -> Result<
        crate::mounting::presentation::UiMountedPresentationLease,
        crate::mounting::presentation::UiMountedPresentationLeaseDenial,
    > {
        self.presentation_lease_gate.claim()
    }
}

impl std::fmt::Debug for UiHostAdapterSessionAuthority {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("UiHostAdapterSessionAuthority")
            .field("host_session_identity", &self.host_session_identity)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use worth_ui_host_contract::{
        UiHostCaptureArtifactBudget, UiHostCaptureFrameAffinity, UiHostCaptureRequestIdentity,
        UiHostCaptureSurfaceAffinity, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiHostVisualCaptureRequest, UiMountedFrameIdentity, UiMountedPresentationAttemptIdentity,
        UiSurfaceBindingGeneration,
    };

    use super::UiHostAdapterSessionAuthority;

    #[test]
    fn visual_capture_authority_rejects_foreign_host_sessions() {
        let authority = UiHostAdapterSessionAuthority::activate(7);
        assert!(authority.admits_visual_capture(capture_request(7)));
        assert!(!authority.admits_visual_capture(capture_request(8)));
    }

    fn capture_request(host_session_identity: u64) -> UiHostVisualCaptureRequest {
        UiHostVisualCaptureRequest::admitted_by_runtime(
            UiHostCaptureRequestIdentity::issued_by_runtime(1),
            UiHostCaptureFrameAffinity::observed_by_runtime(
                UiMountedFrameIdentity::mint_unbound().unwrap(),
                UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
            ),
            UiHostCaptureSurfaceAffinity::observed_by_runtime(
                host_session_identity,
                UiHostSurfaceIdentity::mint_unbound().unwrap(),
                UiSurfaceBindingGeneration::mint_unbound().unwrap(),
                UiHostPresentationEpoch::issued_by_host(1),
            ),
            UiHostCaptureArtifactBudget::admitted_by_runtime(false, 0),
        )
    }
}
