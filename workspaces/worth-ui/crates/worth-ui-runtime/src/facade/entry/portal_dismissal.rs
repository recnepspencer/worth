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
#[path = "portal_dismissal/admission.rs"]
mod admission;
pub use admission::WorthUiAdmittedPortalDismissal;
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
    InteractionCancelled,
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

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn publish_portal_dismissal(
        &mut self,
        interaction: crate::facade::interaction::UiDismissInteraction,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'_> {
        let admitted = self.prepare_portal_dismissal_interaction(interaction);
        self.publish_admitted_portal_dismissal(admitted, now_tick)
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
        let admitted = self.prepare_portal_dismissal_trigger(
            crate::runtime::portal::UiPortalDismissalTrigger::AnchorLoss(portal),
            None,
            presentation,
        );
        self.publish_admitted_portal_dismissal(admitted, now_tick)
    }

    pub(crate) fn publish_admitted_portal_dismissal(
        &mut self,
        admitted: WorthUiAdmittedPortalDismissal,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'_> {
        use crate::runtime::portal::{
            UiPortalDismissalIgnoreReason as Ignore, UiPortalDismissalPreparation as Preparation,
        };
        if admitted.generation != self.active_generation_identity() {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::InteractionCancelled,
            );
        }
        let dismissal = match admitted.preparation {
            Ok(Preparation::Ignored(Ignore::NoMatchingPortal)) => {
                return UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal
            }
            Ok(Preparation::Ignored(Ignore::InsideTopmostPortal)) => {
                return UiPortalDismissalPublicationOutcome::IgnoredInsideTopmostPortal
            }
            Ok(Preparation::Prepared(dismissal)) => dismissal,
            Err(stop) => return UiPortalDismissalPublicationOutcome::Stopped(stop),
        };
        let Some((transition, presentation)) = self
            .portal
            .as_ref()
            .and_then(|portal| portal.continue_prepared_dismissal(dismissal))
        else {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::InteractionCancelled,
            );
        };
        if self
            .mounted
            .current_semantic_surface_for_presentation(presentation)
            .is_err()
        {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::InteractionCancelled,
            );
        }
        if !self.focus.is_installed() || !self.motion.is_installed() {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::Proposal,
            );
        }
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
