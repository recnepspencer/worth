use std::path::Path;

use worth_store::physical_runtime::{
    ExternalPhysicalRecordLocator, FramePortCounterSnapshot, ManifestEntryCapacity,
    PageFillPercent, PhysicalMutationIdempotencyMaterial, PhysicalRecordInitialization,
    PhysicalRecordPlacementPolicy, PhysicalWorkCounterSnapshot, PhysicalWorkCounterStage,
    PhysicalWorkEffectFate, PhysicalWorkOperationFamily, PhysicalWorkSignalFamily,
    PhysicalWritebackCounterSnapshot, RecordAppendBatch, RecordByteLimit, RecordReadLimits,
    RootPublicationPhysicalMutationMember, SegmentPageCount, ServingPhysicalRuntime,
};
use worth_store_offline_verifier::OfflineRecordPlacement;
use worth_store_physical_backend::{MediaCounterSnapshot, MediaOperationRole};
use worth_store_physical_format::{
    store_namespace::StableStoreIdentity, PhysicalRecordFormatDeclaration,
};

use super::{
    durable_publication::publish_single, media, read_record,
    scenario_configuration::dense_configuration, stream_fixture::hex, success,
};

#[test]
fn one_batch_rolls_across_four_segments_and_routes_without_scans() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(2);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let records = (0_u8..15)
        .map(|value| vec![value; 3_000])
        .collect::<Vec<_>>();
    let writeback_baseline = SegmentWritebackBaseline::capture(&serving);
    let published = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        publish_single(
            &serving,
            placement,
            PhysicalMutationIdempotencyMaterial::new([161; 32]),
            RecordAppendBatch::try_from_iter(records.iter()).unwrap(),
        )
    }))
    .unwrap_or_else(|_| {
        panic!(
            "C5_PREDICATE:identity-placement-seam: persisted placements must retain admitted record identities"
        )
    });
    let member = &published.settled_members()[0];
    assert_eq!(member.persisted_records().len(), 15);
    assert_eq!(member.observation().segment_artifacts(), 4);
    assert_segment_frame_and_media_evidence(
        &serving,
        &writeback_baseline,
        member.observation().segment_artifacts(),
        published.current_artifacts().len() as u64,
    );
    assert_segment_work_and_signal_evidence(&serving, &writeback_baseline);
    assert_segment_artifact_lengths(&root);
    assert_inline_placement_truth(&root, format.declaration());
    let epoch = member.record_id(0).unwrap().allocation_epoch();
    for index in 0..member.persisted_records().len() {
        let id = member.record_id(index).unwrap();
        assert_eq!(id.allocation_epoch(), epoch);
        assert_eq!(id.ordinal(), index as u64 + 1);
    }
    let request = segment_reader_request(serving.store_identity(), member);
    serving.close();
    assert_fresh_process_records(&root, &request);
}

struct SegmentWritebackBaseline {
    media: MediaCounterSnapshot,
    writebacks: PhysicalWritebackCounterSnapshot,
    frames: FramePortCounterSnapshot,
    work: PhysicalWorkCounterSnapshot,
    wal_frames: u64,
    causal_records: usize,
    causal_overflow: u64,
}

impl SegmentWritebackBaseline {
    fn capture(serving: &ServingPhysicalRuntime) -> Self {
        Self {
            media: serving.media_counters(),
            writebacks: serving.residency_observation().writebacks(),
            frames: serving.certification_frame_port_observer().snapshot(),
            work: serving.physical_work_counters(),
            wal_frames: serving
                .record_submission()
                .wal_observation()
                .expect("the installed WAL owner must remain observable")
                .appended_frames(),
            causal_records: serving.physical_work_observer().causal().records().len(),
            causal_overflow: serving.physical_work_observer().causal().overflow(),
        }
    }
}

fn assert_segment_frame_and_media_evidence(
    serving: &ServingPhysicalRuntime,
    before: &SegmentWritebackBaseline,
    new_segment_candidates: u64,
    root_candidate_artifacts: u64,
) {
    let media = serving.media_counters();
    let residency = serving.residency_observation();
    let frames = serving.certification_frame_port_observer().snapshot();
    let candidate_frames = frames.candidate_frames() - before.frames.candidate_frames();
    let writeback_frames = residency.writebacks().attempts() - before.writebacks.attempts();
    let wal_frames = serving
        .record_submission()
        .wal_observation()
        .expect("the installed WAL owner must remain observable")
        .appended_frames()
        - before.wal_frames;
    assert_eq!(media.replacements(), before.media.replacements() + 3);
    assert_eq!(
        frames.declared_candidate_frames() - before.frames.declared_candidate_frames(),
        candidate_frames
    );
    assert_eq!(
        frames.candidate_publications() - before.frames.candidate_publications(),
        candidate_frames
    );
    assert_eq!(candidate_frames, 16);
    assert_eq!(wal_frames, 1);
    assert_eq!(new_segment_candidates, 4);
    assert_eq!(writeback_frames, 4);
    assert_eq!(root_candidate_artifacts, 8);
    assert_eq!(
        candidate_frames,
        new_segment_candidates + writeback_frames + root_candidate_artifacts
    );
    assert_eq!(frames.writebacks() - before.frames.writebacks(), 4);
    assert_eq!(
        residency.writebacks().exact_receipts() - before.writebacks.exact_receipts(),
        4
    );
    assert_eq!(
        residency.writebacks().retryable() - before.writebacks.retryable(),
        0
    );
    assert_eq!(
        residency.writebacks().inspection_required() - before.writebacks.inspection_required(),
        0
    );
    assert_eq!(
        media.attempts_for(MediaOperationRole::PositionedWrite)
            - before
                .media
                .attempts_for(MediaOperationRole::PositionedWrite),
        candidate_frames + wal_frames
    );
    assert_eq!(residency.counters().dirty_frames(), 0);
    assert_eq!(residency.counters().candidate_frames(), 0);
    assert_eq!(residency.counters().active_writeback_claims(), 0);
}

fn assert_segment_work_and_signal_evidence(
    serving: &ServingPhysicalRuntime,
    before: &SegmentWritebackBaseline,
) {
    assert_eq!(
        serving.physical_work_counters().count(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkCounterStage::Terminal,
        ) - before.work.count(
            PhysicalWorkOperationFamily::ArtifactRangeWrite,
            PhysicalWorkCounterStage::Terminal,
        ),
        4
    );
    assert_eq!(
        serving.physical_work_observer().causal().overflow(),
        before.causal_overflow
    );
    let causal = serving.physical_work_observer().causal().records();
    let writebacks = causal[before.causal_records..]
        .iter()
        .filter(|record| record.operation() == PhysicalWorkOperationFamily::ArtifactRangeWrite)
        .collect::<Vec<_>>();
    assert_eq!(writebacks.len(), 4);
    let signal_bindings = serving.physical_signal_aspect_binding_observations();
    for writeback in writebacks {
        assert_eq!(
            writeback.effect_fate(),
            PhysicalWorkEffectFate::WriteCompleted
        );
        assert!(writeback.backend_operation().is_some());
        assert!(signal_bindings.iter().any(|binding| {
            binding.digest() == writeback.signal_binding()
                && binding
                    .families()
                    .contains(PhysicalWorkSignalFamily::ExactWriteback)
        }));
    }
}

fn assert_segment_artifact_lengths(root: &Path) {
    for segment in 1..=4_u64 {
        let path = root.join(format!(
            "families/records/segments/segment-{segment:016x}-0000000000000001.pages"
        ));
        assert_eq!(std::fs::metadata(path).unwrap().len(), 32_768);
    }
}

fn assert_inline_placement_truth(root: &Path, format: PhysicalRecordFormatDeclaration) {
    let root_manifest = super::manifest_fixture::decode_routing_tree(root, 2, format, 128);
    for (index, entry) in root_manifest.placements().iter().enumerate() {
        let OfflineRecordPlacement::Inline {
            segment,
            page,
            slot,
            ..
        } = entry
        else {
            panic!("the rollover fixture contains only inline records")
        };
        assert_eq!(*segment, index as u64 / 4 + 1);
        assert_eq!(*page, index as u64 / 2 + 1);
        assert_eq!(*slot, index as u16 % 2 + 1);
    }
}

fn segment_reader_request(
    store_identity: StableStoreIdentity,
    member: &RootPublicationPhysicalMutationMember,
) -> String {
    let order = [14_usize, 0, 7, 3, 12, 1, 9, 5, 13, 2, 11, 6, 4, 10, 8];
    order
        .iter()
        .map(|index| {
            let locator = ExternalPhysicalRecordLocator::new(
                store_identity,
                member.record_id(*index).unwrap(),
            );
            format!("{index}:{}", hex(&locator.encode()))
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn assert_fresh_process_records(root: &Path, request: &str) {
    let order = [14_usize, 0, 7, 3, 12, 1, 9, 5, 13, 2, 11, 6, 4, 10, 8];
    let output = super::child_process::run_child("segment_reader", root, Some(request));
    for index in order {
        assert!(
            output
                .lines()
                .any(|line| line == format!("C5_SEGMENT {index} {index} 3000 1 1")),
            "C5_PREDICATE:identity-placement-seam"
        );
    }
}

#[test]
fn multi_block_manifest_lookup_has_logarithmic_path_and_exact_parity() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, _, access) = dense_configuration(2);
    let placement = PhysicalRecordPlacementPolicy::builder()
        .segment_pages(SegmentPageCount::new(2).unwrap())
        .extent_threshold(RecordByteLimit::new(8_000).unwrap())
        .page_fill(PageFillPercent::new(50).unwrap())
        .manifest_capacity(ManifestEntryCapacity::new(2).unwrap())
        .admit(format)
        .unwrap();
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let payloads = (0_u8..9).map(|value| vec![value; 100]).collect::<Vec<_>>();
    let published = publish_single(
        &serving,
        placement,
        PhysicalMutationIdempotencyMaterial::new([162; 32]),
        RecordAppendBatch::try_from_iter(payloads.iter()).unwrap(),
    );
    let member = &published.settled_members()[0];
    let offline = super::manifest_fixture::decode_routing_tree(&root, 2, format.declaration(), 2);
    assert_eq!(offline.routing_level(), Some(3));
    assert_eq!(offline.placements().len(), payloads.len());
    assert!(offline.manifest_blocks() > 5);
    assert_eq!(offline.segment_pages().len(), 1);
    assert_eq!(offline.free_space().len(), 2);
    for (index, expected) in payloads.iter().enumerate().rev() {
        let session = serving
            .records()
            .open(
                member.record_id(index).unwrap(),
                RecordReadLimits::new(RecordByteLimit::new(100).unwrap()),
            )
            .unwrap();
        let observation = session.observation();
        assert_eq!(observation.manifest_blocks(), 5);
        assert_eq!(
            observation.manifest_comparisons(),
            if index == 8 { 6 } else { 9 }
        );
        assert_eq!(
            observation.manifest_bytes(),
            (if index == 8 { 616 } else { 848 })
                + observation.manifest_blocks() * super::durable_frame_oracle::HEADER_BYTES as u64
        );
        assert_eq!(observation.touched_segments(), 1);
        assert_eq!(observation.touched_pages(), 1);
        assert_eq!(observation.touched_extents(), 0);
        assert_eq!(observation.payload_bytes(), 0);
        assert_eq!(read_record(session, 100).0, *expected);
        assert_eq!(
            offline.placements()[index].record().ordinal(),
            index as u64 + 1
        );
    }
    serving.close();
}

#[test]
fn cross_batch_page_reuse_is_cow_and_does_not_rebase_old_slots() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let (format, placement, access) = dense_configuration(4);
    let serving = success(initialize_record_store!(media(&root), |durability| {
        PhysicalRecordInitialization::new(format, placement, access, durability)
    }));
    let publish = |material, batch| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            publish_single(&serving, placement, material, batch)
        }))
        .unwrap_or_else(|_| {
            panic!(
                "C5_PREDICATE:page-layout: copy-on-write publication must retain the admitted slot layout"
            )
        })
    };
    let first = publish(
        PhysicalMutationIdempotencyMaterial::new([163; 32]),
        RecordAppendBatch::try_from_iter([b"alpha".as_slice(), b"beta".as_slice()]).unwrap(),
    );
    let old_page = std::fs::read(
        root.join("families/records/segments/segment-0000000000000001-0000000000000001.pages"),
    )
    .unwrap();
    let old_offset = old_page[88..92].to_vec();
    let before_admission = serving.resident_admission_counters();
    let second = publish(
        PhysicalMutationIdempotencyMaterial::new([164; 32]),
        RecordAppendBatch::try_from_iter([b"gamma".as_slice(), b"delta".as_slice()]).unwrap(),
    );
    let after_admission = serving.resident_admission_counters();
    // The second publication reads one source page and two blocks from each
    // routing family (record, segment membership, and free space). All seven
    // reads now cross resident integrity admission.
    assert_eq!(
        after_admission.fresh_validations() + after_admission.exact_record_reuses(),
        before_admission.fresh_validations() + before_admission.exact_record_reuses() + 7,
    );
    assert_eq!(
        after_admission.owner_decoder_entries(),
        before_admission.owner_decoder_entries() + 7,
    );
    assert_eq!(
        after_admission.refusals_before_owner_entry(),
        before_admission.refusals_before_owner_entry(),
    );
    let new_page = std::fs::read(
        root.join("families/records/segments/segment-0000000000000001-0000000000000002.pages"),
    )
    .unwrap();
    assert_eq!(&new_page[88..92], old_offset, "C5_PREDICATE:page-layout");
    assert!(root
        .join("families/records/segments/segment-0000000000000001-0000000000000001.pages")
        .is_file());
    let old_record = serving
        .records()
        .open(
            first.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(old_record, 5).0, b"alpha");
    let appended = serving
        .records()
        .open(
            second.settled_members()[0].record_id(0).unwrap(),
            RecordReadLimits::new(RecordByteLimit::new(32).unwrap()),
        )
        .unwrap();
    assert_eq!(read_record(appended, 5).0, b"gamma");
    serving.close();
}
