use super::{UiNativeAppearanceCommand, UiNativeAppearanceCommandKey, UiNativeAppearanceRetained};

impl UiNativeAppearanceRetained {
    pub(super) fn remove_text_key(
        &mut self,
        key: UiNativeAppearanceCommandKey,
        identities: &[worth_ui_host_contract::UiMountedPaintCommandIdentity],
    ) {
        for identity in identities {
            if let Some(keys) = self.text_keys_by_paint_command.get_mut(identity) {
                keys.remove(&key);
                if keys.is_empty() {
                    self.text_keys_by_paint_command.remove(identity);
                }
            }
        }
    }
}

pub(crate) fn text_paint_commands(
    command: &UiNativeAppearanceCommand,
) -> Vec<worth_ui_host_contract::UiMountedPaintCommandIdentity> {
    match command {
        UiNativeAppearanceCommand::TextForeground(foreground) => {
            foreground.candidate_commands().collect()
        }
        _ => Vec::new(),
    }
}
