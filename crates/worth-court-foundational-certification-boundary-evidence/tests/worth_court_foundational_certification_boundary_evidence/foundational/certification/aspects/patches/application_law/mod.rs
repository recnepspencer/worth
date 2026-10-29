use {worth_foundational::AspectContract, worth_foundational::AspectValue, worth_foundational::AuthoritativePatchConstructionDenial, worth_foundational::AuthoritativeRecordAspectPatch, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::aspects::patches::patch_fixtures::admitted_state};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::validated_scalar};

#[test]
fn whole_aspect_patch_set_dominates_overlapping_clear_and_applies_canonically() {
    let count_one = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(1));
    let count_two = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(2));
    let state = admitted_state([count_one]);
    let count_contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    let worth_proof::TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([count_two], [count_contract])
    else {
        panic!("expected whole-aspect patch construction to succeed");
    };

    assert!(patch.whole_aspect_clears().next().is_none());

    let worth_proof::TransitionOutcome::Success(next_state) = patch.apply_to(state.payload()) else {
        panic!("expected patch application to succeed");
    };

    let count = next_state
        .payload()
        .get(&key("count"))
        .expect("count exists");
    assert_eq!(count.key().as_str(), "count");
}

#[test]
fn empty_patch_is_a_canonical_no_op() {
    let count = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(1));
    let state = admitted_state([count]);
    let patch = worth_foundational::AuthoritativeRecordAspectPatch::empty();

    let worth_proof::TransitionOutcome::Success(next_state) = patch.apply_to(state.payload()) else {
        panic!("expected no-op patch application to succeed");
    };

    assert!(patch.is_empty());
    assert_eq!(state.payload(), next_state.payload());
}

#[test]
fn whole_aspect_patch_rejects_duplicate_sets() {
    let first = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(1));
    let second = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(2));

    let outcome = worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([first, second], []);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::DuplicateWholeAspectSet(key("count"))
        )
    );
}

#[test]
fn whole_aspect_patch_rejects_duplicate_and_foreign_basis_clears() {
    let clear_contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    assert_eq!(
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([], [clear_contract.clone(), clear_contract]),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::DuplicateWholeAspectClear(key("count"))
        )
    );

    let set = validated_scalar("count", 1, worth_foundational::ScalarAspectType::Int64, worth_foundational::AspectValue::Int64(1));
    let foreign_clear = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(2),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    assert_eq!(
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([set], [foreign_clear]),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::AuthoritativePatchConstructionDenial::OverlappingWholeAspectContractMismatch(key(
                "count"
            ))
        )
    );
}
