//! Recorded-data validation; these fixtures mint no Home or runtime authority.
use super::*;
use crate::durability::{CompositeAttemptAssociation, HomeIdentity};

fn observed_owner(byte: u8) -> DurableWorldOwnerIdentity {
    DurableWorldOwnerIdentity::from_recorded(
        HomeIdentity::from_observed_bytes([byte; 16]).unwrap(),
        WorldOwnerSlotIdentity::from_recorded(0),
    )
}

#[test]
fn attempt_association_rejects_branch_or_incarnation_from_another_home_owner() {
    let owner = observed_owner(1);
    let foreign = observed_owner(2);
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(owner, 7);
    for (branch_owner, incarnation_owner) in [(foreign, owner), (owner, foreign)] {
        let branch =
            DurableProductBranchIdentity::from_recorded(branch_owner, "Main".into()).unwrap();
        let incarnation = DurableProductBranchIncarnation::from_recorded(incarnation_owner, 11);
        assert!(CompositeAttemptAssociation::from_recorded(attempt, branch, incarnation).is_err());
    }
}

#[test]
fn recorded_branch_spelling_and_independent_attempt_and_incarnation_are_preserved() {
    let owner = observed_owner(3);
    let spelling = "  Main/Δ  ";
    let branch = DurableProductBranchIdentity::from_recorded(owner, spelling.into()).unwrap();
    let attempt = DurableCompositePublicationAttemptIdentity::from_recorded(owner, 17);
    let incarnation = DurableProductBranchIncarnation::from_recorded(owner, 29);
    let association =
        CompositeAttemptAssociation::from_recorded(attempt, branch, incarnation).unwrap();
    assert_eq!(association.branch().name(), spelling);
    assert_eq!(association.attempt(), attempt);
    assert_eq!(association.incarnation(), incarnation);
}
