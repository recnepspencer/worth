use super::ScriptedPresentationHost;

impl ScriptedPresentationHost {
    pub fn last_node_changes(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedPresentationNodeChange> {
        self.state.lock().unwrap().last_node_changes.clone()
    }

    pub fn last_surface_colors(&self) -> Vec<worth_ui_host_contract::UiMountedRgba8> {
        self.state.lock().unwrap().last_surface_colors.clone()
    }

    /// The scroll bars the last presentation carrying appearance painted,
    /// every one its frame holds rather than only those that changed. A
    /// Motion sample moves them without painting them again.
    pub fn last_scroll_chrome(&self) -> Vec<worth_ui_host_contract::UiMountedScrollChromeMechanic> {
        self.state.lock().unwrap().last_scroll_chrome.clone()
    }

    /// The command samples the last presented Motion sample carried.
    pub fn last_motion_samples(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        self.state.lock().unwrap().last_motion_samples.clone()
    }

    pub fn last_appearance_samples(
        &self,
    ) -> Vec<worth_ui_host_contract::UiMountedPresentationSampleChange> {
        self.state.lock().unwrap().last_appearance_samples.clone()
    }

    pub fn reconstruction_sample_overrides(
        &self,
    ) -> Vec<Box<[worth_ui_host_contract::UiMountedPresentationSampleChange]>> {
        self.state
            .lock()
            .unwrap()
            .reconstruction_sample_overrides
            .clone()
    }
}
