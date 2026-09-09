use super::{
    UiIntentConsequenceAdmitted, UiIntentConsequencePublicationOutcome,
    WorthUiPreparedIntentConsequenceRebind,
};
use crate::facade::entry::{
    intent_consequence_rebind::WorthUiIntentConsequenceRebindTransfer,
    WorthUiActiveApplicationSession,
};

pub(super) fn stop_prepared<'session>(
    prepared: WorthUiPreparedIntentConsequenceRebind<'session>,
    reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
) -> UiIntentConsequencePublicationOutcome<'session> {
    let WorthUiPreparedIntentConsequenceRebind {
        session,
        plan,
        reservation,
        frame,
        transfer,
    } = prepared;
    drop((reservation, frame));
    retain_stop(session, plan, transfer, reason)
}

pub(super) fn stop_admitted<'session>(
    mut admitted: UiIntentConsequenceAdmitted<'session>,
    reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
) -> UiIntentConsequencePublicationOutcome<'session> {
    withdraw_query(&mut admitted);
    retain_stop(admitted.session, admitted.plan, admitted.transfer, reason)
}

pub(super) fn withdraw_query(admitted: &mut UiIntentConsequenceAdmitted<'_>) {
    if let Some(query) = admitted.query.take() {
        drop(
            admitted
                .session
                .application
                .withdraw_exact_query_change(query)
                .expect("exclusive exact Query admission must remain withdrawable"),
        );
    }
}

fn retain_stop<'session>(
    session: &'session mut WorthUiActiveApplicationSession,
    plan: crate::runtime::rebind::UiRebindPlan,
    mut transfer: WorthUiIntentConsequenceRebindTransfer,
    reason: crate::runtime::intent_execution::UiIntentConsequenceStopReason,
) -> UiIntentConsequencePublicationOutcome<'session> {
    if let Some(proposal) = transfer.portal_proposal.take() {
        session.application.cancel_portal_service_proposal(
            proposal,
            session
                .focus
                .as_mut()
                .expect("staged proposal retains Focus installation"),
            session
                .motion
                .as_mut()
                .expect("staged proposal retains Motion installation"),
        );
    }
    transfer
        .consequence
        .restore_query_from_facts(plan.into_retained_facts());
    UiIntentConsequencePublicationOutcome::Stopped(
        session
            .intent_execution
            .retain_consequence_handoff(transfer.consequence, reason),
    )
}
