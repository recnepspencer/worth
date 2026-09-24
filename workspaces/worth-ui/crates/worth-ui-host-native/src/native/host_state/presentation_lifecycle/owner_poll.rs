use super::{
    ready_token::ReadyPresentation, UiNativeHostState, UiNativePresentationPhysicalProgress,
};

pub(super) struct PolledPresentation {
    pub(super) ready: ReadyPresentation,
    pub(super) observation:
        crate::native::physical_work_signal::UiNativePhysicalSignalExternalObservation,
}

impl UiNativeHostState {
    pub(super) fn poll_presentation_owner(
        &mut self,
        _identity: crate::native::physical_work_signal::UiNativePhysicalPresentationIdentity,
        mut ready: ReadyPresentation,
    ) -> Result<PolledPresentation, UiNativePresentationPhysicalProgress> {
        let device = self
            .presentation_owners
            .as_ref()
            .map(|owners| owners.device.state().device());
        #[cfg(feature = "certification-support")]
        let qualified_override = self.qualification.presentation_poll_override(_identity);
        #[cfg(feature = "certification-support")]
        if let Some((_, Some(class))) = qualified_override {
            let binding = ready.pending.physical_basis().binding().diagnostic_value();
            if !self.can_apply_qualified_derived_state_loss(binding, class) {
                self.pending_presentations
                    .insert(ready.index, ready.pending);
                return Err(UiNativePresentationPhysicalProgress::NoProgress);
            }
        }
        let observation = ready.pending.poll_observation(device);
        Ok(PolledPresentation { ready, observation })
    }
}
