use super::{
    WorthUiActiveApplicationSession, WorthUiApplicationCutoverDenial,
    WorthUiPreparedApplicationActivation,
};
use worth_ui_dsl::UiAppearanceStateAxis;

pub(super) struct UiPreparedApplicationOwnerSuccession {
    text: crate::runtime::presentation_state::UiPreparedApplicationTextSuccession,
    interaction: crate::runtime::interaction::UiPreparedInteractionApplicationRebind,
    operability: crate::runtime::intent::UiPreparedIntentAdmissionRebind,
    validation: crate::runtime::intent::UiIntentApplicationFactState,
    pub(super) focus: Option<crate::runtime::focus::UiPreparedFocusMountedReconciliation>,
    selection: super::selection_replacement::UiPreparedSelectionReplacement,
    prior_selection: Option<crate::runtime::selection::UiSelectionAppearanceOwnerSnapshot>,
    prior_validation: Option<crate::runtime::intent::UiValidationAppearanceFactSnapshot>,
    snapshot: Option<crate::runtime::appearance::UiAppearanceOwnerSnapshot>,
}

pub(super) enum UiApplicationOwnerCutover {
    Unmounted {
        selection: super::selection_replacement::UiPreparedSelectionReplacement,
        text: crate::runtime::presentation_state::UiPreparedApplicationTextSuccession,
    },
    Mounted(UiPreparedApplicationOwnerSuccession),
}

impl UiPreparedApplicationOwnerSuccession {
    pub(super) fn prepare(
        session: &WorthUiActiveApplicationSession,
        application: &WorthUiPreparedApplicationActivation,
        mounted: &crate::mounting::UiMountedGraphReplacementSuccessor,
        frame: &crate::mounting::UiAssembledMountedFrame,
        text: crate::runtime::presentation_state::UiPreparedApplicationTextSuccession,
    ) -> Result<Self, WorthUiApplicationCutoverDenial> {
        let authority = application.candidate_replacement_authority();
        let demand = authority.consumed_fact_index().appearance_axis_demand();
        let interaction = session
            .interaction
            .prepare_application_rebind(
                demand.contains(UiAppearanceStateAxis::Hover)
                    || authority.intent_catalog().has_activation_routes(),
                demand.contains(UiAppearanceStateAxis::Pressed),
            )
            .map_err(|_| WorthUiApplicationCutoverDenial::AppearanceOwnerSuccessionUnavailable)?;
        let operability = session
            .intent_admission
            .prepare_application_rebind(demand.contains(UiAppearanceStateAxis::Operability));
        let validation = crate::runtime::intent::UiIntentApplicationFactState::activate(
            authority.intent_application_fact_plan(),
            demand.contains(UiAppearanceStateAxis::Validation),
        );
        let focus = application
            .candidate_service_policy_plan()
            .focus()
            .map(|_| {
                let participation = mounted.focus_participation_snapshot(frame);
                session.focus.as_ref().map_or_else(
                    || {
                        crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate()
                            .prepare_mounted_reconciliation(&participation)
                    },
                    |focus| focus.prepare_mounted_reconciliation(&participation),
                )
            })
            .transpose()
            .map_err(|_| WorthUiApplicationCutoverDenial::AppearanceOwnerSuccessionUnavailable)?;
        let selection = session.prepare_selection_replacement(application, mounted, Some(frame));
        let themes = application
            .appearance_succession
            .as_ref()
            .expect("replacement prepared themes")
            .theme();
        let snapshot = session
            .appearance_owner_snapshot
            .as_ref()
            .map(|prior| {
                prior
                    .prepare_succession(
                        themes,
                        demand,
                        &interaction,
                        &operability,
                        &validation,
                        focus.as_ref(),
                        selection.appearance_snapshot(),
                    )
                    .ok_or(WorthUiApplicationCutoverDenial::AppearanceOwnerSuccessionUnavailable)
            })
            .transpose()?;
        if snapshot.is_none()
            && !authority
                .consumed_fact_index()
                .appearance_required_role_identities()
                .is_empty()
        {
            return Err(WorthUiApplicationCutoverDenial::AppearanceOwnerSuccessionUnavailable);
        }
        Ok(Self {
            text,
            prior_selection: session
                .selection
                .as_ref()
                .map(|owner| owner.appearance_owner_snapshot()),
            prior_validation: session
                .intent_application_facts
                .validation_appearance_snapshot(),
            interaction,
            operability,
            validation,
            focus,
            selection,
            snapshot,
        })
    }

    pub(super) fn snapshot(
        &self,
    ) -> Option<&crate::runtime::appearance::UiAppearanceOwnerSnapshot> {
        self.snapshot.as_ref()
    }

    pub(super) fn is_current(&self, session: &WorthUiActiveApplicationSession) -> bool {
        if !self.text.is_current(&session.presentation) {
            return false;
        }
        if self.prior_selection
            != session
                .selection
                .as_ref()
                .map(|owner| owner.appearance_owner_snapshot())
            || self.prior_validation
                != session
                    .intent_application_facts
                    .validation_appearance_snapshot()
            || self.snapshot.as_ref().is_some_and(|snapshot| {
                snapshot.succession_predecessor() != Some(&session.active_generation_identity())
            })
        {
            return false;
        }
        if let Some(prepared) = &self.focus {
            let current = session.focus.as_ref().map_or_else(
                || {
                    crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate()
                        .admits_mounted_reconciliation(prepared)
                },
                |focus| focus.admits_mounted_reconciliation(prepared),
            );
            if !current {
                return false;
            }
        }

        session
            .interaction
            .validate_application_rebind(&self.interaction)
            .is_ok()
            && session
                .intent_admission
                .admits_application_rebind(&self.operability)
    }

    pub(super) fn commit(self, session: &mut WorthUiActiveApplicationSession) {
        self.text.commit(&mut session.presentation);
        let previous_input = session.interaction.active_input_binding();
        session
            .interaction
            .commit_application_rebind(self.interaction);
        session.clear_displaced_input_recipient(previous_input);
        session
            .intent_admission
            .commit_application_rebind(&mut session.intent_execution, self.operability);
        session.intent_application_facts = self.validation;
        session.selection = self.selection.into_state();
        session.appearance_owner_snapshot = self.snapshot;
        session.pointer_affordance_snapshot = None;
    }
}
