use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::PhysicalWalPolicy;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, InlineArtifactRewritePlanDenial, ManifestEntryCapacity,
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PhysicalReadProtectionPolicy,
    PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration, PhysicalRecordId,
    PhysicalRecordInitialization, PhysicalRecordOpen, PhysicalRecordPlacementPolicy,
    PlannedInlineRewriteArtifact, RecordByteLimit, RecordReadLimits, SegmentPageCount,
    WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::MediaOperationRole;

const RECORD_BYTES: usize = 7_500;
const PUBLISHED_ARTIFACTS: u32 = 16;

#[test]
fn rewrite_scope_selects_one_four_and_sixteen_artifacts() {
    for count in [1_u32, 4, 16] {
        select_and_rewrite(count);
    }
}

fn select_and_rewrite(count: u32) {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_one_page_segments(&root);
    let policy = one_page_placement();
    let ids = publish_one_record_per_segment(&serving, &policy, PUBLISHED_ARTIFACTS);
    assert_eq!(
        super::segment_ids(&root).len(),
        PUBLISHED_ARTIFACTS as usize,
        "each record must seal its own segment artifact"
    );
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let planned = serving
        .record_submission()
        .plan_inline_artifact_rewrites(count)
        .unwrap_or_else(|denial| panic!("{count} artifacts were not planned: {denial:?}"));
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
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
        *super::segment_ids(&root).iter().next().unwrap(),
        "the scope starts at the first artifact, not the tail"
    );
    let before = super::segment_files(&root);
    for (ordinal, artifact) in planned.iter().copied().enumerate() {
        rewrite_one(&serving, &policy, 100 + ordinal as u64, artifact);
    }
    let after = super::segment_files(&root);
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
        let tail = super::segment_ids(&root)
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

fn publish_one_record_per_segment(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    count: u32,
) -> Vec<PhysicalRecordId> {
    (0..count)
        .map(|ordinal| {
            let mut bytes = vec![ordinal as u8; RECORD_BYTES];
            bytes[0] = ordinal as u8;
            super::append(serving, *policy, u64::from(ordinal) + 1, &bytes)
        })
        .collect()
}

fn rewrite_one(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    artifact: PlannedInlineRewriteArtifact,
) {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .rewrite_planned_inline_artifact(
            *policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            artifact,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("artifact {ordinal} did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("artifact {ordinal} denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => {
            panic!("artifact {ordinal} deferred: {deferred:?}")
        }
        TransitionOutcome::Stale(stale) => panic!("artifact {ordinal} stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => {
            panic!("artifact {ordinal} rebind: {rebind:?}")
        }
        TransitionOutcome::Failed(failure) => panic!("artifact {ordinal} failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("artifact {ordinal} had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!(
                "artifact {ordinal} became indeterminate at {:?}",
                fate.stage()
            )
        }
    }
}

fn read_back(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    ids: &[PhysicalRecordId],
) {
    let limits = RecordReadLimits::new(RecordByteLimit::new(RECORD_BYTES as u32).unwrap());
    let reader = serving.records().unwrap();
    for (ordinal, id) in ids.iter().copied().enumerate() {
        let mut session = reader.open(id, limits).unwrap();
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes()[0], ordinal as u8);
        assert_eq!(chunk.bytes().len(), RECORD_BYTES);
    }
}

fn one_page_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(SegmentPageCount::new(1).unwrap())
        .admit(format)
        .unwrap()
}

fn open_one_page_segments(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_store(root, true)
}

fn reopen_one_page_segments(
    root: &std::path::Path,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    open_store(root, false)
}

fn open_store(
    root: &std::path::Path,
    initialize: bool,
) -> worth_store::physical_runtime::ServingPhysicalRuntime {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    let access = PhysicalRecordAccessPolicy::builder().admit(format).unwrap();
    let media = super::super::media(root);
    let durability = super::super::durability_with_wal_policy(
        &media,
        PhysicalWalPolicy::segmented(
            WalSegmentByteLimit::new(std::num::NonZeroU64::new(512 * 1024).unwrap()),
            WalSegmentInventoryLimit::new(std::num::NonZeroU32::new(64).unwrap()),
        ),
    );
    let residency = super::rewrite_axis_residency(format, 1024 * 1024);
    if initialize {
        super::super::success(
            media.initialize_record_store(
                PhysicalRecordInitialization::new(format, one_page_placement(), access, durability)
                    .with_residency_policy(residency)
                    .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
            ),
        )
    } else {
        super::super::success(
            media.open_record_store(
                PhysicalRecordOpen::new(format, access, durability)
                    .with_residency_policy(residency)
                    .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
            ),
        )
    }
}

#[test]
fn artifact_scope_above_sixteen_one_page_segments_is_refused_before_media() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_one_page_segments(&root);
    let policy = one_page_placement();
    publish_one_record_per_segment(&serving, &policy, 17);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let denial = serving
        .record_submission()
        .plan_inline_artifact_rewrites(17)
        .expect_err("seventeen artifacts fit in 256 KiB");
    assert_eq!(denial, InlineArtifactRewritePlanDenial::PhysicalPressure);
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}
