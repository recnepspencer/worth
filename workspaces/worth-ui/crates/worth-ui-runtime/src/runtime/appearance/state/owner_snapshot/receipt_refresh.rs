use super::UiAppearanceOwnerSnapshot;

impl UiAppearanceOwnerSnapshot {
    pub(crate) fn refresh_receipt_sources(
        &self,
        focus: Option<&crate::runtime::focus::UiFocusRuntimeState>,
        selection: Option<&crate::runtime::selection::UiSelectionRuntimeState>,
        operability: &crate::runtime::intent::UiIntentAdmissionState,
        validation: &crate::runtime::intent::UiIntentApplicationFactState,
        interaction: &crate::runtime::interaction::UiInteractionRuntimeState,
    ) -> Option<Self> {
        Some(Self {
            turn: self.turn,
            session: self.session,
            source_basis: self.source_basis,
            generation: self.generation.clone(),
            predecessor_generation: self.predecessor_generation.clone(),
            demand: self.demand,
            focus: if self.focus.is_some() {
                Some(focus?.appearance_posture())
            } else {
                None
            },
            selection: if self.selection.is_some() {
                Some(selection?.appearance_owner_snapshot())
            } else {
                None
            },
            operability: if self.operability.is_some() {
                Some(operability.operability_standing_snapshot()?)
            } else {
                None
            },
            validation: if self.validation.is_some() {
                Some(validation.validation_appearance_snapshot()?)
            } else {
                None
            },
            pointer_presence: if self.pointer_presence.is_some() {
                Some(interaction.pointer_presence_appearance_snapshot()?)
            } else {
                None
            },
            pressed: self
                .pressed
                .as_ref()
                .map(|_| interaction.pressed_appearance_snapshot()),
        })
    }
}
