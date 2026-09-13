use super::{UiNativePresentationWorkKind, UiNativeRetainedFrameObservation};
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiMountedPresentationAttemptIdentity,
};

impl UiNativeRetainedFrameObservation {
    /// Compares reporting data with the current basis selected by the runtime.
    /// This observation does not establish acceptance or retain runtime authority.
    pub fn matches_runtime_attribution_basis(
        &self,
        publication_attempt: UiMountedPresentationAttemptIdentity,
        current: UiHostObservationPresentationBasis,
    ) -> bool {
        let Some(observed) = self.presentation() else {
            return false;
        };
        if self.frame() != observed.presented_frame()
            || self.semantic_surface() != observed.semantic_surface()
            || self.host_surface() != observed.host_surface()
            || self.binding_generation() != observed.binding_generation()
            || self.presentation_attempt() != observed.presentation_attempt()
            || self.frame() != current.frame().diagnostic_value()
            || self.host_surface() != current.host_surface().diagnostic_value()
            || self.binding_generation() != current.binding().diagnostic_value()
        {
            return false;
        }
        // A sample changes physical epoch without republishing mounted identity.
        // Unchanged publication can instead retain its predecessor's epoch.
        if self.kind() == UiNativePresentationWorkKind::Sample {
            self.sample_presentation_epoch() == Some(current.epoch())
        } else {
            self.presentation_attempt() == publication_attempt.diagnostic_value()
        }
    }
}

#[cfg(test)]
#[path = "attribution_basis_tests.rs"]
mod tests;
