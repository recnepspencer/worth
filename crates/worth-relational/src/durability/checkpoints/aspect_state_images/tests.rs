use super::{export_state, CheckpointAspectContractCatalog};
use crate::storage::substrate::{EntityExtra, SharedColumn, StorageAllocation};
use worth_execution::{ExecutionRequest, SerialMemoryBudget};
use worth_foundational::facade::InternedString;
use worth_foundational::facade::{
    validate_aspect_value, AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
    AspectValue, AuthoritativeRecordAspectPatch, AuthoritativeRecordAspectState,
    ContractValidationInput, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

#[test]
fn unique_extra_move_and_borrowed_export_preserve_original_funding() {
    assert_unique_extra_move_funding(|state, contract| {
        let catalog = CheckpointAspectContractCatalog::from_contracts(&[contract]).unwrap();
        assert!(export_state(state, Some(&catalog)).unwrap().is_some());
    });
}

fn patch(text: &str) -> (AspectContract, AuthoritativeRecordAspectPatch) {
    let contract = AspectContract::scalar(
        AspectKey::new("data").unwrap(),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let TransitionOutcome::Success(value) = validate_aspect_value(
        &contract,
        ContractValidationInput::Scalar(AspectValue::String(InternedString::Raw(text.into()))),
    ) else {
        panic!("admitted source value")
    };
    let TransitionOutcome::Success(patch) =
        AuthoritativeRecordAspectPatch::whole_aspect([value], [])
    else {
        panic!("admitted source patch")
    };
    (contract, patch)
}

fn assert_unique_extra_move_funding(
    observe: impl FnOnce(Option<&AuthoritativeRecordAspectState>, AspectContract),
) {
    let budget = SerialMemoryBudget::new(Some(16 * 1024));
    let host = worth_harness::facade::execution_fixture::serial_request_with_memory(budget.clone());
    let execution = ExecutionRequest::serial(&host);
    let (contract, patch) = patch("move-and-export");
    let value = EntityExtra::apply_payload(&execution, None, &patch).unwrap();
    let bill = StorageAllocation::<EntityExtra>::layout_bytes()
        + value.authoritative_aspect_state().map_or(0, |state| {
            u64::try_from(state.aspects().retained_storage_bytes().unwrap()).unwrap()
        });
    let mut column =
        SharedColumn::<crate::storage::substrate::StorageAllocation<EntityExtra>>::default();
    column.push(value);
    let moved = column.into_vec();
    assert_eq!(
        budget.charged_memory_bytes(),
        bill,
        "the whole unique extra moved, including its footer"
    );
    observe(moved[0].authoritative_aspect_state(), contract);
    assert_eq!(budget.charged_memory_bytes(), bill);
    drop(moved);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
