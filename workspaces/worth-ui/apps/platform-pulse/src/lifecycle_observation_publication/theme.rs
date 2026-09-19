use super::{PlatformPulseObservationPublicationDenial, PlatformPulseObservationPublisher};

impl PlatformPulseObservationPublisher {
    pub(crate) fn theme_switch_settled(
        &self,
        preference_revision: u64,
        definition: &worth_ui::facade::appearance::UiThemeDefinitionIdentity,
        binding_generation: u64,
        publication: Option<&worth_ui::facade::app::UiMountedFramePublicationReceipt>,
    ) -> Result<(), PlatformPulseObservationPublicationDenial> {
        self.project_observation(|stream| {
            stream.project_theme_switch_settled(
                preference_revision,
                definition,
                binding_generation,
                publication,
            )
        })
    }
}
