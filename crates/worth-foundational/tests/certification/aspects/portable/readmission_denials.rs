use worth_foundational::{
    readmit_portable_record_aspect_patch, AspectContract, AspectValue, CanonicalTime,
    ContractValidationDenial, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectFieldSet, PortableAspectPatchOperation, PortableAspectReadmissionDenial,
    PortablePatchReadmissionPurpose, PortableRecordAspectPatch, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use crate::certification::aspects::patches::patch_fixtures::task_summary_contract;
use crate::certification::foundational_vocabulary::{field, identity, key, revision};

#[test]
fn foreign_identity_and_missing_contract_deny_before_value_validation() {
    let contract = task_summary_contract();
    let foreign = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::new(contract.key().clone(), identity(99), revision(1)),
        value: ContractValidationInput::Scalar(AspectValue::Bool(true)),
    }]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            foreign,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::ContractIdentityMismatch { .. })
    ));

    let missing = PortableRecordAspectPatch::new([PortableAspectPatchOperation::ClearWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
    }]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            missing,
            PortablePatchReadmissionPurpose::RecordDeletion,
            &missing_lookup,
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::MissingContract(_))
    ));
}

#[test]
fn clear_policy_distinguishes_creation_mutation_and_record_deletion() {
    let contract = task_summary_contract();
    let candidate = || {
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::ClearWhole {
            basis: PortableAspectContractBasis::from_contract(&contract),
        }])
    };

    for purpose in [
        PortablePatchReadmissionPurpose::RecordCreation,
        PortablePatchReadmissionPurpose::RecordMutation,
    ] {
        assert!(matches!(
            readmit_portable_record_aspect_patch(candidate(), purpose, &lookup(&contract)),
            TransitionOutcome::Denied(PortableAspectReadmissionDenial::WholeClearDenied { .. })
        ));
    }

    let TransitionOutcome::Success(deletion) = readmit_portable_record_aspect_patch(
        candidate(),
        PortablePatchReadmissionPurpose::RecordDeletion,
        &lookup(&contract),
    ) else {
        panic!("record deletion may clear required aspects");
    };
    assert_eq!(deletion.whole_aspect_clears().count(), 1);
}

#[test]
fn malformed_value_and_illegal_field_mask_deny_at_foundational_admission() {
    let contract = task_summary_contract();
    let wrong_shape = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::Bool(true)),
    }]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            wrong_shape,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::ValueValidation { .. })
    ));

    let illegal_mask =
        PortableRecordAspectPatch::new([PortableAspectPatchOperation::PatchFields {
            basis: PortableAspectContractBasis::from_contract(&contract),
            selected_fields: vec![field("title")],
            field_sets: vec![PortableAspectFieldSet::new(
                field("done"),
                AspectValue::Bool(true),
            )],
            field_clears: Vec::new(),
        }]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            illegal_mask,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::PatchConstruction(_))
    ));
}

#[test]
fn deserialized_noncanonical_scalar_wrapper_denies_instead_of_minting_proof() {
    let contract = AspectContract::scalar(
        key("deadline.time"),
        identity(30),
        revision(1),
        ScalarAspectType::Time,
    );
    let candidate = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(&contract),
        value: ContractValidationInput::Scalar(AspectValue::Time(CanonicalTime {
            nanos_since_midnight: CanonicalTime::NANOS_PER_DAY,
        })),
    }]);
    let bytes = serde_json::to_vec(&candidate).expect("candidate serializes");
    let transported = serde_json::from_slice(&bytes).expect("candidate deserializes");

    assert!(matches!(
        readmit_portable_record_aspect_patch(
            transported,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::ValueValidation {
            denial: ContractValidationDenial::NonCanonicalScalarValue(ScalarAspectType::Time),
            ..
        })
    ));
}

fn lookup(
    contract: &worth_foundational::AspectContract,
) -> impl Fn(&worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> + '_ {
    move |key| (key == contract.key()).then(|| contract.clone())
}

fn missing_lookup(_: &worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> {
    None
}
