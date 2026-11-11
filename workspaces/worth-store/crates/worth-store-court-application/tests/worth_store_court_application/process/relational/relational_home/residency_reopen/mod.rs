//! The production home must serve an untouched cold directory after reopen.
use crate::residency_capture::members_of_world;
use {
    worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId,
};

#[test]
fn reopen_keeps_regions_cold_and_a_point_read_and_commit_demand_only_their_partition() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::SOURCE,
        true,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    let branch = BranchId("main".into());
    let mut observed = None;
    for round in 0..3 {
        for partition in 1..=48_u32 {
            let mut batch = crate::batches::create(round * 48 + partition as usize);
            let worth_relational::facade::transactions::MutationIntent::Create(
                worth_relational::facade::transactions::CreateIntent::Entity(spec),
            ) = &mut batch.intents[0]
            else {
                panic!("entity create")
            };
            spec.partition_id = PartitionId(partition);
            let commit =
                crate::workload::commit(home.relational().runtime(), &execution, &branch, batch)
                    .unwrap();
            if observed.is_none() {
                observed = commit.changed_records.first().cloned();
            }
            home.relational()
                .runtime()
                .snapshots()
                .release_snapshot(&commit.snapshot)
                .unwrap();
        }
    }
    let record = observed.unwrap();
    let expected =
        crate::workload::states_of(home.relational().runtime(), std::slice::from_ref(&record));
    home.checkpoint(&execution).unwrap();
    crate::durable_run::world::close(fixture, &execution);

    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::SOURCE,
        false,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    let reopened = home.relational().cold_storage_reads();
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
        crate::workload::states_of(home.relational().runtime(), std::slice::from_ref(&record)),
        expected
    );
    let first_read = home.relational().cold_storage_reads();
    assert!(
        first_read.member_reads < 12,
        "one point read loaded {} members",
        first_read.member_reads
    );
    assert_eq!(
        crate::workload::states_of(home.relational().runtime(), std::slice::from_ref(&record)),
        expected,
        "warm and cold ordinary transaction reads agree"
    );
    assert_eq!(
        home.relational().cold_storage_reads(),
        first_read,
        "the warmed point read performs no I/O"
    );

    let before = members_of_world(&fixture);
    let mut batch = crate::batches::create(1000);
    let worth_relational::facade::transactions::MutationIntent::Create(
        worth_relational::facade::transactions::CreateIntent::Entity(spec),
    ) = &mut batch.intents[0]
    else {
        panic!("entity create")
    };
    spec.partition_id = PartitionId(1);
    let committed =
        crate::workload::commit(home.relational().runtime(), &execution, &branch, batch).unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&committed.snapshot)
        .unwrap();
    let committed = home.relational().cold_storage_reads();
    assert!(
        committed.member_reads - first_read.member_reads < 12,
        "one partition commit warmed {} additional members",
        committed.member_reads - first_read.member_reads
    );
    crate::durable_run::world::close(fixture, &execution);
    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::SOURCE,
        false,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    let replayed = home.relational().cold_storage_reads();
    assert!(
        replayed.member_reads < 12,
        "one-partition tail replay loaded {} members",
        replayed.member_reads
    );
    home.checkpoint(&execution).unwrap();
    assert_eq!(
        home.relational().cold_storage_reads(),
        replayed,
        "capture must walk leaves without demanding payloads"
    );
    let added = members_of_world(&fixture)
        .into_iter()
        .filter(|member| !before.contains(member))
        .count();
    assert!(
        added < 12,
        "one partition commit published {added} new physical members"
    );
    let embedded = home
        .relational()
        .runtime()
        .durability_authority()
        .native_checkpoint(
            &execution,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut receiving = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
        .build()
        .expect("valid native reference runtime configuration");
    receiving
        .durability_recovery()
        .restore_native_checkpoint(&execution, &embedded)
        .unwrap();
    assert_eq!(
        crate::workload::artifacts_of(&execution, &receiving),
        crate::workload::artifacts_of(&execution, home.relational().runtime())
    );
    assert_eq!(
        crate::workload::states_of(&receiving, std::slice::from_ref(&record)),
        expected
    );
    drop((receiving, embedded));
    crate::durable_run::world::close(fixture, &execution);
}

#[test]
fn a_cold_schema_transition_replays_and_reframes_all_carriers_before_load_free_capture() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::SOURCE,
        true,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    let reference = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
        .build()
        .expect("valid native reference runtime configuration");
    let branch = BranchId("main".into());
    let mut records = Vec::new();
    for partition in 1..=8_u32 {
        let mut batch = crate::batches::create(partition as usize);
        let worth_relational::facade::transactions::MutationIntent::Create(
            worth_relational::facade::transactions::CreateIntent::Entity(spec),
        ) = &mut batch.intents[0]
        else {
            panic!("entity create")
        };
        spec.partition_id = PartitionId(partition);
        let expected =
            crate::workload::commit(&reference, &execution, &branch, batch.clone()).unwrap();
        let committed =
            crate::workload::commit(home.relational().runtime(), &execution, &branch, batch)
                .unwrap();
        records.extend(committed.changed_records.iter().cloned());
        reference
            .snapshots()
            .release_snapshot(&expected.snapshot)
            .unwrap();
        home.relational()
            .runtime()
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
    }
    let transitioned = crate::schema_transition::commit(
        &reference,
        &execution,
        &branch,
        crate::batches::create(2000),
    )
    .unwrap();
    reference
        .snapshots()
        .release_snapshot(&transitioned.snapshot)
        .unwrap();
    let expected = crate::workload::states_of(&reference, &records);
    home.checkpoint(&execution).unwrap();
    crate::durable_run::world::close(fixture, &execution);

    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::SOURCE,
        false,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    assert_eq!(home.relational().cold_storage_reads().member_reads, 0);
    let transitioned = crate::schema_transition::commit(
        home.relational().runtime(),
        &execution,
        &branch,
        crate::batches::create(2000),
    )
    .unwrap();
    home.relational()
        .runtime()
        .snapshots()
        .release_snapshot(&transitioned.snapshot)
        .unwrap();
    crate::durable_run::world::close(fixture, &execution);
    // The schema transition is in the tail above the original cold manifest.
    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::TARGET,
        false,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    let before = home.relational().cold_storage_reads();
    home.checkpoint(&execution).unwrap();
    assert_eq!(
        home.relational().cold_storage_reads(),
        before,
        "carrier reframing must use preparation's admitted contents"
    );
    crate::durable_run::world::close(fixture, &execution);

    let fixture = crate::durable_run::world::open(
        &location,
        crate::schema_transition::TARGET,
        false,
        &execution,
    )
    .unwrap();
    let home = &fixture.home;
    assert_eq!(home.relational().cold_storage_reads().member_reads, 0);
    assert_eq!(
        crate::workload::states_of(home.relational().runtime(), &records),
        expected
    );
    crate::durable_run::world::close(fixture, &execution);
}

#[test]
fn a_cold_first_commit_on_a_fork_preserves_reference_artifacts_and_record_identities() {
    for seed in [0x5eed_0301, 0x5eed_0302, 0x5eed_0402] {
        let script = crate::workload::Script::generate(seed, 9, 3);
        let fork = script
            .steps
            .iter()
            .position(|step| matches!(step, crate::workload::Step::Fork { .. }))
            .expect("the script forks");
        let target = fork + 1;
        assert!(script.steps[target].is_commit());
        let host = crate::workload::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let prepared =
            crate::prepared_home::PreparedHome::prepare(&script.steps[..target], &execution);
        let home = crate::durable_run::world::open(
            prepared.location(),
            crate::durable_run::served_schema(&script.steps[..target]),
            false,
            &execution,
        )
        .unwrap();
        assert_eq!(
            home.home.relational().cold_storage_reads().member_reads,
            0,
            "fork recovery stays cold"
        );
        crate::durable_run::world::apply(&home.home, &execution, &script.steps[target..target + 1]);
        crate::durable_run::assert_reference_artifacts(
            &crate::workload::artifacts_of(&execution, home.home.relational().runtime()),
            &script,
            &format!("cold fork seed {seed:#x}"),
        );
        crate::durable_run::world::apply(&home.home, &execution, &script.steps[target + 1..]);
        crate::durable_run::world::assert_holds(&execution, &home.home, &script, &script.artifacts);
        crate::durable_run::world::close(home, &execution);
    }
}
