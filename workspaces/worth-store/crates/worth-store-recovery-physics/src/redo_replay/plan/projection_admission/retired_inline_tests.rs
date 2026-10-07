use super::*;
use worth_store_physical_format::store_namespace::{
    ProposedStoreIdentity, StoreNamespaceIdentityRecord, StoreNamespaceVersion,
};
use worth_store_physical_format::{
    encode_data_frame_page_lsn, encode_inline_page, DurableFrameKind, DurableInlineRecordPlacement,
    InlineRecordAppend, PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId,
    RecordArtifactFile, RecordFrameCoordinate,
};

#[test]
fn retired_inline_slot_needs_exact_wal_witness_even_when_not_new_record() {
    let full = inline_with_retired_slot(0);
    let admitted = admit_projection(&full, store(), format()).unwrap();
    assert_eq!(admitted.inline_frames().len(), 1);
    assert_eq!(admitted.inline_frames()[0].records.len(), 5);
    assert_eq!(full.record_identities().len(), 1);
    assert_eq!(
        full.placements().len() - 1,
        4,
        "retired witness is not a live route"
    );
    for mutation in [1, 2, 3] {
        let corrupted = inline_with_retired_slot(mutation);
        assert!(
            matches!(
                admit_projection(&corrupted, store(), format()),
                Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection)
            ),
            "mutation {mutation} must deny"
        );
    }
}

// 1 omits the physical retired slot, 2 forges its slot coordinate, and
// 3 adds a witness for a record absent from the intact physical page.
fn inline_with_retired_slot(mutation: u8) -> PersistedPhysicalRecoveryProjection {
    let format = format();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let generation = PhysicalGeneration::from_raw(4).unwrap();
    let segment = authority
        .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
        .with_segment_generation(generation);
    let page = authority
        .page_cell(
            PhysicalSegmentId::from_raw(1).unwrap(),
            PhysicalPageId::from_raw(1).unwrap(),
        )
        .with_page_generation(generation);
    let slot = |number| {
        authority
            .slot_cell(
                PhysicalSegmentId::from_raw(1).unwrap(),
                PhysicalPageId::from_raw(1).unwrap(),
                PhysicalRecordSlot::from_raw(number).unwrap(),
            )
            .with_slot_generation(generation)
    };
    let retired = PersistedRecordIdentity::new([4; 16], 1).unwrap();
    let carried = (2..=4)
        .map(|ordinal| PersistedRecordIdentity::new([4; 16], ordinal).unwrap())
        .collect::<Vec<_>>();
    let inserted = PersistedRecordIdentity::new([4; 16], 5).unwrap();
    let forged = PersistedRecordIdentity::new([4; 16], 6).unwrap();
    let mut bytes = encode_inline_page(
        format,
        page,
        &[
            InlineRecordAppend::new(retired, slot(1), b"old"),
            InlineRecordAppend::new(carried[0], slot(2), b"one"),
            InlineRecordAppend::new(carried[1], slot(3), b"two"),
            InlineRecordAppend::new(carried[2], slot(4), b"tri"),
            InlineRecordAppend::new(inserted, slot(5), b"new"),
        ],
    )
    .unwrap();
    encode_data_frame_page_lsn(
        &mut bytes,
        DurableFrameKind::InlinePage,
        PhysicalPageLsn::new(12),
    )
    .unwrap();
    let frame = PersistedPhysicalRecoveryFrame::new(
        PersistedPhysicalDataFrameSubject::InlinePage(page),
        RecordFrameCoordinate::new(
            RecordArtifactFile::Segment {
                segment: 1,
                generation: 4,
            },
            0,
            bytes.len() as u32,
        )
        .unwrap(),
        &bytes,
    )
    .unwrap();
    let placement = |record, slot_cell, length| {
        CurrentPhysicalRecordPlacement::Inline(
            DurableInlineRecordPlacement::legacy_unknown(
                record,
                segment,
                page,
                slot_cell,
                format.page_size().bytes(),
                length,
            )
            .unwrap(),
        )
    };
    let mut placements = Vec::new();
    if mutation != 1 {
        placements.push(placement(
            retired,
            slot(if mutation == 2 { 2 } else { 1 }),
            3,
        ));
    }
    for (number, record) in (2..=4).zip(carried) {
        placements.push(placement(record, slot(number), 3));
    }
    placements.push(placement(inserted, slot(5), 3));
    if mutation == 3 {
        placements.push(placement(forged, slot(6), 3));
    }
    PersistedPhysicalRecoveryProjection::new(
        3,
        PersistedPhysicalRecoveryRootState::new(4096, 1, 4, Vec::new(), None, None).unwrap(),
        vec![inserted],
        vec![frame],
        placements,
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn store() -> StableStoreIdentity {
    StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([0x51; 16]).unwrap(),
    )
    .published_identity()
}
