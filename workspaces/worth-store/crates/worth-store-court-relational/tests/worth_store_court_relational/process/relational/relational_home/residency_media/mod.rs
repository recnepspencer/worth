//! Ordinary home reads distinguish unavailable members from absent records.
#[allow(unused_imports)]
use crate::residency_media::*;

#[test]
fn missing_and_stale_member_locators_damage_only_their_region_and_never_answer_empty() {
    for fault in [
        crate::residency_media::DirectoryFault::Missing,
        crate::residency_media::DirectoryFault::Stale,
    ] {
        let host = crate::workload::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let fixture = crate::residency_fixture::Fixture::new(&execution);
        let directory = tempfile::tempdir().unwrap();
        let location = directory.path().join("home");
        let home = crate::durable_run::open(&location, &execution).unwrap();
        let stale = if matches!(fault, crate::residency_media::DirectoryFault::Stale) {
            crate::durable_run::apply(&home, &execution, &fixture.steps[..24]);
            home.checkpoint(&execution).unwrap();
            let earlier = region_head(&home).1;
            crate::durable_run::apply(&home, &execution, &fixture.steps[24..]);
            Some(earlier)
        } else {
            crate::durable_run::apply(&home, &execution, &fixture.steps);
            None
        };
        home.close();
        let home = crate::durable_run::open(&location, &execution).unwrap();
        assert_eq!(home.cold_storage_reads().member_reads, 0);
        let records = home.certification_serving().anchored_records().unwrap();
        let anchor = worth_store::physical_runtime::AnchoredRecordName::new(
            crate::selected_checkpoint::ANCHOR,
        )
        .unwrap();
        let selected = records.read(anchor, 1 << 30).unwrap().unwrap();
        let (target, bytes) = region_head(&home);
        let publish = |members| {
            records
                .publish(
                    worth_store::physical_runtime::AnchoredRecordPublication::new(
                        anchor,
                        crate::selected_checkpoint::generation(&home),
                        selected.bytes().to_vec(),
                        crate::open_refusals::home_placement(),
                        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                    )
                    .with_members(members),
                )
                .unwrap()
        };
        let mut members = selected
            .members()
            .iter()
            .filter(|member| **member != target)
            .map(
                |locator| worth_store::physical_runtime::AnchoredRecordMember::Carried {
                    locator: *locator,
                },
            )
            .collect::<Vec<_>>();
        if let Some(stale) = stale {
            // An actual earlier head from this partition cannot answer a
            // demand for the current manifest's locator.
            use {sha2::Digest, sha2::Sha256};
            assert_ne!(stale, bytes);
            members.push(worth_store::physical_runtime::AnchoredRecordMember::New {
                key: Sha256::digest(&stale).into(),
                bytes: stale,
            });
        }
        publish(members);
        let failure = read(&home, &fixture.records[0], &execution).unwrap_err();
        let worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(
            unavailable,
        ) = failure
        else {
            panic!("{failure:?}")
        };
        assert_eq!(
            unavailable.subject,
            worth_relational::facade::storage::RelationalStorageSubject::Region(
                worth_relational::facade::identity::PartitionId(1)
            )
        );
        assert_eq!(
            unavailable.cause,
            worth_relational::facade::storage::RelationalStorageUnavailableCause::MissingMember
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
            crate::workload::states_of(home.runtime(), &fixture.records[1..2]),
            crate::workload::states_of(&fixture.reference, &fixture.records[1..2]),
            "another cold subject still serves"
        );
        // Restore the directory through Store's publisher. The already damaged
        // cell stays damaged, while a fresh recovery serves the repaired home.
        let members = selected
            .members()
            .iter()
            .map(|locator| {
                if *locator == target {
                    worth_store::physical_runtime::AnchoredRecordMember::New {
                        key: target.sha256(),
                        bytes: bytes.clone(),
                    }
                } else {
                    worth_store::physical_runtime::AnchoredRecordMember::Carried {
                        locator: *locator,
                    }
                }
            })
            .collect();
        publish(members);
        let repaired = home.cold_storage_reads();
        assert_eq!(read(&home, &fixture.records[0], &execution), Err(failure));
        assert_eq!(home.cold_storage_reads(), repaired);
        home.close();
        let home = crate::durable_run::open(&location, &execution).unwrap();
        fixture.assert_reads(&home, &execution);
        home.close();
    }
}

#[test]
fn a_damaged_member_in_store_files_is_a_sticky_typed_refusal_after_reopen() {
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let mut fixture = crate::residency_fixture::Fixture::new(&execution);
    isolate_healthy_region(&mut fixture, &execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = crate::durable_run::open(&location, &execution).unwrap();
    crate::durable_run::apply(&home, &execution, &fixture.steps);
    let bytes = region_head(&home).1;
    let page_bytes = worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap()
        .page_size()
        .bytes();
    let records = home.certification_serving().anchored_records().unwrap();
    let anchor =
        worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR)
            .unwrap();
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
            .map(|path| {
                crate::open_refusals::flip_after(&path, pattern, pattern.len() - 1, &|_| true)
            })
            .sum()
    };
    let flipped = flip_data(&bytes);
    assert!(
        flipped > 0,
        "the selected member's physical bytes were damaged"
    );
    let home = crate::durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    let first = read(&home, &fixture.records[0], &execution).unwrap_err();
    let worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(
        failure,
    ) = first
    else {
        panic!("{first:?}")
    };
    assert_eq!(
        failure.subject,
        worth_relational::facade::storage::RelationalStorageSubject::Region(
            worth_relational::facade::identity::PartitionId(1)
        )
    );
    assert!(
        matches!(
            failure.cause,
            worth_relational::facade::storage::RelationalStorageUnavailableCause::Damaged
                | worth_relational::facade::storage::RelationalStorageUnavailableCause::IntegrityMismatch
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
        Err(worth_store::physical_runtime::AnchoredRecordsDenial::ServingRequiresInspection)
    ));
    let other = read(&home, &fixture.records[23], &execution).unwrap_err();
    let worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(
        unavailable,
    ) = &other
    else {
        panic!("{other:?}")
    };
    assert_eq!(
        unavailable.subject,
        worth_relational::facade::storage::RelationalStorageSubject::Region(
            worth_relational::facade::identity::PartitionId(24)
        )
    );
    assert_eq!(
        unavailable.cause,
        worth_relational::facade::storage::RelationalStorageUnavailableCause::SourceUnavailable
    );
    let retryable = home.cold_storage_reads();
    assert_eq!(read(&home, &fixture.records[23], &execution), Err(other));
    assert!(
        home.cold_storage_reads().member_reads > retryable.member_reads,
        "another subject must retry the port instead of becoming damaged"
    );
    home.close();
    let home = crate::durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);
    assert!(home.certification_serving().anchored_records().is_ok());
    assert_eq!(
        crate::workload::states_of(home.runtime(), &fixture.records[23..24]),
        crate::workload::states_of(&fixture.reference, &fixture.records[23..24]),
        "reopen serves partition 24 before the corrupt subject is touched"
    );
    assert_eq!(read(&home, &fixture.records[0], &execution), Err(first));
    assert!(matches!(
        home.certification_serving().anchored_records(),
        Err(worth_store::physical_runtime::AnchoredRecordsDenial::ServingRequiresInspection)
    ));
    home.close();
    let mut damaged = bytes.clone();
    *damaged.last_mut().unwrap() ^= 0x5a;
    assert_eq!(
        flip_data(&damaged),
        flipped,
        "repair exactly the damaged copies"
    );
    let home = crate::durable_run::open(&location, &execution).unwrap();
    fixture.assert_reads(&home, &execution);
    home.close();
}
