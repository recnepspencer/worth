use super::WorthUiNativeApplicationShell;

impl WorthUiNativeApplicationShell {
    pub(crate) fn current_presentation_attribution(
        &self,
        retained: &worth_ui_host_native::UiNativeRetainedFrameObservation,
    ) -> Option<worth_ui_host_native::UiNativeClientPresentationAttribution> {
        let observed = retained.presentation()?;
        let publication = self.session.mounted.current_publication()?;
        let frame = publication.frame();
        if frame.diagnostic_value() != observed.presented_frame() {
            return None;
        }
        let binding = *publication
            .bindings()
            .iter()
            .find(|binding| binding.diagnostic_value() == observed.binding_generation())?;
        let attribution = self.session.mounted.native_observed_paint_attribution(
            frame,
            binding,
            observed.semantic_surface(),
            observed.mounted_instance(),
            observed.node_receipt(),
        )?;
        let current = self
            .session
            .mounted
            .current_presentation_for_surface(attribution.surface)?;
        if !retained.matches_runtime_attribution_basis(publication.attempt(), current) {
            return None;
        }
        Some(
            worth_ui_host_native::UiNativeClientPresentationAttribution::reported(
                [
                    frame.diagnostic_value(),
                    attribution.surface.diagnostic_value(),
                    binding.diagnostic_value(),
                    attribution.mounted_instance.diagnostic_value(),
                    attribution.node_receipt.diagnostic_value(),
                    observed.presentation_attempt(),
                ],
                [
                    attribution.authored_provenance_digest,
                    attribution.authored_semantic_identity_digest,
                ],
            ),
        )
    }
}
