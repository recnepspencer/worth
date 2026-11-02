pub(crate) use worth_query_bank_testkit_server::facade::estate_operations::external_effect_dispatch::duplicate_acknowledgement::*;

#[test]
fn a_duplicate_acknowledgement_never_advances_the_posture_twice() {
    let world = dispatch_world("duplicate-acknowledgement");
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, PATIENT);
    let binding = idempotency(45);
    let receipt_closed = world.commit_with(binding.clone());
    let receipt = receipt_closed
        .live_receipt()
        .expect("fresh source retains its full native owner");
    let posture = receipt
        .external_dispatch_posture()
        .expect("the declared effect must be dispatched");
    assert_eq!(
        posture.failure(),
        Some(WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement)
    );

    let recovered = world.notify(binding);
    assert!(
        recovered.recorded_receipt().is_some(),
        "native recovered evidence, no original handler result"
    );
    publication_assertions::assert_recovered_commit_axes(&receipt_closed, &recovered);
    assert!(
        recovered.live_receipt().is_none(),
        "a recovered commit re-runs no dispatch and invents no posture"
    );
    assert_eq!(
        world.transport.attempts().len(),
        1,
        "the second acknowledgement must not become a second dispatch"
    );
    assert_eq!(
        world.transport.ledger_status(&rail_correlation(&world)),
        LedgerStatus::Acknowledged,
        "the rail never completed this attempt"
    );
}
