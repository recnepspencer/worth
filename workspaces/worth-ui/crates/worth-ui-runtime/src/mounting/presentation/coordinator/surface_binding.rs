use super::{UiMountedPresentationCoordinator, UiMountedTextPinCandidate};
use worth_ui_host_contract::UiSurfaceBindingGeneration;

impl UiMountedPresentationCoordinator {
    pub(crate) fn prepare_text_pin_deregistration(
        &self,
        binding: UiSurfaceBindingGeneration,
    ) -> UiMountedTextPinCandidate {
        self.text.deregistration_candidate(binding)
    }

    pub(crate) fn commit_surface_deregistration(
        &mut self,
        binding: UiSurfaceBindingGeneration,
        candidate: UiMountedTextPinCandidate,
        preserve_for_rebind: bool,
    ) {
        self.text.commit_surface_candidate(candidate);
        if !preserve_for_rebind {
            self.presentation_states.remove(&binding);
            self.reconstruction_bindings.remove(&binding);
        }
    }

    pub(crate) fn commit_surface_rebind(
        &mut self,
        prior: UiSurfaceBindingGeneration,
        successor: crate::mounting::UiSurfaceBindingIdentityView,
    ) {
        self.reconstruction_bindings.remove(&prior);
        if let Some(mut state) = self.presentation_states.remove(&prior) {
            state.rebind_surface(successor);
            self.presentation_states
                .insert(successor.binding_generation(), state);
            self.reconstruction_bindings
                .insert(successor.binding_generation());
        }
    }

    pub(crate) fn abandon_surface_rebind(&mut self, prior: UiSurfaceBindingGeneration) {
        self.presentation_states.remove(&prior);
        self.reconstruction_bindings.remove(&prior);
    }
}
