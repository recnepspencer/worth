//! Physical membership, rather than encoded-byte estimates, proves reuse.
#[allow(unused_imports)] use crate::residency_capture::*;
use {worth_execution::ExecutionRequest};
use {worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId, worth_relational::facade::transactions::CreateIntent, worth_relational::facade::transactions::MutationIntent};
use {worth_store::physical_runtime::AnchoredRecordMemberLocator, worth_store::physical_runtime::AnchoredRecordName};
use {worth_store_relational::RelationalStoreHome};

use {crate::batches, crate::durable_run, crate::workload};

#[test]
fn a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = crate::durable_run::open(&location, &execution).unwrap();
    let branch = BranchId("main".into());
    for partition in 0..48_u32 {
        let mut batch = crate::batches::create(partition as usize);
        let worth_relational::facade::transactions::MutationIntent::Create(worth_relational::facade::transactions::CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
            panic!("entity create")
        };
        spec.partition_id = PartitionId(partition);
        crate::workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
    }
    let reads = home.cold_storage_reads();
    home.checkpoint(&execution).unwrap();
    assert_eq!(
        home.cold_storage_reads(),
        reads,
        "capture must not demand any member or envelope"
    );
    let first = members(&home);
    assert!(
        first.len() >= 48 * 2,
        "head and history are separate members, deduplicated across root and mirror"
    );
    crate::workload::commit(home.runtime(), &execution, &branch, crate::batches::create(1000)).unwrap();
    let reads = home.cold_storage_reads();
    home.checkpoint(&execution).unwrap();
    assert_eq!(home.cold_storage_reads(), reads);
    let second = members(&home);
    let added = second
        .iter()
        .filter(|member| !first.contains(member))
        .count();
    let carried = second
        .iter()
        .filter(|member| first.contains(member))
        .count();
    assert!(
        added < 12,
        "one touched partition published {added} new physical members"
    );
    assert!(
        carried > 48,
        "unchanged cells must carry their physical locators"
    );
    home.checkpoint(&execution).unwrap();
    assert_eq!(
        members(&home),
        second,
        "a successful publication acknowledges all locators"
    );
    assert_eq!(home.cold_storage_reads(), reads);
    home.close();
}
