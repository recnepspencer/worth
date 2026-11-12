//! Owned ordinary evidence and borrowed original evidence share the same selector.
use super::super::patch_authority::{
    authoritative_patch_binding_evidence, borrowed_authoritative_patch_binding_evidence,
};
use super::*;
use crate::schema::data::{AspectBinding, LoweredAspectContractBinding};
use worth_foundational::facade::{
    aspects, validate_aspect_value, AspectContractRevision, AspectIdentity, FieldKey,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;
#[test]
fn borrowed_patch_selection_keeps_actual_value_and_ordinary_meaning() {
    let key = AspectKey::new("status").unwrap();
    let contract = aspects()
        .contract()
        .for_key(key.clone())
        .identified_by(AspectIdentity(1))
        .at_revision(AspectContractRevision(1))
        .scalar(ScalarAspectType::String);
    let binding = LoweredAspectContractBinding {
        contract: contract.clone(),
        target: AspectBinding::EntityField {
            field: FieldKey::new("status").unwrap(),
        },
    };
    let TransitionOutcome::Success(value) =
        validate_aspect_value(&contract, AspectValue::String("active".into()).into())
    else {
        panic!("valid value");
    };
    let TransitionOutcome::Success(patch) =
        worth_foundational::facade::AuthoritativeRecordAspectPatch::whole_aspect([value], [])
    else {
        panic!("valid patch");
    };
    let borrowed = borrowed_authoritative_patch_binding_evidence(
        &binding,
        RecordStructuralChange::Created,
        &patch,
    )
    .unwrap();
    let BorrowedAspectDeltaEvidence::AuthoritativePatch {
        operation: BorrowedPatchOperation::WholeAspectSet { value },
    } = borrowed
    else {
        panic!("set evidence");
    };
    assert!(std::ptr::eq(
        value,
        patch.whole_aspect_sets().next().unwrap().1
    ));
    assert_eq!(
        borrowed.into_owned(&key),
        authoritative_patch_binding_evidence(&binding, RecordStructuralChange::Created, &patch)
            .unwrap()
    );
}
#[test]
fn borrowed_value_transition_preserves_presence_and_same_payload_references() {
    let key = AspectKey::new("status").unwrap();
    let old = AspectValue::String("off".into());
    let new = AspectValue::String("active".into());
    let borrowed = BorrowedAspectDeltaEvidence::ScalarAspectValueTransition {
        old_present: true,
        new_present: true,
        old_value: Some(&old),
        new_value: Some(&new),
    };
    let owned = borrowed.into_owned(&key);
    let BorrowedAspectDeltaEvidence::ScalarAspectValueTransition {
        old_value,
        new_value,
        ..
    } = owned.borrowed()
    else {
        panic!("scalar evidence");
    };
    let Owned::ScalarAspectValueTransition {
        old_value: actual_old,
        new_value: actual_new,
        ..
    } = &owned
    else {
        panic!("scalar evidence");
    };
    assert!(std::ptr::eq(
        old_value.unwrap(),
        actual_old.as_ref().unwrap()
    ));
    assert!(std::ptr::eq(
        new_value.unwrap(),
        actual_new.as_ref().unwrap()
    ));
    assert_eq!(old_value, Some(&old));
    assert_eq!(new_value, Some(&new));
}
