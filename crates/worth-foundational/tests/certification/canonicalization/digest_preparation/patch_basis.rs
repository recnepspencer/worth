use worth_foundational::{
    aspect_patch_digest_preparation_basis, AspectValue, AuthoritativeRecordAspectPatch,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use super::readiness_fixtures::ready_patch;
use crate::certification::foundational_vocabulary::validated_scalar;

#[test]
fn patch_digest_preparation_basis_is_ordered_independent_of_construction_path() {
    let count = validated_scalar("count", 1, ScalarAspectType::Int64, AspectValue::Int64(2));
    let name = validated_scalar(
        "name",
        2,
        ScalarAspectType::String,
        AspectValue::String("Ada".into()),
    );
    let TransitionOutcome::Success(left_patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([count.clone(), name.clone()], [])
    else {
        panic!("expected left patch");
    };
    let TransitionOutcome::Success(right_patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([name, count], [])
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
