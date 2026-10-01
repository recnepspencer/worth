use super::*;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableInlineRecordPlacement,
    PersistedPhysicalDataFrameSubject, PersistedPhysicalRecoveryFrame,
    PersistedPhysicalRecoveryManifest, PersistedPhysicalRecoveryProjection,
    PersistedPhysicalRecoveryRootState, PersistedRecordIdentity, PhysicalGeneration,
    PhysicalGenerationAuthority, PhysicalPageId, PhysicalRecordSlot, PhysicalSegmentId,
    RecordArtifactFile, RecordFrameCoordinate, RecordSegmentPageManifestEntry,
};

#[test]
fn projection_omission_and_foreign_entry_mutants_are_rejected() {
    for mutation in [
        ProjectionMutation::MissingFrame,
        ProjectionMutation::ForeignFrame,
        ProjectionMutation::ForeignArtifactGeneration,
        ProjectionMutation::ForeignArtifactOffset,
        ProjectionMutation::ForeignArtifactLength,
        ProjectionMutation::ExtraPlacement,
        ProjectionMutation::ExtraSegmentUpdate,
        ProjectionMutation::ExtraManifest,
        ProjectionMutation::MissingInlineAllocation,
    ] {
        let projection = mutated_projection_bytes(mutation);
        let member = PhysicalRedoMemberInput::new(
            range(),
            [1; 32],
            RecoveryOperationFate::Indeterminate,
            &encoded_redo_with_projection_bytes(&canonical_target_bytes(), &projection),
        );
        assert_eq!(
            plan_physical_redo(vec![member], vec![observation(1, 9, [0; 32])], 1),
            Err(PhysicalRedoPlanningDenial::InvalidRecoveryProjection),
            "mutation {mutation:?} must not self-certify",
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionMutation {
    MissingFrame,
    ForeignFrame,
    ForeignArtifactGeneration,
    ForeignArtifactOffset,
    ForeignArtifactLength,
    ExtraPlacement,
    ExtraSegmentUpdate,
    ExtraManifest,
    MissingInlineAllocation,
}

fn mutated_projection_bytes(mutation: ProjectionMutation) -> Vec<u8> {
    let base = projection_with_segment_page_count(1);
    if mutation == ProjectionMutation::MissingFrame {
        return projection_without_first_frame(&base);
    }
    if matches!(
        mutation,
        ProjectionMutation::ForeignArtifactGeneration
            | ProjectionMutation::ForeignArtifactOffset
            | ProjectionMutation::ForeignArtifactLength
    ) {
        return projection_with_foreign_frame_coordinate(&base, mutation);
    }
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let segment_cell = authority
        .segment_cell(segment)
        .with_segment_generation(PhysicalGeneration::from_raw(2).unwrap());
    let mut frames = base.frames().expect("ordinary frame fixture").to_vec();
    let mut records = base.record_identities().to_vec();
    let mut placements = base.placements().to_vec();
    let mut updates = base.segment_updates().to_vec();
    let mut manifests = base.manifests().to_vec();
    let mut root_state = base.root_state().clone();

    match mutation {
        ProjectionMutation::MissingFrame => unreachable!(),
        ProjectionMutation::ForeignArtifactGeneration
        | ProjectionMutation::ForeignArtifactOffset
        | ProjectionMutation::ForeignArtifactLength => {
            unreachable!()
        }
        ProjectionMutation::ForeignFrame => {
            let page = authority
                .page_cell(segment, PhysicalPageId::from_raw(3).unwrap())
                .with_page_generation(PhysicalGeneration::from_raw(2).unwrap());
            frames[0] = PersistedPhysicalRecoveryFrame::new(
                PersistedPhysicalDataFrameSubject::InlinePage(page),
                RecordFrameCoordinate::new(
                    RecordArtifactFile::Segment {
                        segment: 1,
                        generation: 2,
                    },
                    0,
                    frame_len(),
                )
                .unwrap(),
                &result_bytes(),
            )
            .unwrap();
        }
        ProjectionMutation::ExtraPlacement => {
            let record = PersistedRecordIdentity::new([2; 16], 1).unwrap();
            records.push(record);
            let page = authority
                .page_cell(segment, PhysicalPageId::from_raw(2).unwrap())
                .with_page_generation(PhysicalGeneration::from_raw(2).unwrap());
            let slot = authority
                .slot_cell(
                    segment,
                    page.page_id(),
                    PhysicalRecordSlot::from_raw(2).unwrap(),
                )
                .with_slot_generation(PhysicalGeneration::from_raw(1).unwrap());
            placements.push(CurrentPhysicalRecordPlacement::Inline(
                DurableInlineRecordPlacement::legacy_unknown(
                    record,
                    segment_cell,
                    page,
                    slot,
                    2,
                    1,
                )
                .unwrap(),
            ));
        }
        ProjectionMutation::ExtraSegmentUpdate => {
            let page = authority
                .page_cell(segment, PhysicalPageId::from_raw(3).unwrap())
                .with_page_generation(PhysicalGeneration::from_raw(2).unwrap());
            updates.push(RecordSegmentPageManifestEntry::new(page, segment_cell, 2, 1).unwrap());
        }
        ProjectionMutation::ExtraManifest => manifests.push(
            PersistedPhysicalRecoveryManifest::new(
                RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 7 }, 4096, 16)
                    .unwrap(),
                b"foreign-manifest",
            )
            .unwrap(),
        ),
        ProjectionMutation::MissingInlineAllocation => {
            root_state = PersistedPhysicalRecoveryRootState::new(
                root_state.root_publication_allocation_bytes(),
                root_state.manifest_capacity_transition(),
                root_state.successor_manifest_capacity(),
                Vec::new(),
                root_state.last_inline_record(),
                root_state.last_inline_segment(),
            )
            .unwrap();
        }
    }

    PersistedPhysicalRecoveryProjection::new(
        base.source_root_generation(),
        root_state,
        records,
        frames,
        placements,
        updates,
        manifests,
    )
    .unwrap()
    .encode()
}

fn projection_with_foreign_frame_coordinate(
    projection: &PersistedPhysicalRecoveryProjection,
    mutation: ProjectionMutation,
) -> Vec<u8> {
    let mut bytes = projection.encode();
    let frame_start = first_frame_start(&bytes);
    let subject_coordinate = frame_start + 8;
    let artifact_generation = subject_coordinate + 1 + 8 + 8 + 8;
    let offset = artifact_generation + 8;
    match mutation {
        ProjectionMutation::ForeignArtifactGeneration => {
            bytes[artifact_generation..artifact_generation + 8]
                .copy_from_slice(&3_u64.to_le_bytes());
        }
        ProjectionMutation::ForeignArtifactOffset => {
            bytes[offset..offset + 8].copy_from_slice(&1_u64.to_le_bytes());
        }
        ProjectionMutation::ForeignArtifactLength => {
            let length = offset + 8;
            bytes[length..length + 4].copy_from_slice(&(frame_len() - 1).to_le_bytes());
        }
        _ => unreachable!(),
    }
    bytes
}

fn projection_without_first_frame(projection: &PersistedPhysicalRecoveryProjection) -> Vec<u8> {
    let mut bytes = projection.encode();
    let frame_start = first_frame_start(&bytes);
    let frame_count_offset = frame_start - 8;
    assert_eq!(encoded_u64(&bytes, frame_count_offset), 1);
    let frame_end = encoded_field_end(&bytes, frame_start);
    bytes.drain(frame_start..frame_end);
    bytes[frame_count_offset..frame_start].copy_from_slice(&0_u64.to_le_bytes());
    bytes
}

fn first_frame_start(bytes: &[u8]) -> usize {
    let mut offset = encoded_field_end(bytes, 0);
    offset += 8;
    offset = encoded_field_end(bytes, offset);
    let record_count = encoded_u64(bytes, offset);
    offset += 8;
    for _ in 0..record_count {
        offset = encoded_field_end(bytes, offset);
    }
    assert_eq!(
        bytes[offset], 0,
        "fixture carries the explicit frame payload tag"
    );
    offset + 1 + 8
}

fn encoded_field_end(bytes: &[u8], offset: usize) -> usize {
    offset + 8 + encoded_u64(bytes, offset) as usize
}

fn encoded_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}
