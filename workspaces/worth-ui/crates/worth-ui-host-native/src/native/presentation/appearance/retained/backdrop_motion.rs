use super::{UiNativeAppearanceCommand, UiNativeAppearanceCommandKey, UiNativeAppearanceRetained};

impl UiNativeAppearanceRetained {
    pub(super) fn index_backdrop_motion(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        command: &UiNativeAppearanceCommand,
    ) {
        if let UiNativeAppearanceCommand::Backdrop(backdrop) = command {
            if let Some(target) = backdrop.motion_target() {
                self.backdrop_keys_by_portal
                    .entry((target.owner(), target.portal_identity()))
                    .or_default()
                    .insert(key);
            }
        }
    }

    pub(super) fn remove_backdrop_motion(&mut self, key: UiNativeAppearanceCommandKey) {
        let Some(UiNativeAppearanceCommand::Backdrop(backdrop)) = self.commands.get(&key) else {
            return;
        };
        let Some(target) = backdrop.motion_target() else {
            return;
        };
        let identity = (target.owner(), target.portal_identity());
        if let Some(keys) = self.backdrop_keys_by_portal.get_mut(&identity) {
            keys.remove(&key);
            if keys.is_empty() {
                self.backdrop_keys_by_portal.remove(&identity);
            }
        }
    }

    pub(in crate::native::presentation) fn backdrops_for_motion(
        &self,
        target: worth_ui_host_contract::UiMountedPortalPresentationAffinity,
    ) -> impl Iterator<Item = &worth_ui_host_contract::UiMountedBackdropMechanic> {
        self.backdrop_keys_by_portal
            .get(&(target.owner(), target.portal_identity()))
            .into_iter()
            .flat_map(|keys| keys.iter())
            .filter_map(|key| match self.commands.get(key) {
                Some(UiNativeAppearanceCommand::Backdrop(backdrop)) => Some(backdrop),
                _ => None,
            })
    }
}
