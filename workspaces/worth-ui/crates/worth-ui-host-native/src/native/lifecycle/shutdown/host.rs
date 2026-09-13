use super::orchestrator::UiNativeShutdownPort;

pub(super) struct UiNativeHostShutdownPort<'state> {
    state: &'state mut crate::native::UiNativeHostState,
}

impl<'state> UiNativeHostShutdownPort<'state> {
    pub(super) fn new(state: &'state mut crate::native::UiNativeHostState) -> Self {
        Self { state }
    }
}

impl UiNativeShutdownPort for UiNativeHostShutdownPort<'_> {
    type Census = crate::native::UiNativeResourceCensus;

    fn begin_close(&mut self) {
        self.state.last_retained_frame = None;
        self.state.lifecycle.close_input_protocol();
        let _ = self.state.physical_signal.shutdown();
    }

    fn settle_external_effects(&mut self) -> bool {
        let captures_settled = crate::native::capture::close(self.state);
        let presentations_settled = self.state.pending_presentations.is_empty();
        let atlas_settled = self
            .state
            .text_atlas_gpu
            .as_ref()
            .is_none_or(|gpu| gpu.pending_count() == 0)
            && self.state.text_atlas_in_flight.is_none()
            && self.state.text_atlas_recovery.is_none();
        captures_settled && presentations_settled && atlas_settled
    }

    fn release_derived_state(&mut self) {
        self.state.retained_draw_lists.clear();
        self.state.presentation_epochs.clear();
        self.state.accepted_cursor = None;
        self.state.lifecycle.clear_recovery();
        let atlas_cleared = self.state.text_atlas.clear();
        self.state.text_atlas_completion = None;
        self.state.text_pins_by_binding.clear();
        self.state.pending_text_presentations.clear();
        self.state.retained_frame_observations.clear();
        self.state.text_pin_frame_counts.clear();
        self.state.text_pin_frame_observations.clear();
        self.state.text_atlas_model_frame_digests.clear();
        self.state.text_atlas_plan_observations.clear();
        debug_assert!(atlas_cleared);
    }

    fn release_native_resources(&mut self) -> bool {
        if let Some(atlas_gpu) = self.state.text_atlas_gpu.take() {
            atlas_gpu
                .try_close(&mut self.state.resources)
                .unwrap_or_else(|_| panic!("settled atlas pages must close"));
        }
        match (
            self.state.device.take(),
            self.state.presentation_surface.take(),
        ) {
            (Some(device), Some(surface)) => {
                if let Err(owners) = crate::native::lifecycle::close_platform_owners(
                    device,
                    surface,
                    &mut self.state.resources,
                ) {
                    let (device, surface) = *owners;
                    self.state.device = Some(device);
                    self.state.presentation_surface = Some(surface);
                    return false;
                }
            }
            (None, None) => {}
            (device, surface) => {
                self.state.device = device;
                self.state.presentation_surface = surface;
                return false;
            }
        }
        if let Some(window) = self.state.window.take() {
            window.close(&mut self.state.resources);
        }
        let _ = self.state.physical_signal.shutdown();
        true
    }

    fn census(&self) -> Self::Census {
        self.state.current_resource_census()
    }

    fn terminal_zero(&self) -> bool {
        self.state.current_resource_census().is_zero()
    }
}
