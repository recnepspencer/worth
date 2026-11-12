//! These are recorded-value checks, not runtime or source-authority fixtures.
use super::*;
use crate::facade::*;
use std::num::NonZeroU64;

fn plan(home: u8, ordinal: u64, digest: u8, steps: u32) -> HomeProgramTransitionPlanDescription {
    let identity = HomeIdentity::from_observed_bytes([home; 16]).unwrap();
    let owner = DurableWorldOwnerIdentity::from_recorded(
        identity,
        WorldOwnerSlotIdentity::from_recorded(1),
    );
    let source = DurableProductBranchReference::from_recorded(
        DurableProductBranchIdentity::from_recorded(owner, "main".to_owned()).unwrap(),
        DurableProductBranchIncarnation::from_recorded(owner, 1),
        DurableProductReferenceGeneration::from_recorded(2),
        DurableCompositeCommitIdentity::from_recorded(owner, 3),
    )
    .unwrap();
    HomeProgramTransitionPlanDescription::from_recorded(
        HomeProgramTransitionIdentity::from_recorded(identity, NonZeroU64::new(ordinal).unwrap()),
        source,
        HomeProgramTransitionPlanDigest::from_recorded([digest; 32]),
        steps,
        HomeProgramTransitionTargetMeaning::from_recorded(
            0,
            HomeProgramTransitionCatalogDigest::from_recorded([9; 32]),
            HomeProgramTransitionSchemaDigest::from_recorded([11; 32]),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([13; 32]),
        ),
    )
    .unwrap()
}

#[test]
fn plan_commitment_binds_the_home_intent_digest_and_complete_step_count() {
    let original = plan(1, 1, 7, 3);
    for changed in [
        plan(2, 1, 7, 3),
        plan(1, 2, 7, 3),
        plan(1, 1, 8, 3),
        plan(1, 1, 7, 4),
    ] {
        assert_ne!(original.commitment(), changed.commitment());
    }
    let foreign = plan(2, 1, 7, 3);
    assert_eq!(
        HomeProgramTransitionPlanDescription::from_recorded(
            original.identity(),
            foreign.predecessor().clone(),
            original.digest(),
            original.steps(),
            original.target_meaning(),
        ),
        Err(HomeProgramTransitionDescriptionDenial::ForeignHome)
    );
}

#[test]
fn step_commitment_binds_the_exact_position_and_original_world_attempt() {
    let plan = plan(1, 1, 7, 3);
    let association = |ordinal| {
        CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(
                plan.predecessor().branch().owner(),
                ordinal,
            ),
            plan.predecessor().branch().clone(),
            plan.predecessor().incarnation(),
        )
        .unwrap()
    };
    let first = HomeProgramTransitionStepAssociation::from_recorded(plan.identity(), 0);
    let second = HomeProgramTransitionStepAssociation::from_recorded(plan.identity(), 1);
    assert_ne!(
        plan.step_commitment(first, &association(4)).unwrap(),
        plan.step_commitment(second, &association(4)).unwrap()
    );
    assert_ne!(
        plan.step_commitment(first, &association(4)).unwrap(),
        plan.step_commitment(first, &association(5)).unwrap()
    );
    let past_end = HomeProgramTransitionStepAssociation::from_recorded(plan.identity(), 3);
    assert_eq!(
        plan.step_commitment(past_end, &association(4)),
        Err(HomeProgramTransitionDescriptionDenial::StepOutOfRange)
    );
}

#[test]
fn target_meaning_and_its_catalog_are_bound_before_any_step_can_be_admitted() {
    let original = plan(1, 1, 7, 3);
    for target in [
        HomeProgramTransitionTargetMeaning::from_recorded(
            0,
            original.target_meaning().catalog(),
            original.target_meaning().schema(),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([14; 32]),
        ),
        HomeProgramTransitionTargetMeaning::from_recorded(
            0,
            original.target_meaning().catalog(),
            HomeProgramTransitionSchemaDigest::from_recorded([12; 32]),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([13; 32]),
        ),
        HomeProgramTransitionTargetMeaning::from_recorded(
            1,
            original.target_meaning().catalog(),
            original.target_meaning().schema(),
            original.target_meaning().carrier(),
        ),
        HomeProgramTransitionTargetMeaning::from_recorded(
            0,
            HomeProgramTransitionCatalogDigest::from_recorded([10; 32]),
            HomeProgramTransitionSchemaDigest::from_recorded([11; 32]),
            HomeProgramTransitionSchemaCarrierDigest::from_recorded([13; 32]),
        ),
    ] {
        let changed = HomeProgramTransitionPlanDescription::from_recorded(
            original.identity(),
            original.predecessor().clone(),
            original.digest(),
            original.steps(),
            target,
        )
        .unwrap();
        assert_ne!(original.commitment(), changed.commitment());
    }
    assert_eq!(
        HomeProgramTransitionPlanDescription::from_recorded(
            original.identity(),
            original.predecessor().clone(),
            original.digest(),
            original.steps(),
            HomeProgramTransitionTargetMeaning::from_recorded(
                original.steps(),
                original.target_meaning().catalog(),
                original.target_meaning().schema(),
                original.target_meaning().carrier(),
            ),
        ),
        Err(HomeProgramTransitionDescriptionDenial::StepOutOfRange),
    );
}
