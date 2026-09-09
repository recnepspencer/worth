use super::{
    intent_consequence_rebind::WorthUiIntentConsequenceRebindTransfer,
    WorthUiActiveApplicationSession,
};

#[path = "intent_consequence_publication/managed.rs"]
mod managed;
#[path = "intent_consequence_publication/recovery.rs"]
mod recovery;
pub(in crate::facade::entry) use managed::{
    DetachedUiIntentConsequenceInFlight, DetachedUiIntentConsequenceIndeterminate,
    DetachedUiIntentConsequenceRecoveryResources,
};
#[path = "intent_consequence_publication/deadline.rs"]
mod deadline;
use deadline::presentation_deadline;
#[path = "intent_consequence_publication/portal_settlement.rs"]
mod portal_settlement;
#[path = "intent_consequence_publication/receipt.rs"]
mod receipt;
#[path = "intent_consequence_publication/stop.rs"]
mod stop;
use portal_settlement::{
    settle_indeterminate_portal_proposal, settle_published_portal_proposal,
    settle_rejected_portal_proposal,
};
pub use receipt::UiIntentConsequencePublicationReceipt;
use stop::{stop_admitted, stop_prepared, withdraw_query};

pub enum UiIntentConsequencePublicationOutcome<'session> {
    NoConsequences(crate::runtime::intent_execution::UiIntentConsequenceCompletionReceipt),
    Stopped(crate::runtime::intent_execution::UiIntentConsequenceStop),
    Published(UiIntentConsequencePublicationReceipt),
    InFlight(UiIntentConsequencePublicationCompletion<'session>),
    Indeterminate(UiIntentConsequencePublicationRecovery<'session>),
    InternalDefect(crate::runtime::rebind::UiRebindInternalDefectOutcome),
}

#[must_use = "consequence presentation must be completed or explicitly disposed"]
pub struct UiIntentConsequencePublicationCompletion<'session> {
    state: Option<Box<UiIntentConsequenceInFlight<'session>>>,
}

#[must_use = "indeterminate consequence presentation requires reconciliation or shutdown"]
pub struct UiIntentConsequencePublicationRecovery<'session> {
    state: Option<Box<UiIntentConsequenceIndeterminate<'session>>>,
}

pub(super) struct WorthUiPreparedIntentConsequenceRebind<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: crate::runtime::rebind::UiRebindReservation,
    frame: crate::mounting::UiPreparedMountedFrame,
    transfer: WorthUiIntentConsequenceRebindTransfer,
}

struct UiIntentConsequenceAdmitted<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    reservation: crate::runtime::rebind::UiRebindReservation,
    transfer: WorthUiIntentConsequenceRebindTransfer,
    query: Option<worth_ui_query_binding::WorthUiAdmittedCollectionChangePublication>,
}

struct UiIntentConsequenceInFlight<'session> {
    admitted: UiIntentConsequenceAdmitted<'session>,
    mounted: crate::mounting::UiMountedPresentationInFlight,
}

struct UiIntentConsequenceIndeterminate<'session> {
    admitted: UiIntentConsequenceAdmitted<'session>,
    frame: crate::mounting::UiMountedIndeterminateFrame,
    portal: Option<crate::runtime::session::UiIndeterminatePortalProposalTransaction>,
}

impl<'session> WorthUiPreparedIntentConsequenceRebind<'session> {
    pub(super) fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        plan: crate::runtime::rebind::UiRebindPlan,
        reservation: crate::runtime::rebind::UiRebindReservation,
        frame: crate::mounting::UiPreparedMountedFrame,
        transfer: WorthUiIntentConsequenceRebindTransfer,
    ) -> Self {
        Self {
            session,
            plan,
            reservation,
            frame,
            transfer,
        }
    }

    pub(super) fn execute(
        mut self,
        now_tick: u64,
    ) -> UiIntentConsequencePublicationOutcome<'session> {
        if let Err(denial) = self.reservation.begin_effecting() {
            return stop_prepared(
                self,
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::RebindAdmission(
                    denial,
                ),
            );
        }
        let query = match self.transfer.query_reference.as_ref() {
            Some(reference) => match self
                .session
                .application
                .prepare_exact_query_change_publication(reference)
            {
                Ok(admission) => Some(admission),
                Err(reason) => return stop_prepared(self, reason),
            },
            None => None,
        };
        let deadline = presentation_deadline(&self.plan);
        let Self {
            session,
            plan,
            reservation,
            frame,
            transfer,
        } = self;
        let outcome = match transfer.portal_proposal.as_ref() {
            Some(proposal) => session.present_prepared_portal_frame_internal(
                frame,
                proposal,
                proposal.overlay_appearance_sources().0.closes_portal(),
                deadline,
                now_tick,
            ),
            None => session.present_prepared_mounted_frame_internal(frame, deadline, now_tick),
        };
        finish_first(
            UiIntentConsequenceAdmitted {
                session,
                plan,
                reservation,
                transfer,
                query,
            },
            outcome,
        )
    }
}

impl UiIntentConsequencePublicationCompletion<'_> {
    pub fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.state().mounted.attempt()
    }

    pub fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.state().mounted.deadline()
    }
}

impl<'session> UiIntentConsequencePublicationCompletion<'session> {
    pub fn complete(mut self, now_tick: u64) -> UiIntentConsequencePublicationOutcome<'session> {
        let state = self.take_state();
        let outcome = state
            .admitted
            .session
            .complete_mounted_presentation(state.mounted, now_tick);
        finish_completion(state.admitted, outcome)
    }

    pub fn dispose(mut self) -> UiIntentConsequencePublicationOutcome<'session> {
        let state = self.take_state();
        let outcome = state
            .admitted
            .session
            .cancel_mounted_presentation(state.mounted);
        finish_completion(state.admitted, outcome)
    }

    fn state(&self) -> &UiIntentConsequenceInFlight<'session> {
        self.state
            .as_deref()
            .expect("live consequence completion owns its state")
    }

    fn take_state(&mut self) -> Box<UiIntentConsequenceInFlight<'session>> {
        self.state
            .take()
            .expect("live consequence completion owns its state")
    }
}

impl Drop for UiIntentConsequencePublicationCompletion<'_> {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        let outcome = state
            .admitted
            .session
            .cancel_mounted_presentation(state.mounted);
        drop(finish_completion(state.admitted, outcome));
    }
}

fn finish_first<'session>(
    mut admitted: UiIntentConsequenceAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiIntentConsequencePublicationOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::InFlight(mounted) => {
            admitted
                .reservation
                .retain_completion()
                .expect("effect admission reserved consequence completion capacity");
            UiIntentConsequencePublicationOutcome::InFlight(
                UiIntentConsequencePublicationCompletion {
                    state: Some(Box::new(UiIntentConsequenceInFlight { admitted, mounted })),
                },
            )
        }
        outcome => finish_terminal(admitted, outcome),
    }
}

fn finish_completion<'session>(
    admitted: UiIntentConsequenceAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiIntentConsequencePublicationOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::InFlight(mounted) => {
            UiIntentConsequencePublicationOutcome::InFlight(
                UiIntentConsequencePublicationCompletion {
                    state: Some(Box::new(UiIntentConsequenceInFlight { admitted, mounted })),
                },
            )
        }
        outcome => finish_terminal(admitted, outcome),
    }
}

fn finish_terminal<'session>(
    mut admitted: UiIntentConsequenceAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiIntentConsequencePublicationOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(mounted) => publish(admitted, mounted),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            settle_rejected_portal_proposal(&mut admitted);
            stop_admitted(
                admitted,
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::HostRejectedBeforeEffects {
                    rejection_count: rejected.rejections().len(),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejected) => {
            settle_rejected_portal_proposal(&mut admitted);
            stop_admitted(
                admitted,
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::MountedRetention(
                    rejected.denial(),
                ),
            )
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejected) => {
            settle_rejected_portal_proposal(&mut admitted);
            stop_admitted(
                admitted,
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::MountedPresentation(
                    rejected.denial(),
                ),
            )
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            let portal = settle_indeterminate_portal_proposal(&mut admitted);
            admitted
                .reservation
                .retain_recovery()
                .expect("effect admission reserved consequence recovery capacity");
            UiIntentConsequencePublicationOutcome::Indeterminate(
                UiIntentConsequencePublicationRecovery {
                    state: Some(Box::new(UiIntentConsequenceIndeterminate {
                        admitted,
                        frame,
                        portal,
                    })),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            unreachable!("ordinary intent publication cannot overlap a superseding frame")
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("exact consequence completion authority became unknown")
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(_) => {
            unreachable!("in-flight outcomes are retained by the phase-specific mapper")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            unreachable!("explicit consequence content always presents a fresh frame")
        }
    }
}

fn publish<'session>(
    mut admitted: UiIntentConsequenceAdmitted<'session>,
    mounted: crate::mounting::UiMountedFramePublicationReceipt,
) -> UiIntentConsequencePublicationOutcome<'session> {
    if let Some(query) = admitted.query.take() {
        let receipt = admitted
            .session
            .application
            .publish_exact_query_change(query)
            .expect("exclusive exact Query admission must remain publishable");
        assert_eq!(receipt.published_change_count(), 1);
    }
    assert!(matches!(
        admitted.plan.take_semantic_proof(),
        crate::runtime::rebind::UiRebindSemanticProof::NonSource
    ));
    let focus = settle_published_portal_proposal(&mut admitted, &mounted);
    admitted
        .session
        .application
        .commit_prepared_observation_progress(admitted.transfer.observation);
    if let Some(posture) = admitted.transfer.posture.take() {
        admitted.session.intent_postures.commit(posture);
    }
    admitted
        .session
        .intent_execution
        .finish_consequence_handoff(admitted.transfer.consequence);
    let generation = admitted.plan.basis().candidate_generation().clone();
    match crate::runtime::rebind::UiRebindReceipt::content(
        admitted.plan,
        admitted.reservation,
        generation,
        mounted,
    ) {
        Ok(receipt) => UiIntentConsequencePublicationOutcome::Published(
            UiIntentConsequencePublicationReceipt::new(receipt, focus),
        ),
        Err(defect) => UiIntentConsequencePublicationOutcome::InternalDefect(defect),
    }
}
