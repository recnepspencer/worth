impl super::UiMountedPresentationState {
    pub(crate) fn rebind_surface(
        &mut self,
        successor: crate::mounting::UiSurfaceBindingIdentityView,
    ) {
        self.rebound_from_binding
            .get_or_insert(self.requirement.binding());
        self.requirement =
            worth_ui_host_contract::UiMountedSurfaceBindingRequirement::with_baseline(
                successor.semantic_surface_identity(),
                successor.host_surface_identity(),
                successor.binding_generation(),
                successor.capability_observation_generation(),
                successor.capability_profile_digest(),
                successor.presentation_mode(),
                successor.baseline(),
            );
    }
}
