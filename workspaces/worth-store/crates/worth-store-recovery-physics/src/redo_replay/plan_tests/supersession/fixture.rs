//! Canonically encoded append images; every test crosses real WAL/projection admission.
use super::super::*;
use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    append_inline_records_owned, encode_data_frame_page_lsn, DurableFrameKind,
    DurableInlineRecordPlacement, InlineRecordAppend, PersistedInlineSegmentAllocation,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryRootState, PersistedRecordIdentity,
    PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId, PhysicalPageLsn,
    PhysicalRecordSlot, PhysicalSegmentId, RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};
use worth_store_wal::LogSequenceNumber;

pub(super) fn admitted_images(
    members: Vec<PhysicalRedoMemberInput>,
) -> AdmittedPhysicalRedoMembers {
    admit_physical_redo_members(
        members,
        test_store(),
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
        PhysicalRedoAdmissionLimits {
            recovery_memory_bytes: u64::MAX,
            targets: 64,
            distinct_targets: 64,
            projection: PhysicalRecoveryProjectionDecodeLimits {
                frames: 64,
                record_identities: 64,
                placements: 64,
                segment_updates: 64,
                manifests: 64,
                total_entries: 192,
                inline_allocations: 64,
            },
        },
    )
    .expect("canonical image members cross real WAL/projection admission")
}

pub(super) fn image(
    generation: u64,
    root_generation: u64,
    first_lsn: u64,
    total_records: u32,
    appended_records: u32,
    change_first_payload: bool,
) -> (PhysicalRedoMemberInput, RecoveryPageObservation) {
    image_generations(
        (generation, generation, root_generation),
        first_lsn,
        total_records,
        appended_records,
        change_first_payload,
    )
}

pub(super) fn image_generations(
    (generation, artifact_generation, root_generation): (u64, u64, u64),
    first_lsn: u64,
    total_records: u32,
    appended_records: u32,
    change_first_payload: bool,
) -> (PhysicalRedoMemberInput, RecoveryPageObservation) {
    let format = PhysicalRecordFormatDeclaration::builder().admit().unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let segment_cell = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(artifact_generation).unwrap());
    let page = authority
        .page_cell(segment, PhysicalPageId::from_raw(2).unwrap())
        .with_page_generation(PhysicalGeneration::from_raw(generation).unwrap());
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation: artifact_generation,
        },
        0,
        frame_len(),
    )
    .unwrap();
    let records = (1..=total_records)
        .map(|ordinal| PersistedRecordIdentity::new([1; 16], u64::from(ordinal)).unwrap())
        .collect::<Vec<_>>();
    let slots = (1..=total_records)
        .map(|ordinal| {
            authority
                .slot_cell(
                    segment,
                    page.page_id(),
                    PhysicalRecordSlot::from_raw(ordinal as u16).unwrap(),
                )
                .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap())
        })
        .collect::<Vec<_>>();
    let payloads = (0..total_records)
        .map(|index| {
            vec![
                if index == 0 && change_first_payload {
                    0xFE
                } else {
                    index as u8 + 1
                };
                12
            ]
        })
        .collect::<Vec<_>>();
    let appends = records
        .iter()
        .zip(&slots)
        .zip(&payloads)
        .map(|((&record, &slot), payload)| InlineRecordAppend::new(record, slot, payload))
        .collect::<Vec<_>>();
    let mut bytes = append_inline_records_owned(format, page, None, &appends)
        .unwrap()
        .0;
    let last_lsn = first_lsn + u64::from(appended_records) - 1;
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(last_lsn),
    )
    .unwrap();
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    let placements = records
        .iter()
        .zip(&slots)
        .map(|(&record, &slot)| {
            CurrentPhysicalRecordPlacement::Inline(
                DurableInlineRecordPlacement::new(record, segment_cell, page, slot, 4, 12).unwrap(),
            )
        })
        .collect();
    let first_new = (total_records - appended_records) as usize;
    let root = PersistedPhysicalRecoveryRootState::new(
        4096,
        1,
        4,
        vec![PersistedInlineSegmentAllocation::new(segment_cell, 4, 1).unwrap()],
        records.last().copied(),
        Some(segment_cell),
    )
    .unwrap();
    let projection = PersistedPhysicalRecoveryProjection::new(
        root_generation,
        root,
        records[first_new..].to_vec(),
        vec![PersistedPhysicalRecoveryFrame::new(
            PersistedPhysicalDataFrameSubject::InlinePage(page),
            coordinate,
            &bytes,
        )
        .unwrap()],
        placements,
        vec![RecordSegmentPageManifestEntry::new(page, segment_cell, 1, 0).unwrap()],
        Vec::new(),
    )
    .unwrap();
    let target = canonical_target_bytes_with_generations(generation, artifact_generation);
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&u64::from(appended_records).to_le_bytes());
    for ordinal in 0..appended_records {
        encoded.extend_from_slice(&ordinal.to_le_bytes());
        encoded.extend_from_slice(&(first_lsn + u64::from(ordinal)).to_le_bytes());
        encoded.extend_from_slice(&1_u64.to_le_bytes());
        field(&mut encoded, &target);
        encoded.extend_from_slice(&digest);
        field(&mut encoded, &payloads[first_new + ordinal as usize]);
    }
    field(&mut encoded, &projection.encode());
    let input = PhysicalRedoMemberInput::new(
        WalLsnRange::new(
            LogSequenceNumber::new(first_lsn),
            LogSequenceNumber::new(last_lsn + 1),
        )
        .unwrap(),
        [first_lsn as u8; 32],
        RecoveryOperationFate::Indeterminate,
        &encoded,
    );
    (
        input,
        RecoveryPageObservation::materialized(
            PhysicalRedoTargetIdentity::InlinePage {
                segment: 1,
                page: 2,
                generation,
            },
            last_lsn,
            digest,
            coordinate,
            [3; 32],
        ),
    )
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
