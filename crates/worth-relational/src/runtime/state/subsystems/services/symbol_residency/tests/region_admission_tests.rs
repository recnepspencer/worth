//! A region's dictionary closure is admitted once per runtime and view.
use super::*;

#[test]
fn region_symbol_admission_is_retryable_and_owned_by_each_fork() {
    let (table, _, source, all) = dictionary();
    let fork = table.detached_owner_snapshot();
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let entity = create_entity(&execution, &runtime, "region-admission");
    let root = runtime
        .history
        .branch_cell(&crate::history::data::BranchId("main".into()))
        .unwrap()
        .root()
        .unwrap();
    let lease = root
        .demand_region_head(&execution, entity.partition_id)
        .unwrap()
        .unwrap();
    let symbol = all.entries[0].0;
    *source.failure.lock().unwrap() = Some(Cause::MemoryExhausted);
    assert!(
        matches!(table.demand_region_symbols(&execution, lease.symbol_admission_identity(),
        || BTreeSet::from([symbol])), Err(Failure::StorageUnavailable(failure))
        if failure.cause == Cause::MemoryExhausted)
    );
    *source.failure.lock().unwrap() = None;
    table
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            BTreeSet::from([symbol])
        })
        .unwrap();
    table
        .demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
            panic!("a warm region must not rescan its dictionary closure")
        })
        .unwrap();
    assert!(fork.with_read(|symbols| symbols.retained_text(symbol).is_none()));
    let required = std::cell::Cell::new(false);
    fork.demand_region_symbols(&execution, lease.symbol_admission_identity(), || {
        required.set(true);
        BTreeSet::from([symbol])
    })
    .unwrap();
    assert!(
        required.get(),
        "another runtime must admit this view independently"
    );
    assert!(fork.with_read(|symbols| symbols.retained_text(symbol).is_some()));
}
