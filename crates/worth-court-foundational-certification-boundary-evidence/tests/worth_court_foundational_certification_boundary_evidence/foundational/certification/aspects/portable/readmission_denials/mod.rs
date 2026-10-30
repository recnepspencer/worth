#[allow(unused_imports)]
use crate::certification::ordinary_context::aspects::portable::readmission_denials::*;
use {worth_foundational::readmit_portable_record_aspect_patch, worth_foundational::CanonicalTime};

use crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract;
use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn foreign_identity_and_missing_contract_deny_before_value_validation() {
    let contract = task_summary_contract();
    let foreign = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::SetWhole {
            basis: worth_foundational::PortableAspectContractBasis::new(
                contract.key().clone(),
                identity(99),
                revision(1),
            ),
            value: worth_foundational::ContractValidationInput::Scalar(
                worth_foundational::AspectValue::Bool(true),
            ),
        },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            foreign,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::ContractIdentityMismatch { .. }
        )
    ));

    let missing = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::ClearWhole {
            basis: worth_foundational::PortableAspectContractBasis::from_contract(&contract),
        },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            missing,
            worth_foundational::PortablePatchReadmissionPurpose::RecordDeletion,
            &missing_lookup,
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::MissingContract(_)
        )
    ));
}

#[test]
fn clear_policy_distinguishes_creation_mutation_and_record_deletion() {
    let contract = task_summary_contract();
    let candidate = || {
        worth_foundational::PortableRecordAspectPatch::new([
            worth_foundational::PortableAspectPatchOperation::ClearWhole {
                basis: worth_foundational::PortableAspectContractBasis::from_contract(&contract),
            },
        ])
    };

    for purpose in [
        worth_foundational::PortablePatchReadmissionPurpose::RecordCreation,
        worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
    ] {
        assert!(matches!(
            readmit_portable_record_aspect_patch(candidate(), purpose, &lookup(&contract)),
            worth_proof::TransitionOutcome::Denied(
                worth_foundational::PortableAspectReadmissionDenial::WholeClearDenied { .. }
            )
        ));
    }

    let worth_proof::TransitionOutcome::Success(deletion) = readmit_portable_record_aspect_patch(
        candidate(),
        worth_foundational::PortablePatchReadmissionPurpose::RecordDeletion,
        &lookup(&contract),
    ) else {
        panic!("record deletion may clear required aspects");
    };
    assert_eq!(deletion.whole_aspect_clears().count(), 1);
}

#[test]
fn malformed_value_and_illegal_field_mask_deny_at_foundational_admission() {
    let contract = task_summary_contract();
    let wrong_shape = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::SetWhole {
            basis: worth_foundational::PortableAspectContractBasis::from_contract(&contract),
            value: worth_foundational::ContractValidationInput::Scalar(
                worth_foundational::AspectValue::Bool(true),
            ),
        },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            wrong_shape,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::ValueValidation { .. }
        )
    ));

    let illegal_mask = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::PatchFields {
            basis: worth_foundational::PortableAspectContractBasis::from_contract(&contract),
            selected_fields: vec![field("title")],
            field_sets: vec![worth_foundational::PortableAspectFieldSet::new(
                field("done"),
                worth_foundational::AspectValue::Bool(true),
            )],
            field_clears: Vec::new(),
        },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            illegal_mask,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::PatchConstruction(_)
        )
    ));
}

#[test]
fn deserialized_noncanonical_scalar_wrapper_denies_instead_of_minting_proof() {
    let contract = worth_foundational::AspectContract::scalar(
        key("deadline.time"),
        identity(30),
        revision(1),
        worth_foundational::ScalarAspectType::Time,
    );
    let candidate = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::SetWhole {
            basis: worth_foundational::PortableAspectContractBasis::from_contract(&contract),
            value: worth_foundational::ContractValidationInput::Scalar(
                worth_foundational::AspectValue::Time(CanonicalTime {
                    nanos_since_midnight: worth_foundational::CanonicalTime::NANOS_PER_DAY,
                }),
            ),
        },
    ]);
    let bytes = serde_json::to_vec(&candidate).expect("candidate serializes");
    let transported = serde_json::from_slice(&bytes).expect("candidate deserializes");

    assert!(matches!(
        readmit_portable_record_aspect_patch(
            transported,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::ValueValidation {
                denial: worth_foundational::ContractValidationDenial::NonCanonicalScalarValue(
                    worth_foundational::ScalarAspectType::Time
                ),
                ..
            }
        )
    ));
}
