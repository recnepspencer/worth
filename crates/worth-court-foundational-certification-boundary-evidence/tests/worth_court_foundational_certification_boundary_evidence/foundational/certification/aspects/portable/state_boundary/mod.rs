#[allow(unused_imports)]
use crate::certification::ordinary_context::aspects::portable::state_boundary::*;
use {
    worth_foundational::export_portable_record_aspect_state,
    worth_foundational::readmit_portable_record_aspect_state,
};

use {
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::admitted_state,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::task_summary_contract,
    crate::certification::ordinary_context::aspects::patches::patch_fixtures::validated_task_summary,
};
use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn serialized_state_is_only_a_candidate_until_every_entry_readmits() {
    let contract = task_summary_contract();
    let state = admitted_state([validated_task_summary(
        &contract,
        "restore",
        true,
        Some("atomically"),
    )]);
    let portable = export_portable_record_aspect_state(state.payload(), &lookup(&contract))
        .expect("state exports");
    let bytes = serde_json::to_vec(&portable).expect("portable state serializes");
    let transported = serde_json::from_slice(&bytes).expect("portable state deserializes");
    let worth_proof::TransitionOutcome::Success(readmitted) =
        readmit_portable_record_aspect_state(transported, &lookup(&contract))
    else {
        panic!("state should earn fresh authority");
    };

    assert_eq!(readmitted.payload(), state.payload());
}

#[test]
fn duplicate_state_entries_deny_the_complete_snapshot() {
    let contract = task_summary_contract();
    let state = admitted_state([validated_task_summary(&contract, "restore", true, None)]);
    let portable = export_portable_record_aspect_state(state.payload(), &lookup(&contract))
        .expect("state exports");
    let entry = portable.entries()[0].clone();
    let duplicate = worth_foundational::PortableRecordAspectState::new([entry.clone(), entry]);

    assert!(matches!(
        readmit_portable_record_aspect_state(duplicate, &lookup(&contract)),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::StateAdmission(_)
        )
    ));
}

#[test]
fn a_late_invalid_entry_denies_the_complete_snapshot() {
    let summary_contract = task_summary_contract();
    let count_contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(21),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    let valid_state = admitted_state([validated_task_summary(
        &summary_contract,
        "restore",
        true,
        None,
    )]);
    let portable =
        export_portable_record_aspect_state(valid_state.payload(), &lookup(&summary_contract))
            .expect("state exports");
    let invalid_entry = worth_foundational::PortableRecordAspectStateEntry::new(
        worth_foundational::PortableAspectContractBasis::from_contract(&count_contract),
        worth_foundational::ContractValidationInput::Scalar(worth_foundational::AspectValue::Bool(
            true,
        )),
    );
    let snapshot = worth_foundational::PortableRecordAspectState::new(
        portable.entries().iter().cloned().chain([invalid_entry]),
    );
    let contracts = |aspect_key: &worth_foundational::AspectKey| {
        if aspect_key == summary_contract.key() {
            Some(summary_contract.clone())
        } else if aspect_key == count_contract.key() {
            Some(count_contract.clone())
        } else {
            None
        }
    };

    assert!(matches!(
        readmit_portable_record_aspect_state(snapshot, &contracts),
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::PortableAspectReadmissionDenial::ValueValidation { .. }
        )
    ));
}
