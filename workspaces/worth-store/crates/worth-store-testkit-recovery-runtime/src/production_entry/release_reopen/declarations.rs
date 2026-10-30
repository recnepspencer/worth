//! Real Store release, checkpoint, close, and separate-process recovery.

pub use std::num::NonZeroU32;
pub use {
    std::num::NonZeroU16, std::num::NonZeroU64, std::process::Command,
    worth_proof::AdmittedBlobReleaseProof, worth_proof::TransitionOutcome,
    worth_store::physical_runtime::BlobReclaimDisposition,
    worth_store::physical_runtime::BlobReclaimLimits,
    worth_store::physical_runtime::BlobReclaimRequest,
    worth_store::physical_runtime::PhysicalCheckpointDeadline,
    worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey,
    worth_store::physical_runtime::PhysicalCheckpointOutcome,
    worth_store::physical_runtime::PhysicalCheckpointRequest,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld,
};
pub use {
    worth_store::physical_runtime::BlobReclaimReceipt,
    worth_store::physical_runtime::BlobReclaimRetirement,
    worth_store::physical_runtime::BlobReclaimRetirementBudget,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordCountLimit,
    worth_store::physical_runtime::RecordScanOutcome,
    worth_store::physical_runtime::RecordScanRequest,
    worth_store::physical_runtime::ServingPhysicalRuntime,
};
pub use {
    worth_store_physical_format::checkpoint_certificate_frame_bytes,
    worth_store_physical_format::decode_blob_record,
    worth_store_physical_format::decode_checkpoint_certificate,
    worth_store_physical_format::BlobReclaimSourceKind, worth_store_physical_format::BlobRecordV1,
    worth_store_physical_format::CheckpointCertificateKind,
    worth_store_physical_format::CheckpointStreamFooter,
    worth_store_physical_format::PersistedRecordIdentity,
    worth_store_physical_format::ReleaseCheckpointAccumulatorV2,
    worth_store_physical_format::ReleaseCheckpointBatchV1,
    worth_store_physical_format::ReleaseCheckpointCertificateV1,
    worth_store_physical_format::CHECKPOINT_CERTIFICATE_PREFIX_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES,
    worth_store_physical_format::CHECKPOINT_CERTIFIED_SCHEMA,
};

pub fn run(maximum_dropped_records: u16, expect_terminal: bool) -> std::process::Output {
    let (world, mut first, record) = released_world(maximum_dropped_records);
    let mut selected_first_tip = None;
    if expect_terminal {
        let (batches, accumulator) = selected_release_certificates(&world);
        if let Some(batch) = batches.last() {
            assert_eq!(accumulator.base().tip(), batch.tip_provenance().unwrap());
        }
        assert!(accumulator.base().terminal());
        selected_first_tip = Some(accumulator.base().tip());
        assert!(first.dropped_records().len() > 1);
        assert_eq!(first.remaining_payload_records(), 0);
        assert_selected_terminal_descriptor(world.serving());
        let expected_retired = first
            .displaced_extents()
            .iter()
            .map(|extent| extent.range().length())
            .sum::<u64>();
        assert!(
            expected_retired > 0,
            "terminal drop must displace real extents"
        );
        let first_checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
            worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x73; 32]),
            worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
                .unwrap(),
        );
        let worth_proof::TransitionOutcome::Success(handle) = world
            .serving()
            .checkpoints()
            .start(first_checkpoint)
            .into_raw()
        else {
            panic!("first release-custody checkpoint must admit")
        };
        assert!(matches!(
            handle.wait(),
            worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
        ));
        let (batches, accumulator) = selected_release_certificates(&world);
        assert!(
            batches.is_empty(),
            "successor checkpoint must carry the earlier Batch"
        );
        assert_eq!(Some(accumulator.base().tip()), selected_first_tip);
        assert!(accumulator.base().prior_checkpoint_sequence() > 0);
        let budget = worth_store::physical_runtime::BlobReclaimRetirementBudget::new(
            std::num::NonZeroU32::new(128).unwrap(),
            worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(120_000)
                .unwrap(),
        );
        for _ in 0..64 {
            if first.retirement() == worth_store::physical_runtime::BlobReclaimRetirement::Completed
            {
                break;
            }
            world
                .serving()
                .blobs()
                .unwrap()
                .continue_reclaim_retirement(&mut first, budget)
                .expect("continue terminal retirement");
        }
        assert_eq!(
            first.retirement(),
            worth_store::physical_runtime::BlobReclaimRetirement::Completed
        );
        assert_eq!(first.bytes_released(), expected_retired);
    } else {
        assert_eq!(first.dropped_records(), &[record]);
        assert!(first.remaining_payload_records() > 0);
    }
    let checkpoint = worth_store::physical_runtime::PhysicalCheckpointRequest::fuzzy(
        worth_store::physical_runtime::PhysicalCheckpointIdempotencyKey::new([0x72; 32]),
        worth_store::physical_runtime::PhysicalCheckpointDeadline::after_milliseconds(30_000)
            .unwrap(),
    );
    let worth_proof::TransitionOutcome::Success(handle) =
        world.serving().checkpoints().start(checkpoint).into_raw()
    else {
        panic!("checkpoint after release must admit")
    };
    assert!(matches!(
        handle.wait(),
        worth_store::physical_runtime::PhysicalCheckpointOutcome::Completed(_)
    ));
    if expect_terminal {
        assert_selected_terminal_descriptor(world.serving());
        let (batches, accumulator) = selected_release_certificates(&world);
        assert!(
            batches.is_empty(),
            "later checkpoint must carry the earlier release, not replay Batch"
        );
        assert_eq!(Some(accumulator.base().tip()), selected_first_tip);
        assert!(accumulator.base().prior_checkpoint_sequence() > 0);
        assert!(
            accumulator.base().terminal(),
            "selected release tip must be terminal"
        );
    }
    let (_, selected) = selected_release_certificates(&world);
    let _selected_heads = crate::release_reopen::selected_head_oracle::selected_heads(
        world.retained_root().path(),
        selected,
    );
    let retained = world.retained_root();
    let root = retained.path().to_path_buf();
    drop(world);
    // The selected head's bounded control/tree closure alone exceeds the
    // 512 KiB C8 phase-two profile. Use the existing finite C11 envelope;
    // the phase-two runner and production allocation guard stay unchanged.
    std::process::Command::new(worth_store_testkit_executables::physical_store_recover())
        .arg(&root)
        .arg("--bounded-profile=c11-blob-crash-v1")
        .output()
        .expect("run C11 release checkpoint recovery entry")
}

pub fn released_world(
    maximum_dropped_records: u16,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
) {
    let (world, receipt, record, _) = released_world_in_tier(maximum_dropped_records, false, None);
    (world, receipt, record)
}

pub fn released_tier_world(
    maximum_dropped_records: u16,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
) {
    let (world, receipt, record, _) = released_world_in_tier(maximum_dropped_records, true, None);
    (world, receipt, record)
}

pub fn released_world_in_tier(
    maximum_dropped_records: u16,
    activate_tier: bool,
    wal_segment_bytes: Option<NonZeroU64>,
) -> (
    PhysicalResidencyStoreWorld,
    BlobReclaimReceipt,
    PersistedRecordIdentity,
    AdmittedBlobReleaseProof,
) {
    let (world, proof, record) = crate::production_entry::release_reopen::published_world::create(
        activate_tier,
        wal_segment_bytes,
        None,
    );
    let request = worth_store::physical_runtime::BlobReclaimRequest::released(
        reissue_proof(&proof),
        world.placement(),
        worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
            .unwrap(),
        worth_store::physical_runtime::BlobReclaimLimits::new(
            std::num::NonZeroU64::new(1024).unwrap(),
            std::num::NonZeroU64::new(64 << 20).unwrap(),
            std::num::NonZeroU16::new(maximum_dropped_records).unwrap(),
        )
        .unwrap(),
    );
    let first = world
        .serving()
        .blobs()
        .unwrap()
        .reclaim(request)
        .expect("release batch")
        .wait()
        .expect("release completed");
    assert_eq!(
        first.disposition(),
        worth_store::physical_runtime::BlobReclaimDisposition::Dropped
    );
    assert!(first.dropped_records().contains(&record));
    (world, first, record, proof)
}

pub fn reissue_proof(proof: &AdmittedBlobReleaseProof) -> AdmittedBlobReleaseProof {
    worth_proof::AdmittedBlobReleaseProof::certification_admit(
        proof.store(),
        proof.object(),
        proof.generation(),
        proof.publication_allocation_epoch(),
        proof.publication_record_ordinal(),
        proof.publication_frame_sha256(),
        proof.issuer_evidence_sha256(),
    )
    .expect("same genuine selected release evidence")
}

pub fn selected_release_certificates(
    world: &PhysicalResidencyStoreWorld,
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    let bytes = std::fs::read(
        world
            .retained_root()
            .path()
            .join("families/checkpoint.current"),
    )
    .expect("selected checkpoint bytes");
    selected_release_certificates_from_bytes(&bytes)
}

pub fn selected_release_certificates_from_bytes(
    bytes: &[u8],
) -> (
    Vec<ReleaseCheckpointBatchV1>,
    ReleaseCheckpointAccumulatorV2,
) {
    assert_eq!(bytes[8], CHECKPOINT_CERTIFIED_SCHEMA);
    let footer_start = bytes.len() - CHECKPOINT_CERTIFIED_FOOTER_RECORD_BYTES;
    let footer =
        worth_store_physical_format::CheckpointStreamFooter::decode_record(&bytes[footer_start..])
            .expect("selected certified checkpoint footer");
    let mut offset = footer_start - footer.certificate_record_bytes() as usize;
    let mut batches = Vec::new();
    let mut accumulator = None;
    for _ in 0..footer.certificate_record_count() {
        let frame_bytes = checkpoint_certificate_frame_bytes(
            &bytes[offset..offset + CHECKPOINT_CERTIFICATE_PREFIX_BYTES],
        )
        .expect("selected certificate frame prefix");
        let (kind, payload) = decode_checkpoint_certificate(&bytes[offset..offset + frame_bytes])
            .expect("selected certificate frame");
        if kind == worth_store_physical_format::CheckpointCertificateKind::ReleasedDrop {
            match worth_store_physical_format::ReleaseCheckpointCertificateV1::decode(payload)
                .expect("selected tag-7 certificate")
            {
                worth_store_physical_format::ReleaseCheckpointCertificateV1::Batch(value) => {
                    batches.push(value)
                }
                worth_store_physical_format::ReleaseCheckpointCertificateV1::AccumulatorV2(
                    value,
                ) => {
                    assert!(accumulator.replace(value).is_none());
                }
                worth_store_physical_format::ReleaseCheckpointCertificateV1::Accumulator(_) => {
                    panic!("released checkpoint cannot use headless V1 accumulator")
                }
                worth_store_physical_format::ReleaseCheckpointCertificateV1::NoRelease(_) => {
                    panic!("released checkpoint cannot carry NoRelease custody")
                }
            }
        }
        offset += frame_bytes;
    }
    assert_eq!(offset, footer_start);
    (batches, accumulator.expect("selected release accumulator"))
}

pub fn assert_selected_terminal_descriptor(serving: &ServingPhysicalRuntime) {
    let mut scan = serving
        .records()
        .unwrap()
        .scan(
            worth_store::physical_runtime::RecordScanRequest::from_start()
                .with_batch_limit(worth_store::physical_runtime::RecordCountLimit::new(8).unwrap())
                .with_payload_limit(
                    worth_store::physical_runtime::RecordByteLimit::new(1024).unwrap(),
                ),
        )
        .unwrap();
    let mut scratch = [0; 8192];
    let mut found = 0;
    while let worth_store::physical_runtime::RecordScanOutcome::Batch(batch) =
        scan.read_next_into(&mut scratch).unwrap()
    {
        for index in 0..batch.records().len() {
            let Some(bytes) = batch.payload(index) else {
                continue;
            };
            let released_terminal = match decode_blob_record(bytes) {
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV2(value)) => {
                    value.source_kind()
                        == worth_store_physical_format::BlobReclaimSourceKind::ReleasedGeneration
                        && value.terminal()
                }
                Ok(worth_store_physical_format::BlobRecordV1::ReclaimDescriptorV3(value)) => {
                    value.base().terminal()
                }
                _ => false,
            };
            found += usize::from(released_terminal);
        }
        if batch.is_complete() {
            break;
        }
    }
    assert_eq!(found, 1, "one terminal descriptor remains selected");
}
