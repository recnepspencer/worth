use super::UiAppearanceOwnerSnapshot;

impl UiAppearanceOwnerSnapshot {
    #[allow(
        clippy::too_many_arguments,
        reason = "succession consumes evidence from each independent owner"
    )]
    pub(crate) fn prepare_succession(
        &self,
        themes: &crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        demand: crate::runtime::appearance::UiAppearanceStateAxisDemand,
        interaction: &crate::runtime::interaction::UiPreparedInteractionApplicationRebind,
        operability: &crate::runtime::intent::UiPreparedIntentAdmissionRebind,
        validation: &crate::runtime::intent::UiIntentApplicationFactState,
        focus: Option<&crate::runtime::focus::UiPreparedFocusMountedReconciliation>,
        selection: Option<crate::runtime::selection::UiSelectionAppearanceOwnerSnapshot>,
    ) -> Option<Self> {
        if self.generation() != themes.predecessor()
            || self.session() != themes.successor().session_identity()
        {
            return None;
        }
        Some(Self {
            turn: self.turn,
            session: self.session,
            source_basis: self.source_basis,
            predecessor_generation: Some(self.generation.clone()),
            generation: themes.successor().clone(),
            demand,
            focus: focus.map(|owner| owner.appearance_posture()),
            selection,
            operability: operability.appearance_snapshot(),
            validation: validation.validation_appearance_snapshot(),
            pointer_presence: interaction.pointer_presence(),
            pressed: Some(interaction.pressed()),
        })
    }

    pub(crate) fn succession_predecessor(
        &self,
    ) -> Option<&crate::runtime::WorthUiActiveApplicationGenerationIdentity> {
        self.predecessor_generation.as_ref()
    }
}
