//! A checksum-valid Store record must still match its manifest's member digest.
use super::{
    durable_run, residency_fixture::Fixture, residency_media, selected_checkpoint, workload,
};
use std::path::Path;
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    identity::PartitionId,
    mvcc::RelationalTransactionReadDenial,
    storage::{RelationalStorageSubject, RelationalStorageUnavailableCause},
};
use worth_store::physical_runtime::{
    AnchoredRecordMemberDamage, AnchoredRecordMemberReadFailure, AnchoredRecordName,
    RecordByteLimit, RecordReadLimits,
};
use worth_store_physical_format::{
    decode_data_frame_page_lsn, durable_artifact_checksum, DurableFrameKind,
    DURABLE_FRAME_HEADER_BYTES,
};

#[test]
fn a_checksum_valid_substituted_member_refuses_its_digest_and_keeps_subject_damage_sticky() {
    let host = workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let fixture = Fixture::new(&execution);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = durable_run::open(&location, &execution).unwrap();
    durable_run::apply(&home, &execution, &fixture.steps);
    let (locator, original) = residency_media::region_head(&home);
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
    let home = durable_run::open(&location, &execution).unwrap();
    assert_eq!(home.cold_storage_reads().member_reads, 0);

    // The ordinary physical reader accepts identity, geometry and checksums.
    // This prevents a Store record-checksum refusal from counterfeiting the
    // intended member-digest refusal.
    let reader = home.certification_serving().records().unwrap();
    let mut session = reader
        .open(
            locator.record(),
            RecordReadLimits::new(RecordByteLimit::new(u32::MAX).unwrap()),
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
                AnchoredRecordName::new(selected_checkpoint::ANCHOR).unwrap(),
                locator,
                1 << 30
            ),
        Err(AnchoredRecordMemberReadFailure::Damaged(
            AnchoredRecordMemberDamage::DigestMismatch
        ))
    ));
    let refusal = residency_media::read(&home, &fixture.records[0], &execution).unwrap_err();
    let RelationalTransactionReadDenial::StorageUnavailable(unavailable) = &refusal else {
        panic!("{refusal:?}")
    };
    assert_eq!(
        unavailable.subject,
        RelationalStorageSubject::Region(PartitionId(1))
    );
    assert_eq!(
        unavailable.cause,
        RelationalStorageUnavailableCause::IntegrityMismatch
    );
    let damaged = home.cold_storage_reads();
    assert_eq!(
        residency_media::read(&home, &fixture.records[0], &execution),
        Err(refusal)
    );
    assert_eq!(home.cold_storage_reads(), damaged);
    assert_eq!(
        workload::states_of(home.runtime(), &fixture.records[23..24]),
        workload::states_of(&fixture.reference, &fixture.records[23..24]),
        "member-digest rejection does not damage another region"
    );
    home.close();
    assert_eq!(
        substitute_in_data(&location.join("families"), &substituted, &original),
        copies
    );
    let home = durable_run::open(&location, &execution).unwrap();
    fixture.assert_reads(&home, &execution);
    home.close();
}

/// Offline fault construction, limited to data frames containing this member.
/// Preserve every frame coordinate and reseal its checksum with Store's codec.
fn substitute_in_data(path: &Path, original: &[u8], replacement: &[u8]) -> usize {
    assert_eq!(original.len(), replacement.len());
    if path.is_dir() {
        return std::fs::read_dir(path)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.file_name().unwrap() != "wal")
            .map(|path| substitute_in_data(&path, original, replacement))
            .sum();
    }
    let mut bytes = std::fs::read(path).unwrap();
    let occurrences: Vec<_> = bytes
        .windows(original.len())
        .enumerate()
        .filter_map(|(at, found)| (found == original).then_some(at))
        .collect();
    for at in &occurrences {
        let frame_at = bytes[..*at]
            .windows(8)
            .rposition(|magic| magic == b"WRC5FRM\0")
            .expect("member belongs to a physical frame");
        let frame_len = DURABLE_FRAME_HEADER_BYTES
            + u32::from_le_bytes(bytes[frame_at + 24..frame_at + 28].try_into().unwrap()) as usize;
        assert!(*at + original.len() <= frame_at + frame_len);
        let frame = &mut bytes[frame_at..frame_at + frame_len];
        let kind = match frame[8] {
            3 => DurableFrameKind::InlinePage,
            4 => DurableFrameKind::Extent,
            other => panic!("member inside non-data frame {other}"),
        };
        let lsn = decode_data_frame_page_lsn(frame, kind).unwrap();
        let offset = *at - frame_at;
        frame[offset..offset + replacement.len()].copy_from_slice(replacement);
        // The durable-frame format excludes its four checksum bytes.
        let covered = [&frame[..44], &frame[DURABLE_FRAME_HEADER_BYTES..]].concat();
        frame[44..DURABLE_FRAME_HEADER_BYTES]
            .copy_from_slice(&durable_artifact_checksum(&covered).to_le_bytes());
        assert_eq!(decode_data_frame_page_lsn(frame, kind).unwrap(), lsn);
    }
    if !occurrences.is_empty() {
        std::fs::write(path, bytes).unwrap();
    }
    occurrences.len()
}
