impl super::UiMountedIdentityState {
    pub(in crate::mounting) fn admits_retained_appearance_generation(
        &self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) -> bool {
        self.current_projection
            .as_ref()
            .is_none_or(|owner| owner.admits_retained_appearance_generation(owners))
    }

    pub(in crate::mounting) fn commit_retained_appearance_generation(
        &mut self,
        owners: &crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) {
        if let Some(owner) = self.current_projection.as_mut() {
            std::rc::Rc::make_mut(owner).commit_retained_appearance_generation(owners);
        }
    }
}
