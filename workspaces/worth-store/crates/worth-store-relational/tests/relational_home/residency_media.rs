//! Ordinary home reads distinguish unavailable members from absent records.
use super::{
    durable_run, open_refusals, residency_fixture::Fixture, selected_checkpoint, workload,
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    mvcc::{RelationalTransactionIntent, RelationalTransactionReadDenial},
    storage::{RelationalStorageSubject, RelationalStorageUnavailableCause},
    transactions::RecordRef,
};
use worth_store::physical_runtime::{
    AnchoredRecordMember, AnchoredRecordName, AnchoredRecordPublication, AnchoredRecordsDenial,
    PhysicalMutationDeadline,
};
use worth_store_relational::RelationalStoreHome;

/// Keep both levels cold: a Relational miss alone need not reach media when
/// Store's readmission already cached the same physical page.
pub(super) fn prepare_cold_media_read(home: &RelationalStoreHome) {
    assert_eq!(
        home.cold_storage_reads().member_reads,
        0,
        "the target must start with cold Relational cells"
    );
    home.certification_serving()
        .certification_physical_residency()
        .drain_unpinned_clean_frames();
}

pub(super) fn read_first(
    home: &RelationalStoreHome,
    script: &workload::Script,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<String, RelationalTransactionReadDenial> {
    let commits = home.runtime().history().immutable_commit_count();
    read(
        home,
        &script.records[first_live(script, commits)],
        execution,
    )
}

pub(super) fn expected_first(script: &workload::Script, commits: usize) -> &str {
    &script.states[commits]["main"][first_live(script, commits)]
}

fn first_live(script: &workload::Script, commits: usize) -> usize {
    script.states[commits]["main"]
        .iter()
        .position(|state| state != "None")
        .expect("a prefix with a live record to demand")
}

pub(super) fn read(
    home: &RelationalStoreHome,
    record: &RecordRef,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<String, RelationalTransactionReadDenial> {
    let runtime = home.runtime();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    let mut reader = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    match record {
        RecordRef::Entity(id) => reader
            .read_entity(execution, *id)
            .map(|record| format!("{:?}", record.effective())),
        RecordRef::Relation(id) => reader
            .read_relation(execution, *id)
            .map(|record| format!("{:?}", record.effective())),
    }
}

#[derive(Clone, Copy, Debug)]
enum DirectoryFault {
    Missing,
    Stale,
}

#[test]
fn missing_and_stale_member_locators_damage_only_their_region_and_never_answer_empty() {
    for fault in [DirectoryFault::Missing, DirectoryFault::Stale] {
        let host = workload::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let fixture = Fixture::new(&execution);
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let home = durable_run::open(&location, &execution).unwrap();
        let stale = if matches!(fault, DirectoryFault::Stale) {
            durable_run::apply(&home, &execution, &fixture.steps[..24]);
            home.checkpoint(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            let earlier = region_head(&home).1;
            durable_run::apply(&home, &execution, &fixture.steps[24..]);
            Some(earlier)
        } else {
            durable_run::apply(&home, &execution, &fixture.steps);
            None
        };
        home.close();
        let home = durable_run::open(&location, &execution).unwrap();
        assert_eq!(home.cold_storage_reads().member_reads, 0);
        let records = home.certification_serving().anchored_records().unwrap();
        let anchor = AnchoredRecordName::new(selected_checkpoint::ANCHOR).unwrap();
        let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
        let (target, bytes) = region_head(&home);
        let publish = |members| {
            records
                .publish(
                    AnchoredRecordPublication::new(
                        anchor,
                        selected_checkpoint::generation(&home),
                        selected.bytes().to_vec(),
                        open_refusals::home_placement(),
                        PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                    )
                    .with_members(members),
                )
                .unwrap()
        };
        let mut members = selected
            .members()
            .iter()
            .filter(|member| **member != target)
            .map(|locator| AnchoredRecordMember::Carried { locator: *locator })
            .collect::<Vec<_>>();
        if let Some(stale) = stale {
            // An actual earlier head from this partition cannot answer a
            // demand for the current manifest's locator.
            use sha2::{Digest, Sha256};
            assert_ne!(stale, bytes);
            members.push(AnchoredRecordMember::New {
                key: Sha256::digest(&stale).into(),
                bytes: stale,
            });
        }
        publish(members);
        let failure = read(&home, &fixture.records[0], &execution).unwrap_err();
        let RelationalTransactionReadDenial::StorageUnavailable(unavailable) = failure else {
            panic!("{failure:?}")
        };
        assert_eq!(
            unavailable.subject,
            RelationalStorageSubject::Region(worth_relational::facade::identity::PartitionId(1))
        );
        assert_eq!(
            unavailable.cause,
            RelationalStorageUnavailableCause::MissingMember
        );
        let damaged = home.cold_storage_reads();
        assert_eq!(
            read(&home, &fixture.records[0], &execution),
            Err(failure.clone())
        );
        assert_eq!(
            home.cold_storage_reads(),
            damaged,
            "proven missing bytes are sticky"
        );
        assert_eq!(
            workload::states_of(home.runtime(), &fixture.records[1..2]),
            workload::states_of(&fixture.reference, &fixture.records[1..2]),
            "another cold subject still serves"
        );
        // Restore the directory through Store's publisher. The already damaged
        // cell stays damaged, while a fresh recovery serves the repaired home.
        let members = selected
            .members()
            .iter()
            .map(|locator| {
                if *locator == target {
                    AnchoredRecordMember::New {
                        key: target.sha256(),
                        bytes: bytes.clone(),
                    }
                } else {
                    AnchoredRecordMember::Carried { locator: *locator }
                }
            })
            .collect();
        publish(members);
        let repaired = home.cold_storage_reads();
        assert_eq!(read(&home, &fixture.records[0], &execution), Err(failure));
        assert_eq!(home.cold_storage_reads(), repaired);
        home.close();
        let home = durable_run::open(&location, &execution).unwrap();
        fixture.assert_reads(&home, &execution);
        home.close();
    }
}

#[test]
fn a_damaged_member_in_store_files_is_a_sticky_typed_refusal_after_reopen() {
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let mut fixture = Fixture::new(&execution);
    isolate_healthy_region(&mut fixture, &execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = durable_run::open(&location, &execution).unwrap();
    durable_run::apply(&home, &execution, &fixture.steps);
    let bytes = region_head(&home).1;
    let page_bytes = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap()
        .page_size()
        .bytes();
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor = AnchoredRecordName::new(selected_checkpoint::ANCHOR).unwrap();
    let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
    assert!(
        selected
            .members()
            .iter()
            .filter(|member| member.length() > u64::from(page_bytes))
            .any(|member| {
                let bytes = records.read_member(anchor, *member, 1 << 30).unwrap();
                bytes
                    .strip_prefix(b"WORTH-REL-REG6\n")
                    .is_some_and(|header| header[2..6] == 24_u32.to_be_bytes() && header[38] == 0)
            }),
        "the healthy region's head must occupy an extent, separate from the corrupt inline frame"
    );
    home.close();
    // Damage only materialized data, leaving the canonical WAL frames intact.
    // Corrupting every occurrence would test WAL admission before any cold read.
    let flip_data = |pattern: &[u8]| -> usize {
        std::fs::read_dir(location.join("families"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap() != "wal")
            .map(|path| open_refusals::flip_after(&path, pattern, pattern.len() - 1, &|_| true))
            .sum()
    };
    let flipped = flip_data(&bytes);
    assert!(
        flipped > 0,
        "the selected member's physical bytes were damaged"
    );
    let home = durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    let first = read(&home, &fixture.records[0], &execution).unwrap_err();
    let RelationalTransactionReadDenial::StorageUnavailable(failure) = first else {
        panic!("{first:?}")
    };
    assert_eq!(
        failure.subject,
        RelationalStorageSubject::Region(worth_relational::facade::identity::PartitionId(1))
    );
    assert!(
        matches!(
            failure.cause,
            RelationalStorageUnavailableCause::Damaged
                | RelationalStorageUnavailableCause::IntegrityMismatch
        ),
        "{failure:?}"
    );
    let before = home.cold_storage_reads();
    assert_eq!(
        read(&home, &fixture.records[0], &execution),
        Err(first.clone())
    );
    assert_eq!(home.cold_storage_reads(), before);
    // Detecting corrupt record bytes locks the whole Store home for inspection
    // until reopen. Relational damages only partition 1; partition 24 retries
    // the port but cannot serve while this Store session is locked out.
    assert!(matches!(
        home.certification_serving().anchored_records(),
        Err(AnchoredRecordsDenial::ServingRequiresInspection)
    ));
    let other = read(&home, &fixture.records[23], &execution).unwrap_err();
    let RelationalTransactionReadDenial::StorageUnavailable(unavailable) = &other else {
        panic!("{other:?}")
    };
    assert_eq!(
        unavailable.subject,
        RelationalStorageSubject::Region(worth_relational::facade::identity::PartitionId(24))
    );
    assert_eq!(
        unavailable.cause,
        RelationalStorageUnavailableCause::SourceUnavailable
    );
    let retryable = home.cold_storage_reads();
    assert_eq!(read(&home, &fixture.records[23], &execution), Err(other));
    assert!(
        home.cold_storage_reads().member_reads > retryable.member_reads,
        "another subject must retry the port instead of becoming damaged"
    );
    home.close();
    let home = durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    assert!(home.certification_serving().anchored_records().is_ok());
    assert_eq!(
        workload::states_of(home.runtime(), &fixture.records[23..24]),
        workload::states_of(&fixture.reference, &fixture.records[23..24]),
        "reopen serves partition 24 before the corrupt subject is touched"
    );
    assert_eq!(read(&home, &fixture.records[0], &execution), Err(first));
    assert!(matches!(
        home.certification_serving().anchored_records(),
        Err(AnchoredRecordsDenial::ServingRequiresInspection)
    ));
    home.close();
    let mut damaged = bytes.clone();
    *damaged.last_mut().unwrap() ^= 0x5a;
    assert_eq!(
        flip_data(&damaged),
        flipped,
        "repair exactly the damaged copies"
    );
    let home = durable_run::open(&location, &execution).unwrap();
    fixture.assert_reads(&home, &execution);
    home.close();
}

/// A record-checksum fault affects its physical frame. Put the healthy region's
/// head in an extent so its own bytes remain intact when region 1 is damaged.
fn isolate_healthy_region(fixture: &mut Fixture, execution: &ExecutionRequest<'_, '_>) {
    use worth_relational::facade::{
        history::BranchId,
        identity::PartitionId,
        transactions::{CreateIntent, MutationIntent, WorkerIntentBatch},
    };
    let mut batch = WorkerIntentBatch::new("extent-backed healthy subject");
    for index in 10_000..10_512 {
        let mut create = super::batches::create(index);
        let MutationIntent::Create(CreateIntent::Entity(spec)) = &mut create.intents[0] else {
            panic!("entity fixture")
        };
        spec.partition_id = PartitionId(24);
        batch.intents.extend(create.intents);
    }
    let branch = BranchId("main".into());
    let outcome = workload::commit(&fixture.reference, execution, &branch, batch.clone()).unwrap();
    fixture
        .records
        .extend(outcome.changed_records.iter().cloned());
    fixture.steps.insert(
        fixture.steps.len() - 1,
        workload::Step::Commit { branch, batch },
    );
}

/// Inspect the producer's framed member header, never a guessed locator.
pub(super) fn region_head(
    home: &RelationalStoreHome,
) -> (
    worth_store::physical_runtime::AnchoredRecordMemberLocator,
    Vec<u8>,
) {
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor = AnchoredRecordName::new(selected_checkpoint::ANCHOR).unwrap();
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
