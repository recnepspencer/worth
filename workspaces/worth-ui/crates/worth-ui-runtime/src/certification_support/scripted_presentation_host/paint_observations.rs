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
