//! A checksum-valid Store record must still match its manifest's member digest.
pub(crate) use {crate::durable_run, crate::residency_fixture::Fixture, crate::residency_media, crate::selected_checkpoint, crate::workload};
pub(crate) use {std::path::Path};
pub(crate) use {worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::identity::PartitionId, worth_relational::facade::mvcc::RelationalTransactionReadDenial, worth_relational::facade::storage::RelationalStorageSubject, worth_relational::facade::storage::RelationalStorageUnavailableCause};
pub(crate) use {worth_store::physical_runtime::AnchoredRecordMemberDamage, worth_store::physical_runtime::AnchoredRecordMemberReadFailure, worth_store::physical_runtime::AnchoredRecordName, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits};
pub(crate) use {worth_store_physical_format::decode_data_frame_page_lsn, worth_store_physical_format::durable_artifact_checksum, worth_store_physical_format::DurableFrameKind, worth_store_physical_format::DURABLE_FRAME_HEADER_BYTES};

/// Offline fault construction, limited to data frames containing this member.
/// Preserve every frame coordinate and reseal its checksum with Store's codec.
pub(crate) fn substitute_in_data(path: &Path, original: &[u8], replacement: &[u8]) -> usize {
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
            3 => worth_store_physical_format::DurableFrameKind::InlinePage,
            4 => worth_store_physical_format::DurableFrameKind::Extent,
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