pub(crate) struct UiAppearanceObservationCloseInput<'owner> {
    demand: crate::runtime::appearance::UiAppearanceStateAxisDemand,
    generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    focus: Option<&'owner crate::runtime::focus::UiFocusRuntimeState>,
    selection: Option<&'owner crate::runtime::selection::UiSelectionRuntimeState>,
    operability: &'owner crate::runtime::intent::UiIntentAdmissionState,
    validation: &'owner crate::runtime::intent::UiIntentApplicationFactState,
    interaction: &'owner crate::runtime::interaction::UiInteractionRuntimeState,
}

impl<'owner> UiAppearanceObservationCloseInput<'owner> {
    #[allow(
        clippy::too_many_arguments,
        reason = "the close input borrows each independent appearance state owner"
    )]
    pub(crate) const fn new(
        demand: crate::runtime::appearance::UiAppearanceStateAxisDemand,
        generation: crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        focus: Option<&'owner crate::runtime::focus::UiFocusRuntimeState>,
        selection: Option<&'owner crate::runtime::selection::UiSelectionRuntimeState>,
        operability: &'owner crate::runtime::intent::UiIntentAdmissionState,
        validation: &'owner crate::runtime::intent::UiIntentApplicationFactState,
        interaction: &'owner crate::runtime::interaction::UiInteractionRuntimeState,
    ) -> Self {
        Self {
            demand,
            generation,
            focus,
            selection,
            operability,
            validation,
            interaction,
        }
    }

    pub(super) fn seal(
        self,
        authority: &super::UiObservationTurnCloseAuthority,
        turn: super::UiObservationTurnIdentity,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        source_basis: u64,
    ) -> crate::runtime::appearance::UiAppearanceOwnerSnapshot {
        crate::runtime::appearance::UiAppearanceOwnerSnapshot::seal_at_turn_close(
            authority,
            turn,
            session,
            source_basis,
            self.generation,
            self.demand,
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Focus)
                .then(|| {
                    self.focus
                        .expect("launch admitted demanded focus owner")
                        .appearance_posture()
                }),
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Selection)
                .then(|| {
                    self.selection
                        .expect("launch admitted demanded selection owner")
                        .appearance_owner_snapshot()
                }),
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Operability)
                .then(|| {
                    self.operability
                        .operability_standing_snapshot()
                        .expect("launch admitted demanded operability owner")
                }),
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Validation)
                .then(|| {
                    self.validation
                        .validation_appearance_snapshot()
                        .expect("launch admitted demanded validation owner")
                }),
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Hover)
                .then(|| {
                    self.interaction
                        .pointer_presence_appearance_snapshot()
                        .expect("launch admitted demanded pointer-presence owner")
                }),
            self.demand
                .contains(worth_ui_dsl::UiAppearanceStateAxis::Pressed)
                .then(|| self.interaction.pressed_appearance_snapshot()),
        )
    }
}
