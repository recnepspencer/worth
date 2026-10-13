use super::*;
#[test]
fn post_effect_rebase_door_preserves_stops_at_every_consult() {
    let world = installed_authorization_world(true);
    world.application.with_host_advancement(|phase| {
        let (entity, locator) = account_note(&phase, &world, "account-1");
        let kind = world.application.runtime.primary_graph().unwrap().layout.entity_kind(AccountIdentity::reference().entity()).unwrap();
        let fact = WorthQueryApplicationObservedFact::AbsentField { entity_id: entity, kind, locator: planned(&locator) };
        crate::test_read_stops::every_consult_under(&phase.borrowed_request_for_test(),|execution| match rebase_result_within(&phase, execution, &world, vec![fact.clone()], 64) {
            RebasedSourceFacts::Exact(_) => Ok(true),
            RebasedSourceFacts::VerificationRequired { reason: RebaseVerificationReason::Execution(cause), .. } => Err(cause),
            other => panic!("a native stop must keep its cause: {other:?}"),
        }, worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled, worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted);
    }).unwrap();
}
