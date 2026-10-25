//! Physical membership, rather than encoded-byte estimates, proves reuse.
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    history::BranchId,
    identity::PartitionId,
    transactions::{CreateIntent, MutationIntent},
};
use worth_store::physical_runtime::{AnchoredRecordMemberLocator, AnchoredRecordName};
use worth_store_relational::RelationalStoreHome;

use super::{batches, durable_run, workload};

pub(super) fn members(home: &RelationalStoreHome) -> Vec<AnchoredRecordMemberLocator> {
    home.certification_serving()
        .anchored_records()
        .unwrap()
        .read(
            AnchoredRecordName::new(super::selected_checkpoint::ANCHOR).unwrap(),
            1 << 30,
        )
        .unwrap()
        .unwrap()
        .members()
        .to_vec()
}

#[test]
fn a_one_partition_commit_publishes_constant_new_members_and_capture_reads_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = durable_run::open(&location, &execution).unwrap();
    let branch = BranchId("main".into());
    for partition in 0..48_u32 {
        let mut batch = batches::create(partition as usize);
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
            panic!("entity create")
        };
        spec.partition_id = PartitionId(partition);
        workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
    }
    let reads = home.cold_storage_reads();
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
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
    workload::commit(home.runtime(), &execution, &branch, batches::create(1000)).unwrap();
    let reads = home.cold_storage_reads();
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
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
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(
        members(&home),
        second,
        "a successful publication acknowledges all locators"
    );
    assert_eq!(home.cold_storage_reads(), reads);
    home.close();
}
