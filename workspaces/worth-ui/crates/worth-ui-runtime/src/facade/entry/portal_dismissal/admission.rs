use super::{UiPortalDismissalPublicationStop as Stop, WorthUiActiveApplicationSession};
use crate::runtime::portal::{
    UiPortalDismissalIgnoreReason, UiPortalDismissalPreparation, UiPortalDismissalTrigger,
};

/// One Portal-owned decision made against the input's accepted presentation.
/// Moving it to publication preserves its target; it cannot be copied or forged.
#[must_use]
pub struct WorthUiAdmittedPortalDismissal {
    pub(super) generation: crate::facade::WorthUiActiveApplicationGenerationIdentity,
    pub(super) preparation: Result<UiPortalDismissalPreparation, Stop>,
}

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_portal_dismissal_interaction(
        &mut self,
        interaction: crate::facade::interaction::UiDismissInteraction,
    ) -> WorthUiAdmittedPortalDismissal {
        let preparation = self.resolve_portal_dismissal_interaction(interaction);
        WorthUiAdmittedPortalDismissal {
            generation: self.active_generation_identity(),
            preparation,
        }
    }

    fn resolve_portal_dismissal_interaction(
        &mut self,
        interaction: crate::facade::interaction::UiDismissInteraction,
    ) -> Result<UiPortalDismissalPreparation, Stop> {
        let Some(portal) = self.portal.as_ref() else {
            return Ok(UiPortalDismissalPreparation::Ignored(
                UiPortalDismissalIgnoreReason::NoMatchingPortal,
            ));
        };
        if portal.topmost_presentation().is_none() {
            return Ok(UiPortalDismissalPreparation::Ignored(
                UiPortalDismissalIgnoreReason::NoMatchingPortal,
            ));
        }
        let admitted = interaction.presentation();
        let surface = self
            .mounted
            .current_surface_for_binding(admitted.binding())
            .ok_or(Stop::StalePresentation)?;
        let trigger =
            super::trigger::dismissal_trigger(interaction, surface).ok_or(Stop::Transition)?;
        let target = if matches!(trigger, UiPortalDismissalTrigger::OutsidePress { .. }) {
            portal.dismissal_target_identity(trigger).map(|target| {
                crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
                    surface,
                    target.owner().mounted_instance_identity(),
                    target.diagnostic_value(),
                )
            })
        } else {
            None
        };
        // A press names what the reader saw. When a later publication has
        // superseded it, mounting says whether this Portal still looks the
        // same; only then is the press read at the current presentation.
        let presentation = match target {
            Some(target) => self
                .mounted
                .portal_dismissal_presentation(admitted, target)
                .map_err(|_| Stop::StalePresentation)?,
            // A key carries no geometry. While mounting still admits the frame
            // it was typed against, it applies to what is shown now, as queued
            // keys do; a frame mounting no longer admits stops it.
            None if matches!(trigger, UiPortalDismissalTrigger::Escape { .. }) => {
                self.mounted
                    .classify_interaction_presentation(admitted)
                    .map_err(|_| Stop::StalePresentation)?;
                self.mounted
                    .current_presentation_for_surface(surface)
                    .ok_or(Stop::StalePresentation)?
            }
            None => admitted,
        };
        let sampled_bounds = match target {
            Some(target) => self
                .mounted
                .committed_motion_geometry_for_target(target, presentation)
                .map_err(|_| Stop::StalePresentation)?,
            None => None,
        };
        self.prepare_portal_dismissal_decision(trigger, sampled_bounds, presentation)
    }

    pub(super) fn prepare_portal_dismissal_trigger(
        &mut self,
        trigger: UiPortalDismissalTrigger,
        sampled_bounds: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> WorthUiAdmittedPortalDismissal {
        let preparation =
            self.prepare_portal_dismissal_decision(trigger, sampled_bounds, presentation);
        WorthUiAdmittedPortalDismissal {
            generation: self.active_generation_identity(),
            preparation,
        }
    }

    fn prepare_portal_dismissal_decision(
        &mut self,
        trigger: UiPortalDismissalTrigger,
        sampled_bounds: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
    ) -> Result<UiPortalDismissalPreparation, Stop> {
        self.mounted
            .current_semantic_surface_for_presentation(presentation)
            .map_err(|_| Stop::StalePresentation)?;
        let lineage = self.next_portal_service_event_identity;
        self.next_portal_service_event_identity =
            lineage.checked_add(1).ok_or(Stop::IdentityExhausted)?;
        let idempotency =
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
                self.session_identity().as_u64(),
                lineage,
            );
        let preparation = self
            .portal
            .as_ref()
            .ok_or(Stop::Transition)?
            .prepare_dismissal(trigger, sampled_bounds, idempotency)
            .map_err(|_| Stop::Transition)?;
        if let UiPortalDismissalPreparation::Prepared(dismissal) = &preparation {
            if dismissal.presentation() != presentation {
                return Err(Stop::StalePresentation);
            }
        }
        Ok(preparation)
    }
}
