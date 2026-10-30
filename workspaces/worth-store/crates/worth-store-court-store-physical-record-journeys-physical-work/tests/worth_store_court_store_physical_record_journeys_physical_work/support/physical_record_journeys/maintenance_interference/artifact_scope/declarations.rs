pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use worth_store::physical_runtime::PhysicalWalPolicy;
pub(crate) use worth_store_physical_backend::MediaOperationRole;
pub(crate) use {
    worth_store::physical_runtime::AdmittedPhysicalRecordFormat,
    worth_store::physical_runtime::InlineArtifactRewritePlanDenial,
    worth_store::physical_runtime::ManifestEntryCapacity,
    worth_store::physical_runtime::PhysicalMutationDeadline,
    worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial,
    worth_store::physical_runtime::PhysicalMutationOutcome,
    worth_store::physical_runtime::PhysicalMutationPreparationSuccess,
    worth_store::physical_runtime::PhysicalMutationRequest,
    worth_store::physical_runtime::PhysicalReadProtectionPolicy,
    worth_store::physical_runtime::PhysicalRecordAccessPolicy,
    worth_store::physical_runtime::PhysicalRecordFormatDeclaration,
    worth_store::physical_runtime::PhysicalRecordId,
    worth_store::physical_runtime::PhysicalRecordInitialization,
    worth_store::physical_runtime::PhysicalRecordOpen,
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy,
    worth_store::physical_runtime::PlannedInlineRewriteArtifact,
    worth_store::physical_runtime::RecordByteLimit,
    worth_store::physical_runtime::RecordReadLimits,
    worth_store::physical_runtime::SegmentPageCount,
    worth_store::physical_runtime::WalSegmentByteLimit,
    worth_store::physical_runtime::WalSegmentInventoryLimit,
};

pub(crate) const RECORD_BYTES: usize = 7_500;
pub(crate) const PUBLISHED_ARTIFACTS: u32 = 16;

pub(crate) fn select_and_rewrite(count: u32) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_one_page_segments(&root);
    let policy = one_page_placement();
    let ids = publish_one_record_per_segment(&serving, &policy, PUBLISHED_ARTIFACTS);
    assert_eq!(
        worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids(
            &root
        )
        .len(),
        PUBLISHED_ARTIFACTS as usize,
        "each record must seal its own segment artifact"
    );
    let writes = serving
        .media_counters()
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let planned = serving
        .record_submission()
        .plan_inline_artifact_rewrites(count)
        .unwrap_or_else(|denial| panic!("{count} artifacts were not planned: {denial:?}"));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes,
        "planning {count} artifacts wrote media"
    );
    assert_eq!(planned.len(), count as usize);
    let mut seen = std::collections::BTreeSet::new();
    let mut source_bytes = 0_u64;
    for artifact in &planned {
        assert!(seen.insert(artifact.segment_id()));
        source_bytes += artifact.source_bytes();
        assert!(artifact.source_bytes() <= 256 * 1024);
        assert!(artifact.pages() >= 1);
    }
    assert!(source_bytes <= 256 * 1024);
    assert_eq!(
        planned[0].segment_id(),
        *worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids(&root).iter().next().unwrap(),
        "the scope starts at the first artifact, not the tail"
    );
    let before = worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files(&root);
    for (ordinal, artifact) in planned.iter().copied().enumerate() {
        rewrite_one(&serving, &policy, 100 + ordinal as u64, artifact);
    }
    let after = worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files(&root);
    for artifact in &planned {
        let rewritten = format!(
            "segment-{:016x}-{:016x}.pages",
            artifact.segment_id(),
            artifact.generation() + 1
        );
        assert!(
            after.contains(&rewritten),
            "artifact {} did not gain generation {}",
            artifact.segment_id(),
            artifact.generation() + 1
        );
        assert!(
            !before.contains(&rewritten),
            "generation {} of segment {} already existed",
            artifact.generation() + 1,
            artifact.segment_id()
        );
    }
    if count < PUBLISHED_ARTIFACTS {
        let tail = worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_ids(&root)
            .iter()
            .next_back()
            .copied()
            .unwrap();
        assert!(planned.iter().all(|artifact| artifact.segment_id() != tail));
        let tail_rewrite = format!("segment-{tail:016x}-{:016x}.pages", 2);
        assert!(
            !after.contains(&tail_rewrite),
            "selecting {count} artifacts rewrote the tail"
        );
    }
    read_back(&serving, &ids);
    serving.close();
    let serving = reopen_one_page_segments(&root);
    read_back(&serving, &ids);
    serving.close();
}

pub(crate) fn publish_one_record_per_segment(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    count: u32,
) -> Vec<PhysicalRecordId> {
    (0..count)
        .map(|ordinal| {
            let mut bytes = vec![ordinal as u8; RECORD_BYTES];
            bytes[0] = ordinal as u8;
            worth_store_testkit_store::physical_record_journeys::maintenance_interference::append(
                serving,
                *policy,
                u64::from(ordinal) + 1,
                &bytes,
            )
        })
        .collect()
}

pub(crate) fn rewrite_one(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    artifact: PlannedInlineRewriteArtifact,
) {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let prepared = match submission
        .rewrite_planned_inline_artifact(
            *policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
            artifact,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("artifact {ordinal} did not stay prepared")
        }
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("artifact {ordinal} denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(deferred) => {
            panic!("artifact {ordinal} deferred: {deferred:?}")
        }
        worth_proof::TransitionOutcome::Stale(stale) => {
            panic!("artifact {ordinal} stale: {stale:?}")
        }
        worth_proof::TransitionOutcome::RebindRequired(rebind) => {
            panic!("artifact {ordinal} rebind: {rebind:?}")
        }
        worth_proof::TransitionOutcome::Failed(failure) => {
            panic!("artifact {ordinal} failed: {failure:?}")
        }
    };
    match prepared.execute() {
        worth_store::physical_runtime::PhysicalMutationOutcome::Completed(_) => {}
        worth_store::physical_runtime::PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("artifact {ordinal} had no effect: {:?}", fate.cause())
        }
        worth_store::physical_runtime::PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "artifact {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}

pub(crate) fn read_back(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ids: &[PhysicalRecordId],
) {
    let limits = worth_store::physical_runtime::RecordReadLimits::new(
        worth_store::physical_runtime::RecordByteLimit::new(RECORD_BYTES as u32).unwrap(),
    );
    let reader = serving.records().unwrap();
    for (ordinal, id) in ids.iter().copied().enumerate() {
        let mut session = reader.open(id, limits).unwrap();
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes()[0], ordinal as u8);
        assert_eq!(chunk.bytes().len(), RECORD_BYTES);
    }
}

pub(crate) fn one_page_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    worth_store::physical_runtime::PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(worth_store::physical_runtime::ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(worth_store::physical_runtime::SegmentPageCount::new(1).unwrap())
        .admit(format)
        .unwrap()
}

pub(crate) fn open_one_page_segments(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_store(root, true)
}

pub(crate) fn reopen_one_page_segments(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_store(root, false)
}

pub(crate) fn open_store(
    root: &std::path::Path,
    initialize: bool,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = worth_store::physical_runtime::AdmittedPhysicalRecordFormat::admit(
        worth_store::physical_runtime::PhysicalRecordFormatDeclaration::builder()
            .admit()
            .unwrap(),
    );
    let access = worth_store::physical_runtime::PhysicalRecordAccessPolicy::builder()
        .admit(format)
        .unwrap();
    let media = worth_store_testkit_store::physical_record_journeys::media(root);
    let durability =
        worth_store_testkit_store::physical_record_journeys::durability_with_wal_policy(
            &media,
            worth_store::physical_runtime::PhysicalWalPolicy::segmented(
                worth_store::physical_runtime::WalSegmentByteLimit::new(
                    std::num::NonZeroU64::new(512 * 1024).unwrap(),
                ),
                worth_store::physical_runtime::WalSegmentInventoryLimit::new(
                    std::num::NonZeroU32::new(64).unwrap(),
                ),
            ),
        );
    let residency = worth_store_testkit_store::physical_record_journeys::maintenance_interference::rewrite_axis_residency(format, 1024 * 1024);
    if initialize {
        worth_store_testkit_store::physical_record_journeys::success(
            media.initialize_record_store(
                worth_store::physical_runtime::PhysicalRecordInitialization::new(
                    format,
                    one_page_placement(),
                    access,
                    durability,
                )
                .with_residency_policy(residency)
                .with_read_protection_policy(
                    worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
                ),
            ),
        )
    } else {
        worth_store_testkit_store::physical_record_journeys::success(
            media.open_record_store(
                worth_store::physical_runtime::PhysicalRecordOpen::new(format, access, durability)
                    .with_residency_policy(residency)
                    .with_read_protection_policy(
                        worth_store::physical_runtime::PhysicalReadProtectionPolicy::default(),
                    ),
            ),
        )
    }
}
