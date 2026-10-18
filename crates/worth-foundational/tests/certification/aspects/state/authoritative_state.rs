use worth_foundational::{
    admit_authoritative_record_aspect_state, AspectValue, AuthoritativeStateAdmissionDenial,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::{key, validated_scalar};

#[test]
fn authoritative_state_admission_consumes_validated_entries_and_canonicalizes_order() {
    let beta = validated_scalar("beta", 2, ScalarAspectType::Int64, AspectValue::Int64(2));
    let alpha = validated_scalar("alpha", 1, ScalarAspectType::Int64, AspectValue::Int64(1));

    let outcome = admit_authoritative_record_aspect_state([beta, alpha]);
    let TransitionOutcome::Success(artifact) = outcome else {
        panic!("expected admitted authoritative state");
    };

    let keys: Vec<_> = artifact
        .payload()
        .aspects()
        .entries()
        .map(|(key, _)| key.as_str())
        .collect();

    assert_eq!(keys, vec!["alpha", "beta"]);
}

#[test]
fn authoritative_state_admission_rejects_duplicate_aspect_keys() {
    let first = validated_scalar("count", 1, ScalarAspectType::Int64, AspectValue::Int64(1));
    let second = validated_scalar("count", 1, ScalarAspectType::Int64, AspectValue::Int64(2));

    let outcome = admit_authoritative_record_aspect_state([first, second]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(AuthoritativeStateAdmissionDenial::DuplicateAspectKey(key(
            "count"
        )))
    );
}

#[test]
fn authoritative_state_admission_rejects_empty_validated_entry_sets() {
    let outcome = admit_authoritative_record_aspect_state([]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(AuthoritativeStateAdmissionDenial::EmptyAdmission)
    );
}
