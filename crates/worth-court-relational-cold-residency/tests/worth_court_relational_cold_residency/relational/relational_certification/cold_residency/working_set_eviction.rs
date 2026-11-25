//! S1/M1 and S3 through the public facade: a Home reopened under a declarative
//! `ResidencyBudget` keeps an open root's scans within the working set, really
//! returns memory, and refuses genuine saturation with typed `MemoryExhausted`.
use super::working_set_fixture::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use worth_execution::ExecutionRequest;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::cold_recovery::update_entity;
use worth_testkit_relational::durable_publication::field_patch;

#[test]
fn an_open_root_scanned_twice_with_concurrent_edits_stays_within_its_working_set() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let home = cold_home(request);
    let head = head_region_bytes(request, &home);
    // One scan window, the editor's transaction and the mirror it edits fit;
    // the root itself is more than four times the working set.
    let ceiling = 6 * head;
    assert!(u64::from(REGIONS) * head > 4 * ceiling);
    let runtime = reopen(request, &home, budget(ceiling, ceiling));
    // The original root stays open, through one held observation, for both scans.
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let observation = basis.observation();
    let reads_before = member_reads(&home);
    assert_eq!(
        scan_observation(request, &runtime, &observation).unwrap(),
        REGIONS as usize
    );
    let after_first = pressure(&runtime);
    assert!(after_first.head_resident_bytes <= ceiling);
    assert!(after_first.evictions_completed >= u64::from(REGIONS) - 6);
    assert_eq!(
        after_first.head_borrowed_bytes, 0,
        "the open observation retains no region past the scan (M1)"
    );
    let reads_after_first = member_reads(&home);
    assert!(reads_after_first - reads_before >= REGIONS as usize);

    let stop = AtomicBool::new(false);
    let edits = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
            let request = &ExecutionRequest::serial(&host);
            let mut turn = 0usize;
            while !stop.load(Ordering::Acquire) {
                let entity = home.entities[turn % home.entities.len()];
                update_entity(request, &runtime, entity, &format!("edit-{turn}"));
                edits.fetch_add(1, Ordering::AcqRel);
                turn += 7;
            }
        });
        struct StopOnDrop<'flag>(&'flag AtomicBool);
        impl Drop for StopOnDrop<'_> {
            fn drop(&mut self) {
                self.0.store(true, Ordering::Release);
            }
        }
        let _stop = StopOnDrop(&stop);
        // The held observation still reads its own immutable root.
        assert_eq!(
            scan_observation(request, &runtime, &observation).unwrap(),
            REGIONS as usize
        );
    });
    assert!(edits.load(Ordering::Acquire) > 0, "edits ran during scan 2");
    assert!(
        member_reads(&home) - reads_after_first >= REGIONS as usize - 6,
        "scan 2 rewarmed evicted regions from cold storage"
    );
    let settled = pressure(&runtime);
    assert!(settled.head_resident_bytes <= ceiling);
    assert!(settled.evictions_completed > after_first.evictions_completed);
    // The mirror keeps its edit targets; the open root keeps nothing (M1).
    drop((observation, basis));
    assert_eq!(
        pressure(&runtime).head_borrowed_bytes,
        settled.head_borrowed_bytes,
        "releasing the open root frees no borrow, because it held none"
    );
    assert_eq!(scan(request, &runtime), REGIONS as usize);
}

#[test]
fn borrows_pinning_the_working_set_refuse_new_reads_until_released() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let home = cold_home(request);
    let head = head_region_bytes(request, &home);
    let ceiling = 2 * head + head / 2;
    let runtime = reopen(request, &home, budget(ceiling, ceiling));
    // A side branch shares the reopened regions; its prepared candidate holds
    // two regions' Head custody until it is dropped.
    let (_, source) = runtime
        .observe_fork_source(&BranchId("main".into()))
        .unwrap();
    runtime
        .fork_branch(request, BranchId("side".into()), source)
        .unwrap();
    let side = runtime.branch_identity(&BranchId("side".into())).unwrap();
    let (_, side_basis) = runtime.observe_branch(&side).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&side_basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let mut batch = WorkerIntentBatch::new("hold");
    for entity in &home.entities[..2] {
        batch = batch.push(MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: *entity,
                fields: field_patch("name", "name", "held"),
            },
        )));
    }
    transaction
        .push_batch(batch, request.allocation_policy())
        .unwrap();
    let held = runtime
        .prepare_branch_transaction(request, transaction)
        .unwrap();
    let reads = member_reads(&home);
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let refused = scan_observation(request, &runtime, &basis.observation()).unwrap_err();
    assert!(memory_exhausted(&refused), "typed refusal: {refused:?}");
    assert!(pressure(&runtime).head_resident_bytes <= ceiling);
    assert!(
        member_reads(&home) - reads <= 2,
        "saturation refuses before reading the region that cannot fit"
    );
    drop(held);
    assert_eq!(
        scan_observation(request, &runtime, &basis.observation()).unwrap(),
        REGIONS as usize,
        "releasing the borrows restores service"
    );
}
