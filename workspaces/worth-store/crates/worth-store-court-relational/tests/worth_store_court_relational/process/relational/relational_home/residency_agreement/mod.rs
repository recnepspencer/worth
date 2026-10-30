//! A real Store home agrees with an independent memory runtime at every warmth.
#[allow(unused_imports)]
use crate::residency_agreement::*;
use {
    worth_relational::facade::history::BranchId, worth_relational::facade::identity::PartitionId,
};

#[test]
fn cold_warm_and_full_home_reads_and_retained_observations_agree_with_memory() {
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let reference = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
        .build()
        .expect("valid native reference runtime configuration");
    let home = crate::durable_run::open(&location, &execution).unwrap();
    let branch = BranchId("main".into());
    let mut entities = Vec::new();
    let mut history_commit = None;
    for partition in 1..=24 {
        let mut batch = crate::batches::create(partition as usize);
        let worth_relational::facade::transactions::MutationIntent::Create(
            worth_relational::facade::transactions::CreateIntent::Entity(spec),
        ) = &mut batch.intents[0]
        else {
            panic!("entity fixture")
        };
        spec.partition_id = PartitionId(partition);
        let expected =
            crate::workload::commit(&reference, &execution, &branch, batch.clone()).unwrap();
        let actual = crate::workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
        assert_eq!(actual.changed_records, expected.changed_records);
        let worth_relational::facade::transactions::RecordRef::Entity(entity) =
            actual.changed_records[0]
        else {
            panic!("created entity")
        };
        entities.push(entity);
        history_commit.get_or_insert(actual.commit.commit_id);
    }
    let historical_id = history_commit.unwrap();
    let bound = std::num::NonZeroUsize::new(16).unwrap();
    let expected_history = reference
        .history()
        .bounded_canonical_commit_patches(&execution, historical_id, bound)
        .unwrap();
    let expected = crate::workload::states_of(
        &reference,
        &entities
            .iter()
            .copied()
            .map(worth_relational::facade::transactions::RecordRef::Entity)
            .collect::<Vec<_>>(),
    );
    home.checkpoint(&execution).unwrap();
    home.close();
    let home = crate::durable_run::open(&location, &execution).unwrap();
    let reopened = home.cold_storage_reads();
    assert_eq!(
        reopened.member_reads, 0,
        "reopen admits no symbol, lineage or region payloads"
    );
    assert!(
        reopened.commit_reads < 8,
        "only the selected head needs an envelope"
    );
    assert_eq!(
        home.runtime()
            .history()
            .bounded_canonical_commit_patches(&execution, historical_id, bound)
            .unwrap(),
        expected_history
    );
    let queried = home.cold_storage_reads();
    assert!(
        queried.commit_reads - reopened.commit_reads < 4,
        "a bounded history query admits only its selected envelope"
    );
    assert_eq!(
        queried.member_reads, reopened.member_reads,
        "canonical patch history needs no dictionary or lineage member"
    );
    let records = entities
        .iter()
        .copied()
        .map(worth_relational::facade::transactions::RecordRef::Entity)
        .collect::<Vec<_>>();
    assert_eq!(
        crate::workload::states_of(home.runtime(), &records),
        expected,
        "cold point reads agree"
    );
    let warm = home.cold_storage_reads();
    assert_eq!(
        crate::workload::states_of(home.runtime(), &records),
        expected,
        "warm point reads agree"
    );
    assert_eq!(
        home.cold_storage_reads(),
        warm,
        "warm points perform no I/O"
    );
    home.runtime()
        .durability_authority()
        .checkpoint(&execution)
        .unwrap();
    assert_eq!(
        crate::workload::states_of(home.runtime(), &records),
        expected,
        "fully admitted Full reads agree"
    );
    let selected = observation(home.runtime());
    let memory_selected = observation(&reference);
    let retained_expected = render(&reference, &execution, &memory_selected, &entities);
    // Retain the selected root while new publications replace its live carriers.
    for step in 0..3 {
        let batch = crate::batches::update(entities[0], 1000 + step);
        crate::workload::commit(&reference, &execution, &branch, batch.clone()).unwrap();
        crate::workload::commit(home.runtime(), &execution, &branch, batch).unwrap();
        home.checkpoint(&execution).unwrap();
        assert_eq!(
            render(home.runtime(), &execution, &selected, &entities),
            retained_expected,
            "the retained observation keeps its pre-commit meaning"
        );
        assert_eq!(
            render(&reference, &execution, &memory_selected, &entities),
            retained_expected
        );
        assert_eq!(
            crate::workload::states_of(home.runtime(), &records),
            crate::workload::states_of(&reference, &records)
        );
    }
    assert_eq!(
        crate::workload::artifacts_of(home.runtime()),
        crate::workload::artifacts_of(&reference)
    );
    drop(selected);
    home.close();
    let home = crate::durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    assert_eq!(
        crate::workload::states_of(home.runtime(), &records),
        crate::workload::states_of(&reference, &records)
    );
    home.close();
}
