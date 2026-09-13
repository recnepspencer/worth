use super::WorthUiNativeApplicationShell;

impl WorthUiNativeApplicationShell {
    pub(crate) fn current_presentation_attribution(
        &self,
        observed: &worth_ui_host_native::UiNativePresentationObservation,
    ) -> Option<worth_ui_host_native::UiNativeClientPresentationAttribution> {
        let publication = self.session.mounted.current_publication()?;
        let frame = publication.frame();
        if frame.diagnostic_value() != observed.presented_frame()
            || publication.attempt().diagnostic_value() != observed.presentation_attempt()
        {
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
        Some(
            worth_ui_host_native::UiNativeClientPresentationAttribution::reported(
                [
                    frame.diagnostic_value(),
                    attribution.surface.diagnostic_value(),
                    binding.diagnostic_value(),
                    attribution.mounted_instance.diagnostic_value(),
                    attribution.node_receipt.diagnostic_value(),
                    publication.attempt().diagnostic_value(),
                ],
                [
                    attribution.authored_provenance_digest,
                    attribution.authored_semantic_identity_digest,
                ],
            ),
        )
    }
}
