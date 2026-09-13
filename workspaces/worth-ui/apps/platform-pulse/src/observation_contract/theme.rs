use super::{
    PlatformPulseLifecycleObservation, PlatformPulseLifecycleObservationEnvelope,
    PlatformPulseLifecycleObservationProjectionDenial, PlatformPulseLifecycleObservationStream,
    PlatformPulseMountedFrameObservation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum PlatformPulseThemeSettlementPosture {
    Published,
    Unchanged,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PlatformPulseThemeSwitchSettled {
    pub preference_revision: u64,
    pub definition: String,
    pub binding_generation: u64,
    pub frame: PlatformPulseMountedFrameObservation,
    pub posture: PlatformPulseThemeSettlementPosture,
    pub selected_instances: usize,
    pub materialized_contexts: usize,
}

impl PlatformPulseLifecycleObservationStream {
    pub fn project_theme_switch_settled(
        &mut self,
        preference_revision: u64,
        definition: &worth_ui::facade::appearance::UiThemeDefinitionIdentity,
        binding_generation: u64,
        publication: Option<&worth_ui::facade::app::UiMountedFramePublicationReceipt>,
    ) -> Result<
        PlatformPulseLifecycleObservationEnvelope,
        PlatformPulseLifecycleObservationProjectionDenial,
    > {
        let validated = publication
            .map(|receipt| self.validate_content_publication(receipt))
            .transpose()?;
        let frame = match &validated {
            Some(publication) => publication.frame(),
            None => self.published_predecessor()?.2,
        };
        let next_visual_state = if validated.is_some() {
            self.visual_state
                .after_content_publication(frame.diagnostic_value())?
        } else {
            self.visual_state
        };
        let (posture, selected_instances, materialized_contexts) = match publication {
            Some(receipt) => {
                let appearance = receipt.cost_report().appearance();
                (
                    PlatformPulseThemeSettlementPosture::Published,
                    appearance.selected_instance_count(),
                    appearance.materialized_context_count(),
                )
            }
            None => (PlatformPulseThemeSettlementPosture::Unchanged, 0, 0),
        };
        let envelope =
            self.next_envelope(PlatformPulseLifecycleObservation::ThemeSwitchSettled(
                PlatformPulseThemeSwitchSettled {
                    preference_revision,
                    definition: definition.as_str().to_owned(),
                    binding_generation,
                    frame,
                    posture,
                    selected_instances,
                    materialized_contexts,
                },
            ))?;
        if let Some(publication) = validated {
            self.commit_content_publication(publication);
        }
        self.visual_state = next_visual_state;
        Ok(envelope)
    }
}
