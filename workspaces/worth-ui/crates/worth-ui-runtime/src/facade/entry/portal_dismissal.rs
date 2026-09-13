use super::WorthUiActiveApplicationSession;

#[path = "portal_dismissal/completion.rs"]
mod completion;
#[path = "portal_dismissal/handles.rs"]
mod handles;
#[path = "portal_dismissal/receipt.rs"]
mod receipt;
#[path = "portal_dismissal/trigger.rs"]
mod trigger;
pub use receipt::UiPortalDismissalPublicationReceipt;
use trigger::dismissal_trigger;
pub(crate) enum UiPortalDismissalPublicationOutcome<'session> {
    IgnoredNoMatchingPortal,
    IgnoredInsideTopmostPortal,
    Published(UiPortalDismissalPublicationReceipt),
    InFlight(UiPortalDismissalPublicationCompletion<'session>),
    Indeterminate(UiPortalDismissalPublicationRecovery<'session>),
    Stopped(UiPortalDismissalPublicationStop),
}

pub(in crate::facade::entry) fn present_portal_service_proposal(
    session: &mut WorthUiActiveApplicationSession,
    frame: crate::mounting::UiPreparedMountedFrame,
    proposal: crate::runtime::session::UiStagedPortalProposalTransaction,
    retain_exit: bool,
    now_tick: u64,
) -> UiPortalDismissalPublicationOutcome<'_> {
    let outcome = session.present_prepared_portal_frame_internal(
        frame,
        &proposal,
        retain_exit,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
        now_tick,
    );
    completion::finish_presented(
        UiPortalDismissalAdmitted {
            session,
            proposal: Some(proposal),
            retain_exit,
        },
        outcome,
        now_tick,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalDismissalPublicationStop {
    IdentityExhausted,
    StalePresentation,
    Transition,
    Proposal,
    Preparation,
    HostRejectedBeforeEffects,
    MountedRetention,
    MountedPresentation,
    Superseded,
}

#[must_use = "portal dismissal presentation must be completed or cancelled"]
pub(crate) struct UiPortalDismissalPublicationCompletion<'session> {
    state: Option<Box<UiPortalDismissalInFlight<'session>>>,
}

#[must_use = "indeterminate portal dismissal requires shutdown reconciliation"]
pub(crate) struct UiPortalDismissalPublicationRecovery<'session> {
    state: Option<Box<UiPortalDismissalIndeterminate<'session>>>,
}

struct UiPortalDismissalAdmitted<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    proposal: Option<crate::runtime::session::UiStagedPortalProposalTransaction>,
    retain_exit: bool,
}

struct UiPortalDismissalInFlight<'session> {
    admitted: UiPortalDismissalAdmitted<'session>,
    mounted: crate::mounting::UiMountedPresentationInFlight,
}

struct UiPortalDismissalIndeterminate<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    frame: crate::mounting::UiMountedIndeterminateFrame,
    proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
}

pub(in crate::facade::entry) struct DetachedUiPortalDismissalInFlight {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    proposal: crate::runtime::session::UiStagedPortalProposalTransaction,
    mounted: crate::mounting::UiMountedPresentationInFlight,
    retain_exit: bool,
}

pub(in crate::facade::entry) struct DetachedUiPortalDismissalIndeterminate {
    session: crate::facade::WorthUiActiveApplicationSessionIdentity,
    frame: crate::mounting::UiMountedIndeterminateFrame,
    proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
}

impl WorthUiActiveApplicationSession {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn topmost_portal_presentation_for_certification(
        &self,
    ) -> Option<worth_ui_host_contract::UiHostObservationPresentationBasis> {
        self.portal
            .as_ref()
            .and_then(crate::runtime::portal::UiPortalRuntimeState::topmost_presentation)
    }

    pub(crate) fn publish_portal_dismissal(
        &mut self,
        interaction: crate::facade::interaction::UiDismissInteraction,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'_> {
        if self
            .portal
            .as_ref()
            .and_then(|portal| portal.topmost_presentation())
            .is_none()
        {
            return UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal;
        }
        let presentation = interaction.presentation();
        let semantic_surface = match self
            .mounted
            .classify_admitted_interaction_presentation(presentation)
        {
            Ok(_) => match self
                .mounted
                .current_surface_for_binding(presentation.binding())
            {
                Some(surface) => surface,
                None => {
                    return UiPortalDismissalPublicationOutcome::Stopped(
                        UiPortalDismissalPublicationStop::Transition,
                    );
                }
            },
            Err(crate::mounting::UiPresentedFrameBasisDenial::Expired) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::StalePresentation,
                );
            }
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Transition,
                );
            }
        };
        let trigger = match dismissal_trigger(interaction, semantic_surface) {
            Some(trigger) => trigger,
            None => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Transition,
                );
            }
        };
        let sampled_bounds = if matches!(
            trigger,
            crate::runtime::portal::UiPortalDismissalTrigger::OutsidePress { .. }
        ) {
            match self
                .portal
                .as_ref()
                .expect("Portal installation was checked above")
                .dismissal_target_identity(trigger)
            {
                Some(portal) => match self.mounted.committed_motion_geometry_for_target(
                    crate::runtime::motion::UiMotionTargetIdentity::from_portal_owner(
                        semantic_surface,
                        portal.owner().mounted_instance_identity(),
                        portal.diagnostic_value(),
                    ),
                    interaction.presentation(),
                ) {
                    Ok(bounds) => bounds,
                    Err(_) => {
                        return UiPortalDismissalPublicationOutcome::Stopped(
                            UiPortalDismissalPublicationStop::Transition,
                        );
                    }
                },
                None => None,
            }
        } else {
            None
        };
        self.publish_portal_dismissal_trigger(
            trigger,
            sampled_bounds,
            Some(interaction.presentation()),
            now_tick,
        )
    }

    pub(in crate::facade::entry) fn publish_anchor_loss_portal_dismissal(
        &mut self,
        portal: crate::runtime::portal::UiPortalIdentity,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'_> {
        let Some(presentation) = self
            .portal
            .as_ref()
            .and_then(|state| state.committed_presentation_for(portal))
        else {
            return UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal;
        };
        self.publish_portal_dismissal_trigger(
            crate::runtime::portal::UiPortalDismissalTrigger::AnchorLoss(portal),
            None,
            Some(presentation),
            now_tick,
        )
    }

    fn publish_portal_dismissal_trigger(
        &mut self,
        trigger: crate::runtime::portal::UiPortalDismissalTrigger,
        sampled_bounds: Option<worth_ui_host_contract::UiMountedCanonicalBox>,
        expected_presentation: Option<worth_ui_host_contract::UiHostObservationPresentationBasis>,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'_> {
        if !self.portal.is_installed() {
            return UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal;
        }
        if !self.focus.is_installed() || !self.motion.is_installed() {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::Proposal,
            );
        }
        let expected_relation = match expected_presentation
            .map(|presentation| {
                self.mounted
                    .classify_admitted_interaction_presentation(presentation)
            })
            .transpose()
        {
            Ok(relation) => relation,
            Err(crate::mounting::UiPresentedFrameBasisDenial::Expired) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::StalePresentation,
                );
            }
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Transition,
                );
            }
        };
        let lineage = self.next_portal_service_event_identity;
        self.next_portal_service_event_identity = match lineage.checked_add(1) {
            Some(next) => next,
            None => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::IdentityExhausted,
                );
            }
        };
        let idempotency =
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
                self.session_identity().as_u64(),
                lineage,
            );
        let dismissal = match self
            .portal
            .as_ref()
            .expect("Portal installation was checked above")
            .prepare_dismissal(trigger, sampled_bounds, idempotency)
        {
            Ok(crate::runtime::portal::UiPortalDismissalPreparation::Ignored(reason)) => {
                return match reason {
                    crate::runtime::portal::UiPortalDismissalIgnoreReason::NoMatchingPortal => {
                        UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal
                    }
                    crate::runtime::portal::UiPortalDismissalIgnoreReason::InsideTopmostPortal => {
                        UiPortalDismissalPublicationOutcome::IgnoredInsideTopmostPortal
                    }
                };
            }
            Ok(crate::runtime::portal::UiPortalDismissalPreparation::Prepared(dismissal)) => {
                dismissal
            }
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Transition,
                );
            }
        };
        if expected_relation == Some(crate::mounting::UiPresentedFrameBasisRelation::Retained) {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::StalePresentation,
            );
        }
        if expected_presentation.is_some_and(|expected| dismissal.presentation() != expected) {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::Transition,
            );
        }
        let presentation = dismissal.presentation();
        let transition = dismissal.into_transition();
        let revision = transition.successor_revision();
        let overlays = self
            .portal
            .as_ref()
            .expect("Portal installation was checked above")
            .mounted_projection_inputs(&transition, transition.closes_portal());
        let motion_request = match self.prepare_portal_motion_request(&transition) {
            Ok(request) => request,
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Proposal,
                );
            }
        };
        let preparation = match self.application.begin_portal_dismissal_service_proposal(
            transition,
            presentation,
            self.active_generation_identity(),
            self.motion
                .as_mut()
                .expect("Motion installation was checked above"),
            motion_request,
        ) {
            Ok(preparation) => preparation,
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Proposal,
                );
            }
        };
        let frame = match self.prepare_intent_consequence_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            revision,
            overlays,
        ) {
            Ok(frame) => frame,
            Err(_) => {
                self.application.cancel_portal_service_proposal_preparation(
                    preparation,
                    self.motion
                        .as_mut()
                        .expect("Motion installation was checked above"),
                );
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Preparation,
                );
            }
        };
        let scroll_incarnation = self.scroll_owner_incarnation();
        let proposal = match self.application.bind_portal_service_proposal_frame(
            preparation,
            &frame,
            &self.mounted,
            self.focus
                .as_mut()
                .expect("Focus installation was checked above"),
            self.scroll.as_ref(),
            scroll_incarnation,
            self.motion
                .as_mut()
                .expect("Motion installation was checked above"),
        ) {
            Ok(proposal) => proposal,
            Err(_) => {
                return UiPortalDismissalPublicationOutcome::Stopped(
                    UiPortalDismissalPublicationStop::Proposal,
                );
            }
        };
        present_portal_service_proposal(self, frame, proposal, true, now_tick)
    }
}
