use worth_foundational::{
    export_portable_record_aspect_patch, readmit_portable_record_aspect_patch, AspectMask,
    AspectValue, AuthoritativeRecordAspectPatch, CanonicalFieldPath, ContractValidationInput,
    MutationMask, PortableAspectContractBasis, PortableAspectPatchOperation,
    PortableAspectReadmissionDenial, PortablePatchReadmissionPurpose, PortableRecordAspectPatch,
    StructAspectValue,
};
use worth_proof::TransitionOutcome;

use crate::certification::aspects::patches::patch_fixtures::{
    task_summary_contract, validated_task_summary,
};
use crate::certification::foundational_vocabulary::{field, identity, revision};

#[test]
fn serialized_whole_struct_patch_earns_fresh_authority_from_the_current_contract() {
    let contract = task_summary_contract();
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::whole_aspect(
        [validated_task_summary(
            &contract,
            "ship",
            false,
            Some("carefully"),
        )],
        [],
    ) else {
        panic!("expected authoritative patch");
    };

    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("current contract exports portable meaning");
    let bytes = serde_json::to_vec(&portable).expect("portable patch serializes");
    let transported = serde_json::from_slice(&bytes).expect("portable patch deserializes");
    let TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        transported,
        PortablePatchReadmissionPurpose::RecordMutation,
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
        CanonicalFieldPath::single(field("title")),
        CanonicalFieldPath::single(field("done")),
        CanonicalFieldPath::single(field("note")),
    ]);
    let TransitionOutcome::Success(patch) = AuthoritativeRecordAspectPatch::field_level(
        &contract,
        &mask,
        [(field("done"), AspectValue::Bool(true))],
        [field("note")],
    ) else {
        panic!("expected authoritative field patch");
    };

    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("field patch exports");
    let TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        portable,
        PortablePatchReadmissionPurpose::RecordMutation,
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
    let TransitionOutcome::Success(patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([], [contract.clone()])
    else {
        panic!("expected exact clear patch");
    };
    let portable = export_portable_record_aspect_patch(&patch, &lookup(&contract))
        .expect("exact clear exports");
    let TransitionOutcome::Success(readmitted) = readmit_portable_record_aspect_patch(
        portable,
        PortablePatchReadmissionPurpose::RecordDeletion,
        &lookup(&contract),
    ) else {
        panic!("exact clear should readmit for deletion");
    };

    assert_eq!(readmitted, patch);
}

#[test]
fn stale_contract_and_duplicate_operations_deny_before_authority_exists() {
    let contract = task_summary_contract();
    let stale_basis =
        PortableAspectContractBasis::new(contract.key().clone(), identity(20), revision(0));
    let value = ContractValidationInput::Struct(
        StructAspectValue::new([
            (field("title"), AspectValue::String("ship".into())),
            (field("done"), AspectValue::Bool(false)),
        ])
        .expect("unique fields"),
    );
    let stale = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: stale_basis,
        value: value.clone(),
    }]);

    assert!(matches!(
        readmit_portable_record_aspect_patch(
            stale,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::ContractRevisionMismatch { .. })
    ));

    let basis = PortableAspectContractBasis::from_contract(&contract);
    let duplicate = PortableRecordAspectPatch::new([
        PortableAspectPatchOperation::SetWhole {
            basis: basis.clone(),
            value: value.clone(),
        },
        PortableAspectPatchOperation::SetWhole { basis, value },
    ]);
    assert!(matches!(
        readmit_portable_record_aspect_patch(
            duplicate,
            PortablePatchReadmissionPurpose::RecordMutation,
            &lookup(&contract),
        ),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::DuplicateAspectOperation(_))
    ));
}

fn lookup(
    contract: &worth_foundational::AspectContract,
) -> impl Fn(&worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> + '_ {
    move |key| (key == contract.key()).then(|| contract.clone())
}
