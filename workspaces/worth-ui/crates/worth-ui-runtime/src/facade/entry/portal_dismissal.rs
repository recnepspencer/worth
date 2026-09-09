use super::WorthUiActiveApplicationSession;

#[path = "portal_dismissal/completion.rs"]
mod completion;
#[path = "portal_dismissal/handles.rs"]
mod handles;
#[path = "portal_dismissal/receipt.rs"]
mod receipt;
pub use receipt::UiPortalDismissalPublicationReceipt;
pub(crate) enum UiPortalDismissalPublicationOutcome<'session> {
    IgnoredNoMatchingPortal,
    IgnoredInsideTopmostPortal,
    Published(UiPortalDismissalPublicationReceipt),
    InFlight(UiPortalDismissalPublicationCompletion<'session>),
    Indeterminate(UiPortalDismissalPublicationRecovery<'session>),
    Stopped(UiPortalDismissalPublicationStop),
}

pub(in crate::facade::entry) fn finish_portal_service_proposal(
    session: &mut WorthUiActiveApplicationSession,
    proposal: crate::runtime::session::UiStagedPortalProposalTransaction,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiPortalDismissalPublicationOutcome<'_> {
    completion::finish(
        UiPortalDismissalAdmitted {
            session,
            proposal: Some(proposal),
        },
        outcome,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPortalDismissalPublicationStop {
    IdentityExhausted,
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
        if !self.portal.is_installed() {
            return UiPortalDismissalPublicationOutcome::IgnoredNoMatchingPortal;
        }
        let semantic_surface = match self
            .mounted
            .current_semantic_surface_for_presentation(interaction.presentation())
        {
            Ok(surface) => surface,
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
                Some(portal) => match self.mounted.committed_motion_geometry_for_instance(
                    portal.owner().mounted_instance_identity(),
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
        if expected_presentation.is_some_and(|presentation| {
            crate::runtime::interaction::targeting::require_current_presentation(
                &self.mounted,
                presentation,
            )
            .is_err()
        }) {
            return UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::Transition,
            );
        }
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
        let outcome = self.present_prepared_portal_frame_internal(
            frame,
            &proposal,
            true,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            now_tick,
        );
        completion::finish(
            UiPortalDismissalAdmitted {
                session: self,
                proposal: Some(proposal),
            },
            outcome,
        )
    }
}

fn dismissal_trigger(
    interaction: crate::facade::interaction::UiDismissInteraction,
    semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
) -> Option<crate::runtime::portal::UiPortalDismissalTrigger> {
    match interaction.cause() {
        crate::facade::interaction::UiDismissInteractionCause::Escape => {
            Some(crate::runtime::portal::UiPortalDismissalTrigger::Escape { semantic_surface })
        }
        crate::facade::interaction::UiDismissInteractionCause::OutsidePress(position) => {
            let basis = position.basis();
            if basis.coordinate_space()
                != worth_ui_host_contract::UiHostSurfaceCoordinateSpace::Viewport
                || basis.coordinate_unit()
                    != worth_ui_host_contract::UiHostSurfaceCoordinateUnit::LogicalPoint
            {
                return None;
            }
            let scale = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
            let point = [
                (position.x_subpixels() as f64 / scale) as f32,
                (position.y_subpixels() as f64 / scale) as f32,
            ];
            Some(
                crate::runtime::portal::UiPortalDismissalTrigger::OutsidePress {
                    semantic_surface,
                    viewport_point_bits: point.map(f32::to_bits),
                },
            )
        }
    }
}

pub(in crate::facade::entry) fn finish_detached_portal_proposal<'session>(
    session: &'session mut WorthUiActiveApplicationSession,
    proposal: crate::runtime::session::UiStagedPortalProposalTransaction,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiPortalDismissalPublicationOutcome<'session> {
    completion::finish(
        UiPortalDismissalAdmitted {
            session,
            proposal: Some(proposal),
        },
        outcome,
    )
}
