//! A duplicate rail acknowledgement cannot advance native publication twice.
use super::*;
#[test]
fn a_duplicate_acknowledgement_never_advances_the_posture_twice() {
    let world = dispatch_world("duplicate-acknowledgement");
    world
        .transport
        .under(FaultScript::DuplicateAcknowledgement, PATIENT);
    let binding = idempotency(45);
    let receipt = world.commit_with(binding.clone());
    let posture = receipt
        .external_dispatch_posture()
        .expect("the declared effect must be dispatched");
    assert_eq!(
        posture.failure(),
        Some(WorthQueryPublishedExternalEffectFailure::DuplicatedAcknowledgement)
    );

    let (recovered, fresh) = world.notify(binding);
    assert!(
        !fresh,
        "native recovered evidence, no original handler result"
    );
    publication_assertions::assert_recovered_commit_axes(&receipt, &recovered);
    assert!(
        recovered.external_dispatch_posture().is_none(),
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
