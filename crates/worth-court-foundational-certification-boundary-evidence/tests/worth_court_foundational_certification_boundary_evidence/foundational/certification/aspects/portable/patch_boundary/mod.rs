#[allow(unused_imports)]
use crate::certification::ordinary_context::aspects::portable::patch_boundary::*;
use {
    worth_foundational::export_portable_record_aspect_patch,
    worth_foundational::readmit_portable_record_aspect_patch, worth_foundational::AspectMask,
    worth_foundational::MutationMask,
};

use {
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::validated_task_summary,
};
use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn serialized_whole_struct_patch_earns_fresh_authority_from_the_current_contract() {
    let contract = task_summary_contract();
    let worth_proof::TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect(
            [validated_task_summary(
                &contract,
                "ship",
                false,
                Some("carefully"),
            )],
            [],
        )
    else {
        panic!("expected authoritative patch");
    };

    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("current contract exports portable meaning");
    let bytes = serde_json::to_vec(&portable).expect("portable patch serializes");
    let transported = serde_json::from_slice(&bytes).expect("portable patch deserializes");
    let worth_proof::TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        transported,
        worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
        &lookup(&contract),
    ) else {
        panic!("transported candidate should readmit");
    };

    assert_eq!(readmitted, patch);
}

#[test]
fn field_set_and_clear_survive_without_becoming_a_field_map_authority() {
    let contract = task_summary_contract();
    let mask = AspectMask::<MutationMask>::new([
        worth_foundational::CanonicalFieldPath::single(field("title")),
        worth_foundational::CanonicalFieldPath::single(field("done")),
        worth_foundational::CanonicalFieldPath::single(field("note")),
    ]);
    let worth_proof::TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::field_level(
            &contract,
            &mask,
            [(field("done"), worth_foundational::AspectValue::Bool(true))],
            [field("note")],
        )
    else {
        panic!("expected authoritative field patch");
    };

    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("field patch exports");
    let worth_proof::TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        portable,
        worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
        &lookup(&contract),
    ) else {
        panic!("field patch should readmit");
    };

    assert_eq!(readmitted, patch);
    assert_eq!(readmitted.field_patches().next().unwrap().1.mask(), &mask);
}

#[test]
fn whole_clear_retains_its_exact_contract_basis_across_transport() {
    let contract = task_summary_contract();
    let worth_proof::TransitionOutcome::Success(patch) =
        worth_foundational::AuthoritativeRecordAspectPatch::whole_aspect([], [contract.clone()])
    else {
        panic!("expected exact clear patch");
    };
    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("exact clear exports");
    let worth_proof::TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        portable,
        worth_foundational::PortablePatchReadmissionPurpose::RecordDeletion,
        &lookup(&contract),
    ) else {
        panic!("exact clear should readmit for deletion");
    };

    assert_eq!(readmitted, patch);
}

#[test]
fn stale_contract_and_duplicate_operations_deny_before_authority_exists() {
    let contract = task_summary_contract();
    let stale_basis = worth_foundational::PortableAspectContractBasis::new(
        contract.key().clone(),
        identity(20),
        revision(0),
    );
    let value = worth_foundational::ContractValidationInput::Struct(
        worth_foundational::StructAspectValue::new([
            (
                field("title"),
                worth_foundational::AspectValue::String("ship".into()),
            ),
            (field("done"), worth_foundational::AspectValue::Bool(false)),
        ])
        .expect("unique fields"),
    );
    let stale = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::SetWhole {
            basis: stale_basis,
            value: value.clone(),
        },
    ]);

    assert!(matches!(
        readmit_portable_record_aspect_patch(
            stale,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::ContractRevisionMismatch { .. }
        )
    ));

    let basis = worth_foundational::PortableAspectContractBasis::from_contract(&contract);
    let duplicate = worth_foundational::PortableRecordAspectPatch::new([
        worth_foundational::PortableAspectPatchOperation::SetWhole {
            basis: basis.clone(),
            value: value.clone(),
        },
        worth_foundational::PortableAspectPatchOperation::SetWhole { basis, value },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            duplicate,
            worth_foundational::PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::DuplicateAspectOperation(_)
        )
    ));
}
