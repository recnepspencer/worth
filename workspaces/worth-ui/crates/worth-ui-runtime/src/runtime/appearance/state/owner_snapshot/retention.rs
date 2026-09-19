use super::UiAppearanceOwnerSnapshot;

/// Paired scopes for one unchanged set of live owner exports.
pub(crate) struct UiPreparedRetainedAppearanceOwnerSuccession {
    projection: Option<UiAppearanceOwnerSnapshot>,
    successor: Option<UiAppearanceOwnerSnapshot>,
}

impl UiPreparedRetainedAppearanceOwnerSuccession {
    pub(crate) fn prepare(
        predecessor: Option<&UiAppearanceOwnerSnapshot>,
        themes: &crate::runtime::presentation_state::UiPreparedAppearanceGenerationSuccession,
        focus: Option<&crate::runtime::focus::UiFocusRuntimeState>,
        selection: Option<&crate::runtime::selection::UiSelectionRuntimeState>,
        operability: &crate::runtime::intent::UiIntentAdmissionState,
        validation: &crate::runtime::intent::UiIntentApplicationFactState,
        interaction: &crate::runtime::interaction::UiInteractionRuntimeState,
        successor_source_basis: u64,
    ) -> Option<Self> {
        let Some(prior) = predecessor else {
            return Some(Self {
                projection: None,
                successor: None,
            });
        };
        if prior.generation() != themes.predecessor()
            || prior.session() != themes.successor().session_identity()
        {
            return None;
        }
        let projection = UiAppearanceOwnerSnapshot {
            turn: prior.turn,
            session: prior.session,
            source_basis: prior.source_basis,
            generation: themes.predecessor().clone(),
            predecessor_generation: prior.predecessor_generation.clone(),
            demand: prior.demand,
            focus: if prior.focus.is_some() {
                Some(focus?.appearance_posture())
            } else {
                None
            },
            selection: if prior.selection.is_some() {
                Some(selection?.appearance_owner_snapshot())
            } else {
                None
            },
            operability: if prior.operability.is_some() {
                Some(operability.operability_standing_snapshot()?)
            } else {
                None
            },
            validation: if prior.validation.is_some() {
                Some(validation.validation_appearance_snapshot()?)
            } else {
                None
            },
            pointer_presence: if prior.pointer_presence.is_some() {
                Some(interaction.pointer_presence_appearance_snapshot()?)
            } else {
                None
            },
            pressed: prior
                .pressed
                .as_ref()
                .map(|_| interaction.pressed_appearance_snapshot()),
        };
        if projection.changed_axes(prior) != Default::default() {
            return None;
        }
        let successor = UiAppearanceOwnerSnapshot {
            generation: themes.successor().clone(),
            predecessor_generation: Some(themes.predecessor().clone()),
            source_basis: successor_source_basis,
            ..projection.clone()
        };
        Some(Self {
            projection: Some(projection),
            successor: Some(successor),
        })
    }

    pub(crate) fn projection(&self) -> Option<&UiAppearanceOwnerSnapshot> {
        self.projection.as_ref()
    }

    pub(crate) fn generations(
        &self,
    ) -> Option<(
        &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
        &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    )> {
        Some((
            self.projection.as_ref()?.generation(),
            self.successor.as_ref()?.generation(),
        ))
    }

    pub(crate) fn matches_projection(&self, current: Option<&UiAppearanceOwnerSnapshot>) -> bool {
        match (self.projection.as_ref(), current) {
            (None, None) => true,
            (Some(prepared), Some(current)) => {
                !prepared.requires_initial_invalidation(current)
                    && prepared.turn == current.turn
                    && prepared.changed_axes(current) == Default::default()
            }
            _ => false,
        }
    }

    pub(crate) fn into_successor(self) -> Option<UiAppearanceOwnerSnapshot> {
        self.successor
    }
}
