use sha2::{Digest, Sha256};
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StableStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    append_inline_records_owned, encode_data_frame_page_lsn, CurrentPhysicalRecordPlacement,
    DurableFrameKind, DurableInlineRecordPlacement, InlineRecordAppend,
    PersistedInlineSegmentAllocation, PersistedPhysicalDataFrameSubject,
    PersistedPhysicalRecoveryFrame, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalPageLsn, PhysicalRecordFormatDeclaration,
    PhysicalRecordSlot, RecordArtifactFile, RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    plan_physical_redo, ImmutablePhysicalRedoPlan, PhysicalRedoMemberInput,
    PhysicalRedoPlanningDenial, PhysicalRedoTargetIdentity, RecoveryOperationFate,
    RecoveryPageObservation,
};
use worth_store_wal::{LogSequenceNumber, WalLsnRange};

const OPERATION: [u8; 32] = [1; 32];

pub(super) fn applied_plan() -> ImmutablePhysicalRedoPlan {
    plan_physical_redo(
        vec![PhysicalRedoMemberInput::new(
            range(),
            OPERATION,
            RecoveryOperationFate::Indeterminate,
            &encoded_redo(),
        )],
        vec![observation(1, 9, [0; 32])],
        1,
        store(),
    )
    .expect("canonical redo fixture applies after the prior page observation")
}

pub(super) fn skipped_plan() -> ImmutablePhysicalRedoPlan {
    plan_physical_redo(
        vec![PhysicalRedoMemberInput::new(
            range(),
            OPERATION,
            RecoveryOperationFate::Indeterminate,
            &encoded_redo(),
        )],
        vec![observation(2, 10, result_digest())],
        1,
        store(),
    )
    .expect("canonical redo fixture skips an already materialized page")
}

pub(super) fn generation_denial() -> PhysicalRedoPlanningDenial {
    plan_physical_redo(
        vec![PhysicalRedoMemberInput::new(
            range(),
            OPERATION,
            RecoveryOperationFate::Indeterminate,
            &encoded_redo(),
        )],
        vec![observation(10, 10, [0; 32])],
        1,
        store(),
    )
    .expect_err("foreign page generation must be rejected before replay")
}

fn store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity()
}

fn range() -> WalLsnRange {
    WalLsnRange::new(LogSequenceNumber::new(10), LogSequenceNumber::new(11))
        .expect("fixture range is non-empty")
}

fn observation(generation: u64, page_lsn: u64, digest: [u8; 32]) -> RecoveryPageObservation {
    RecoveryPageObservation::materialized(
        target(generation),
        page_lsn,
        digest,
        RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation,
            },
            0,
            frame_len(),
        )
        .expect("fixture frame coordinate is valid"),
        [3; 32],
    )
}

fn target(generation: u64) -> PhysicalRedoTargetIdentity {
    PhysicalRedoTargetIdentity::InlinePage {
        segment: 1,
        page: 2,
        generation,
    }
}

fn encoded_redo() -> Vec<u8> {
    let target = canonical_target_bytes();
    let projection = projection().encode();
    let mut encoded = Vec::new();
    field(&mut encoded, b"store.physical.wal.canonical-redo.v3");
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    encoded.extend_from_slice(&0_u32.to_le_bytes());
    encoded.extend_from_slice(&10_u64.to_le_bytes());
    encoded.extend_from_slice(&1_u64.to_le_bytes());
    field(&mut encoded, &target);
    encoded.extend_from_slice(&result_digest());
    field(&mut encoded, b"redo-record");
    field(&mut encoded, &projection);
    encoded
}

fn canonical_target_bytes() -> Vec<u8> {
    let mut target = Vec::new();
    target.push(1);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.push(5);
    target.extend_from_slice(&1_u64.to_le_bytes());
    target.extend_from_slice(&2_u64.to_le_bytes());
    target.extend_from_slice(&0_u64.to_le_bytes());
    target.extend_from_slice(&frame_len().to_le_bytes());
    target
}

fn projection() -> PersistedPhysicalRecoveryProjection {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(1)
        .expect("fixture segment is nonzero");
    let page = authority
        .page_cell(
            segment,
            PhysicalPageId::from_raw(2).expect("fixture page is nonzero"),
        )
        .with_page_generation(
            PhysicalGeneration::from_raw(2).expect("fixture generation is nonzero"),
        );
    let coordinate = RecordFrameCoordinate::new(
        RecordArtifactFile::Segment {
            segment: 1,
            generation: 2,
        },
        0,
        frame_len(),
    )
    .expect("fixture frame coordinate is valid");
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page),
        coordinate,
        &result_bytes(),
    )
    .expect("fixture recovery frame is valid");
    let record = PersistedRecordIdentity::new([1; 16], 1).expect("fixture record is valid");
    let slot = authority
        .slot_cell(
            segment,
            page.page_id(),
            PhysicalRecordSlot::from_raw(1).expect("fixture slot is nonzero"),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).expect("fixture slot generation"));
    let segment_cell = authority.segment_cell(segment).with_segment_generation(
        PhysicalGeneration::from_raw(2).expect("fixture segment generation"),
    );
    let placement = DurableInlineRecordPlacement::new(
        record,
        segment_cell,
        page,
        slot,
        1,
        b"redo-record".len() as u64,
    )
    .expect("fixture placement is valid");
    let routing = RecordSegmentPageManifestEntry::new(page, segment_cell, 1, 0)
        .expect("fixture routing entry is valid");
    let root_state = PersistedPhysicalRecoveryRootState::new(
        4096,
        1,
        4,
        vec![PersistedInlineSegmentAllocation::new(segment_cell, 1, 1)
            .expect("fixture allocation is valid")],
        Some(record),
        Some(segment_cell),
    )
    .expect("fixture root state is valid");
    PersistedPhysicalRecoveryProjection::new(
        1,
        root_state,
        vec![record],
        vec![frame],
        vec![CurrentPhysicalRecordPlacement::Inline(placement)],
        vec![routing],
        Vec::new(),
    )
    .expect("fixture projection is valid")
}

fn result_bytes() -> Vec<u8> {
    let format = PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("fixture format is valid");
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = worth_store_physical_format::PhysicalSegmentId::from_raw(1)
        .expect("fixture segment is nonzero");
    let page = authority
        .page_cell(
            segment,
            PhysicalPageId::from_raw(2).expect("fixture page is nonzero"),
        )
        .with_page_generation(
            PhysicalGeneration::from_raw(2).expect("fixture generation is nonzero"),
        );
    let record = PersistedRecordIdentity::new([1; 16], 1).expect("fixture record is valid");
    let slot = authority
        .slot_cell(
            segment,
            page.page_id(),
            PhysicalRecordSlot::from_raw(1).expect("fixture slot is nonzero"),
        )
        .with_slot_generation(PhysicalGeneration::from_raw(1).expect("fixture slot generation"));
    let mut bytes = append_inline_records_owned(
        format,
        page,
        None,
        &[InlineRecordAppend::new(record, slot, b"redo-record")],
    )
    .expect("fixture inline page is valid")
    .0;
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(10),
    )
    .expect("fixture page LSN is encodable");
    bytes
}

fn result_digest() -> [u8; 32] {
    Sha256::digest(result_bytes()).into()
}

fn frame_len() -> u32 {
    PhysicalRecordFormatDeclaration::builder()
        .admit()
        .expect("fixture format is valid")
        .page_size()
        .bytes()
}

fn field(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
    target.extend_from_slice(bytes);
}
