use worth_foundational::aspect_patch_digest_preparation_basis;

use {crate::certification::ordinary_context::canonicalization::digest_preparation::readiness_fixtures::ready_patch};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::validated_scalar};

#[test]
fn patch_digest_preparation_basis_is_ordered_independent_of_construction_path() {
    let count = validated_scalar(
        "count",
        1,
        worth_foundational::ScalarAspectType::Int64,
        worth_foundational::AspectValue::Int64(2),
    );
    let name = validated_scalar(
        "name",
        2,
        worth_foundational::ScalarAspectType::String,
        worth_foundational::AspectValue::String("Ada".into()),
    );
    let worth_proof::TransitionOutcome::Success(left_patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect(
            [count.clone(), name.clone()],
            [],
        )
    else {
        panic!("expected left patch");
    };
    let worth_proof::TransitionOutcome::Success(right_patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([name, count], [])
    else {
        panic!("expected right patch");
    };
    let left_ready = ready_patch(&left_patch);
    let right_ready = ready_patch(&right_patch);

    assert_eq!(
        aspect_patch_digest_preparation_basis(&left_ready),
        aspect_patch_digest_preparation_basis(&right_ready)
    );
}
