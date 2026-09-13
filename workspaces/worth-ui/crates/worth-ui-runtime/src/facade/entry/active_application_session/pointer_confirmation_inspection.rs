use worth_ui_host_contract::UiSemanticSurfaceIdentity;
use worth_ui_inspection::{
    UiPointerAffordanceInspectionConfirmationStop as Stop,
    UiPointerAffordanceInspectionDecision as Decision,
    UiPointerAffordanceInspectionOutcome as Outcome,
};

pub(super) fn assert_confirmation(
    session: &crate::facade::WorthUiActiveApplicationSession,
    surface: UiSemanticSurfaceIdentity,
    deadline: Option<u64>,
) {
    let presentation = session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let position = super::target_position(session, surface, super::TargetRoute::Confirmation);
    let target = crate::runtime::interaction::targeting::resolve_presented_target(
        &session.mounted,
        presentation,
        position,
        &mut Default::default(),
    )
    .unwrap()
    .view();
    let before = session.intent_confirmation_metrics();
    let Outcome::Found(explanation) =
        session.why_pointer_affordance(session.appearance_inspection_world(surface), target)
    else {
        panic!("sealed confirmation must have an inspection explanation")
    };
    let Decision::Confirmation {
        eligible,
        stop,
        expiry_wake_millis,
        slots_inspected,
        ..
    } = explanation.decision
    else {
        panic!("confirmation must retain its own decision family")
    };
    assert_eq!(eligible, deadline.is_some());
    assert_eq!(expiry_wake_millis, deadline);
    assert_eq!(
        stop,
        if deadline.is_some() {
            None
        } else {
            Some(Stop::Expired)
        }
    );
    assert_eq!(
        slots_inspected,
        crate::facade::intent::UI_PENDING_INTENT_CONFIRMATION_LIMIT
    );
    assert_eq!(session.intent_confirmation_metrics(), before);
}
