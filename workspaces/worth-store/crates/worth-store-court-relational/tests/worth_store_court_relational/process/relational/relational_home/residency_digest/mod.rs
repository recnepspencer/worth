//! A checksum-valid Store record must still match its manifest's member digest.
#[allow(unused_imports)] use crate::residency_digest::*;
use {crate::durable_run, crate::residency_fixture::Fixture, crate::residency_media, crate::selected_checkpoint, crate::workload};
use {std::path::Path};
use {worth_execution::ExecutionRequest};
use {worth_relational::facade::identity::PartitionId, worth_relational::facade::mvcc::RelationalTransactionReadDenial, worth_relational::facade::storage::RelationalStorageSubject, worth_relational::facade::storage::RelationalStorageUnavailableCause};
use {worth_store::physical_runtime::AnchoredRecordMemberDamage, worth_store::physical_runtime::AnchoredRecordMemberReadFailure, worth_store::physical_runtime::AnchoredRecordName, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};
use {worth_store_physical_format::decode_data_frame_page_lsn, worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES};

#[test]
fn a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky() {
    let host = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    let fixture = crate::residency_fixture::Fixture::new(&execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = crate::durable_run::open(&location, &execution).unwrap();
    crate::durable_run::apply(&home, &execution, &fixture.steps);
    let (locator, original) = crate::residency_media::region_head(&home);
    home.close();
    let mut substituted = original.clone();
    // Change the region's version, preserving the physical record identity,
    // length and geometry. Store's checksums are resealed below; the manifest
    // leaf and its content-addressed locator deliberately remain unchanged.
    substituted[b"WORTH-REL-REG6\n".len() + 1] ^= 1;
    let copies = substitute_in_data(&location.join("families"), &original, &substituted);
    assert!(
        copies > 0,
        "substitute actual producer-written member bytes"
    );
    let home = crate::durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);

    // The ordinary physical reader accepts identity, geometry and checksums.
    // This prevents a Store record-checksum refusal from counterfeiting the
    // intended member-digest refusal.
    let reader = home.certification_serving().records().unwrap();
    let mut session = reader
        .open(
            locator.record(),
            worth_store::physical_runtime::RecordReadLimits::new(worth_store::physical_runtime::RecordByteLimit::new(u32::MAX).unwrap()),
        )
        .unwrap();
    let mut accepted = vec![0; substituted.len()];
    let mut filled = 0;
    while filled < accepted.len() {
        let count = session.read_next(&mut accepted[filled..]).unwrap();
        assert!(count > 0);
        filled += count;
    }
    assert_eq!(session.read_next(&mut [0; 1]).unwrap(), 0);
    assert_eq!(accepted, substituted);
    drop(session);
    drop(reader);
    assert!(matches!(
        home.certification_serving()
            .anchored_records()
            .unwrap()
            .read_member(
                worth_store::physical_runtime::AnchoredRecordName::new(crate::selected_checkpoint::ANCHOR).unwrap(),
                locator,
                1 << 30
            ),
        Err(worth_store::physical_runtime::AnchoredRecordMemberReadFailure::Damaged(
            worth_store::physical_runtime::AnchoredRecordMemberDamage::DigestMismatch
        ))
    ));
    let refusal = crate::residency_media::read(&home, &fixture.records[0], &execution).unwrap_err();
    let worth_relational::facade::mvcc::RelationalTransactionReadDenial::StorageUnavailable(unavailable) = &refusal else {
        panic!("{refusal:?}")
    };
    assert_eq!(
        unavailable.subject,
        worth_relational::facade::storage::RelationalStorageSubject::Region(PartitionId(1))
    );
    assert_eq!(
        unavailable.cause,
        worth_relational::facade::storage::RelationalStorageUnavailableCause::IntegrityMismatch
    );
    let damaged = home.cold_storage_reads();
    assert_eq!(
        crate::residency_media::read(&home, &fixture.records[0], &execution),
        Err(refusal)
    );
    assert_eq!(home.cold_storage_reads(), damaged);
    assert_eq!(
        crate::workload::states_of(home.runtime(), &fixture.records[23..24]),
        crate::workload::states_of(&fixture.reference, &fixture.records[23..24]),
        "member-digest rejection does not damage another region"
    );
    home.close();
    assert_eq!(
        substitute_in_data(&location.join("families"), &substituted, &original),
        copies
    );
    let home = crate::durable_run::open(&location, &execution).unwrap();
    fixture.assert_reads(&home, &execution);
    home.close();
}
