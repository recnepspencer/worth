//! The production home must serve an untouched cold directory after reopen.
use super::{batches, durable_run, residency_capture::members, workload};
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::{
    history::BranchId,
    identity::PartitionId,
    transactions::{CreateIntent, MutationIntent},
};

#[test]
fn reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let home = durable_run::open(&location, &execution).unwrap();
    let branch = BranchId("main".into());
    let mut observed = None;
    for round in 0..3 {
        for partition in 1..=48_u32 {
            let mut batch = batches::create(round * 48 + partition as usize);
            let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
                panic!("entity create")
            };
            spec.partition_id = PartitionId(partition);
            let commit = workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
            if observed.is_none() {
                observed = commit.changed_records.first().cloned();
            }
        }
    }
    let record = observed.unwrap();
    let expected = workload::states_of(home.runtime(), std::slice::from_ref(&record));
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    home.close();

    let home = durable_run::open(&location, &execution).unwrap();
    let reopened = home.cold_storage_reads();
    assert_eq!(
        reopened.member_reads, 0,
        "reopen must read the manifest without any region, symbol, lineage or index member"
    );
    assert!(
        reopened.commit_reads < 8,
        "reopen read {} envelopes for one distinct head",
        reopened.commit_reads
    );
    assert_eq!(
        workload::states_of(home.runtime(), std::slice::from_ref(&record)),
        expected
    );
    let first_read = home.cold_storage_reads();
    assert!(
        first_read.member_reads < 12,
        "one point read loaded {} members",
        first_read.member_reads
    );
    assert_eq!(
        workload::states_of(home.runtime(), std::slice::from_ref(&record)),
        expected,
        "warm and cold ordinary transaction reads agree"
    );
    assert_eq!(
        home.cold_storage_reads(),
        first_read,
        "the warmed point read performs no I/O"
    );

    let before = members(&home);
    let mut batch = batches::create(1000);
    let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
        panic!("entity create")
    };
    spec.partition_id = PartitionId(1);
    workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
    let committed = home.cold_storage_reads();
    assert!(
        committed.member_reads - first_read.member_reads < 12,
        "one partition commit warmed {} additional members",
        committed.member_reads - first_read.member_reads
    );
    home.close();
    let home = durable_run::open(&location, &execution).unwrap();
    let replayed = home.cold_storage_reads();
    assert!(
        replayed.member_reads < 12,
        "one-partition tail replay loaded {} members",
        replayed.member_reads
    );
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(
        home.cold_storage_reads(),
        replayed,
        "capture must walk leaves without demanding payloads"
    );
    let added = members(&home)
        .into_iter()
        .filter(|member| !before.contains(member))
        .count();
    assert!(
        added < 12,
        "one partition commit published {added} new physical members"
    );
    let embedded = home
        .runtime()
        .durability_authority()
        .native_checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let mut receiving = workload::runtime_builder(super::schema_transition::SOURCE).build();
    receiving
        .durability_recovery()
        .restore_native_checkpoint(&execution, &embedded)
        .unwrap();
    assert_eq!(
        workload::artifacts_of(&receiving),
        workload::artifacts_of(home.runtime())
    );
    assert_eq!(
        workload::states_of(&receiving, std::slice::from_ref(&record)),
        expected
    );
    home.close();
}

#[test]
fn a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let home = durable_run::open(&location, &execution).unwrap();
    let reference = workload::runtime_builder(super::schema_transition::SOURCE).build();
    let branch = BranchId("main".into());
    let mut records = Vec::new();
    for partition in 1..=8_u32 {
        let mut batch = batches::create(partition as usize);
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut batch.intents[0] else {
            panic!("entity create")
        };
        spec.partition_id = PartitionId(partition);
        workload::commit(&reference, &execution, &branch, batch.clone()).unwrap();
        let committed = workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
        records.extend(committed.changed_records.iter().cloned());
    }
    super::schema_transition::commit(&reference, &execution, &branch, batches::create(2000))
        .unwrap();
    let expected = workload::states_of(&reference, &records);
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    home.close();

    let home = durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    super::schema_transition::commit(home.runtime(), &execution, &branch, batches::create(2000))
        .unwrap();
    home.close();
    // The schema transition is in the tail above the original cold manifest.
    let home = durable_run::open(&location, &execution).unwrap();
    let before = home.cold_storage_reads();
    home.checkpoint(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert_eq!(
        home.cold_storage_reads(),
        before,
        "carrier reframing must use preparation's admitted contents"
    );
    home.close();

    let home = durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    assert_eq!(workload::states_of(home.runtime(), &records), expected);
    home.close();
}
