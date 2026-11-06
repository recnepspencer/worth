//! Supplemental Home reservations advance issuance, never recovered source facts.
use super::*;
use worth_foundational::facade::{
    DurableWorldOwnerIdentity, HomeIdentity, HomeIncarnation, WorldOwnerSlotIdentity,
};

fn issuer(source: u64, reserved: u64) -> RuntimeWorldIdentityIssuer {
    let owner = DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([1; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(0),
    );
    let binding =
        WorldDurableHomeBinding::from_admitted_adapter(owner, HomeIncarnation::initial(), [2; 32])
            .with_home_transition_attempt_frontier(Some(reserved));
    crate::lifecycle::owner::RuntimeWorldOwnerConstructionContract::for_home(
        &binding,
        WorldIdentityFrontiers {
            attempt: source,
            ..Default::default()
        },
    )
    .unwrap()
    .into_issuer()
}

#[test]
fn home_intent_reservations_and_world_records_both_constrain_restored_issuance() {
    for (source, reserved) in [(3, 9), (12, 9)] {
        let mut owner = issuer(source, reserved);
        assert_eq!(
            owner.publication_attempt().unwrap().ordinal(),
            source.max(reserved)
        );
        assert_eq!(
            owner.publication_attempt().unwrap().ordinal(),
            source.max(reserved) + 1
        );
    }
}

#[test]
fn an_exhausted_home_intent_frontier_refuses_instead_of_reusing_an_attempt() {
    let mut owner = issuer(3, u64::MAX);
    for _ in 0..2 {
        assert_eq!(
            owner.publication_attempt().unwrap_err().family(),
            crate::identity::RuntimeWorldIdentityFamily::PublicationAttempt
        );
    }
}
