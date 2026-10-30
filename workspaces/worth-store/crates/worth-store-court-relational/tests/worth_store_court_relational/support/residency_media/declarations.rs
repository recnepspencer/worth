//! Ordinary home reads distinguish unavailable members from absent records.
pub(crate) use worth_execution::ExecutionRequest;
pub(crate) use worth_store_relational::RelationalStoreHome;
pub(crate) use {
    crate::durable_run, crate::open_refusals, crate::residency_fixture::Fixture,
    crate::selected_checkpoint, crate::workload,
};
pub(crate) use {
    worth_relational::facade::mvcc::RelationalTransactionIntent,
    worth_relational::facade::mvcc::RelationalTransactionReadDenial,
    worth_relational::facade::storage::RelationalStorageSubject,
    worth_relational::facade::storage::RelationalStorageUnavailableCause,
    worth_relational::facade::transactions::RecordRef,
};
pub(crate) use {
    worth_store::physical_runtime::AnchoredRecordMember,
    worth_store::physical_runtime::AnchoredRecordName,
    worth_store::physical_runtime::AnchoredRecordPublication,
    worth_store::physical_runtime::AnchoredRecordsDenial,
    worth_store::physical_runtime::PhysicalMutationDeadline,
};

/// Keep both levels cold: a Relational miss alone need not reach media when
/// Store's readmission already cached the same physical page.
pub(crate) fn prepare_cold_media_read(home: &RelationalStoreHome) {
    assert_eq!(
        home.cold_storage_reads().member_reads,
        0,
        "the target must start with cold Relational cells"
    );
    home.certification_serving()
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
}

pub(crate) fn read_first(
    home: &RelationalStoreHome,
    script: &crate::workload::Script,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<String, RelationalTransactionReadDenial> {
    let commits = home.runtime().history().immutable_commit_count();
    read(
        home,
        &script.records[first_live(script, commits)],
        execution,
    )
}

pub(crate) fn expected_first(script: &crate::workload::Script, commits: usize) -> &str {
    &script.states[commits]["main"][first_live(script, commits)]
}

pub(crate) fn first_live(script: &crate::workload::Script, commits: usize) -> usize {
    script.states[commits]["main"]
        .iter()
        .position(|state| state != "None")
        .expect("a prefix with a live record to demand")
}

pub(crate) fn read(
    home: &RelationalStoreHome,
    record: &RecordRef,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<String, RelationalTransactionReadDenial> {
    let runtime = home.runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut reader = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    match record {
        worth_relational::facade::transactions::RecordRef::Entity(id) => reader
            .read_entity(execution, *id)
            .map(|record| format!("{:?}", record.effective())),
        worth_relational::facade::transactions::RecordRef::Relation(id) => reader
            .read_relation(execution, *id)
            .map(|record| format!("{:?}", record.effective())),
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DirectoryFault {
    Missing,
    Stale,
}

/// A record-checksum fault affects its physical frame. Put the healthy region's
/// head in an extent so its own bytes remain intact when region 1 is damaged.
pub(crate) fn isolate_healthy_region(fixture: &mut Fixture, execution: &ExecutionRequest<'_, '_>) {
    use {
        worth_relational::facade::history::BranchId,
        worth_relational::facade::identity::PartitionId,
        worth_relational::facade::transactions::CreateIntent,
        worth_relational::facade::transactions::MutationIntent,
        worth_relational::facade::transactions::WorkerIntentBatch,
    };
    let mut batch = WorkerIntentBatch::new("extent-backed healthy subject");
    for index in 10_000..10_512 {
        let mut create = crate::batches::create(index);
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut create.intents[0] else {
            panic!("entity fixture")
        };
        spec.partition_id = PartitionId(24);
        batch.intents.extend(create.intents);
    }
    let branch = BranchId("main".into());
    let outcome =
        crate::workload::commit(&fixture.reference, execution, &branch, batch.clone()).unwrap();
    fixture
        .records
        .extend(outcome.changed_records.iter().cloned());
    fixture.steps.insert(
        fixture.steps.len() - 1,
        crate::workload::Step::Commit { branch, batch },
    );
}

/// Inspect the producer's framed member header, never a guessed locator.
pub(crate) fn region_head(
    home: &RelationalStoreHome,
) -> (
    worth_store::physical_runtime::AnchoredRecordMemberLocator,
    Vec<u8>,
) {
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor =
        worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR)
            .unwrap();
    let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
    selected
        .members()
        .iter()
        .find_map(|locator| {
            let bytes = records.read_member(anchor, *locator, 1 << 30).unwrap();
            let header = bytes.strip_prefix(b"WORTH-REL-REG6\n")?;
            (header[2..6] == 1_u32.to_be_bytes() && header[38] == 0).then_some((*locator, bytes))
        })
        .expect("partition one's head member")
}
