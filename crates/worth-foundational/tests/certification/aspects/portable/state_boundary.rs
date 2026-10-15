use worth_foundational::{
    export_portable_record_aspect_state, readmit_portable_record_aspect_state, AspectContract,
    AspectValue, ContractValidationInput, PortableAspectContractBasis,
    PortableAspectReadmissionDenial, PortableRecordAspectState, PortableRecordAspectStateEntry,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use crate::certification::aspects::patches::patch_fixtures::{
    admitted_state, task_summary_contract, validated_task_summary,
};
use crate::certification::foundational_vocabulary::{identity, key, revision};

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
    let TransitionOutcome::Success(readmitted) =
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
    let duplicate = PortableRecordAspectState::new([entry.clone(), entry]);

    assert!(matches!(
        readmit_portable_record_aspect_state(duplicate, &lookup(&contract)),
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::StateAdmission(_))
    ));
}

#[test]
fn a_late_invalid_entry_denies_the_complete_snapshot() {
    let summary_contract = task_summary_contract();
    let count_contract = AspectContract::scalar(
        key("count"),
        identity(21),
        revision(1),
        ScalarAspectType::Int64,
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
    let invalid_entry = PortableRecordAspectStateEntry::new(
        PortableAspectContractBasis::from_contract(&count_contract),
        ContractValidationInput::Scalar(AspectValue::Bool(true)),
    );
    let snapshot =
        PortableRecordAspectState::new(portable.entries().iter().cloned().chain([invalid_entry]));
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
        TransitionOutcome::Denied(PortableAspectReadmissionDenial::ValueValidation { .. })
    ));
}

fn lookup(
    contract: &worth_foundational::AspectContract,
) -> impl Fn(&worth_foundational::AspectKey) -> Option<worth_foundational::AspectContract> + '_ {
    move |key| (key == contract.key()).then(|| contract.clone())
}
