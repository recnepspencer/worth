//! A descriptive receipt outlives its compiler without retaining Home custody.
use super::*;

#[test]
fn retained_catalog_receipts_keep_only_their_identity_charge_and_allow_same_home_reopen() {
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let first = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let memory = SerialMemoryBudget::new(None);
    let compilation =
        SerialRequest::from_memory(memory.clone(), CancellationSource::new().token(), None);
    let compile = || {
        let native = fixture.home.relational().runtime();
        native
            .prepare_home_program_contracts(
                &ExecutionRequest::serial(&compilation),
                first.basis().relational_basis(),
                native.config().schema.registry.clone(),
                Vec::new(),
                Vec::new(),
            )
            .unwrap()
    };
    let prepared = compile();
    let receipt = prepared.catalog_receipt();
    assert!(receipt.same_inventory(&prepared.catalog_receipt()));
    let independently_compiled = compile();
    assert_eq!(
        prepared.catalog_digest(),
        independently_compiled.catalog_digest()
    );
    let independent = independently_compiled.catalog_receipt();
    assert!(!receipt.same_inventory(&independent));
    drop(independent);
    drop(independently_compiled);
    let full_charge = memory.charged_memory_bytes();
    drop(prepared);
    let identity_charge = memory.charged_memory_bytes();
    assert!(identity_charge > 0 && identity_charge < full_charge);
    drop(first);
    drop(fixture);
    assert_eq!(memory.charged_memory_bytes(), identity_charge);
    let reopened = worth_store_testkit_application_oracle::world_home::open(
        super::transition_opening::successor_request(directory.path()),
        &execution,
    )
    .expect("a retained descriptive receipt owns no lease or live read resource");
    assert_eq!(memory.charged_memory_bytes(), identity_charge);
    let shared_receipt = receipt.clone();
    drop(receipt);
    assert_eq!(memory.charged_memory_bytes(), identity_charge);
    drop(shared_receipt);
    assert_eq!(memory.charged_memory_bytes(), 0);
    drop(reopened);
}
