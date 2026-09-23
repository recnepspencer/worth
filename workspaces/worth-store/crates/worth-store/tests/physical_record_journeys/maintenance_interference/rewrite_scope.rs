use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::PhysicalWalPolicy;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, ManifestEntryCapacity, PhysicalMutationDeadline,
    PhysicalMutationIdempotencyMaterial, PhysicalMutationOutcome,
    PhysicalMutationPreparationDenial, PhysicalMutationPreparationSuccess, PhysicalMutationRequest,
    PhysicalReadProtectionPolicy, PhysicalRecordAccessPolicy, PhysicalRecordFormatDeclaration,
    PhysicalRecordId, PhysicalRecordInitialization, PhysicalRecordOpen,
    PhysicalRecordPlacementPolicy, RecordAppendBatch, RecordAppendDenial, RecordByteLimit,
    RecordReadLimits, SegmentPageCount, WalSegmentByteLimit, WalSegmentInventoryLimit,
};
use worth_store_physical_backend::MediaOperationRole;

const PAGE_BYTES: u64 = 16 * 1024;
pub(super) const RECORD_BYTES: usize = 7_500;

#[test]
fn rewrite_scope_selects_one_four_and_sixteen_source_pages() {
    for pages in [1_u32, 4, 16] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let serving = open_wide(&root);
        let policy = wide_placement();
        let ids = publish_span(&serving, &policy, pages);
        let before = super::super::durability_admission::produced_rewrite_payloads(&root).len();
        rewrite(&serving, &policy, 100 + u64::from(pages), pages);
        let found = super::super::durability_admission::produced_rewrite_payloads(&root);
        assert_eq!(found.len(), before + 1);
        assert_eq!(
            u64::from(found[found.len() - 1].source_length),
            u64::from(pages) * PAGE_BYTES
        );
        let limits = RecordReadLimits::new(RecordByteLimit::new(RECORD_BYTES as u32).unwrap());
        let reader = serving.records().unwrap();
        for (ordinal, id) in ids.iter().copied().enumerate() {
            let mut session = reader.open(id, limits).unwrap();
            let chunk = session.next_chunk().unwrap().unwrap();
            assert_eq!(chunk.bytes()[0], ordinal as u8);
            assert_eq!(chunk.bytes().len(), RECORD_BYTES);
        }
        drop(reader);
        serving.close();
        let serving = reopen_wide(&root);
        let reader = serving.records().unwrap();
        for (ordinal, id) in ids.iter().copied().enumerate() {
            let mut session = reader.open(id, limits).unwrap();
            let chunk = session.next_chunk().unwrap().unwrap();
            assert_eq!(chunk.bytes()[0], ordinal as u8);
            assert_eq!(chunk.bytes().len(), RECORD_BYTES);
        }
        serving.close();
    }
}

#[test]
fn a_retry_with_a_different_page_span_is_not_the_same_rewrite() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_wide(&root);
    let policy = wide_placement();
    publish_span(&serving, &policy, 4);
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&7_u64.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let held = match submission
        .rewrite_selected_inline_pages(
            policy,
            PhysicalMutationRequest::platform_durable(
                key.clone(),
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            1,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Denied(denial) => panic!("one-page rewrite denied: {denial:?}"),
        TransitionOutcome::Deferred(_) => panic!("one-page rewrite deferred"),
        TransitionOutcome::Stale(_) => panic!("one-page rewrite stale"),
        TransitionOutcome::RebindRequired(_) => panic!("one-page rewrite rebind"),
        TransitionOutcome::Failed(_) => panic!("one-page rewrite failed"),
        TransitionOutcome::Success(_) => panic!("one-page rewrite did not stay prepared"),
    };
    assert!(matches!(
        submission
            .rewrite_selected_inline_pages(
                policy,
                PhysicalMutationRequest::platform_durable(
                    key,
                    PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
                ),
                4,
            )
            .into_raw(),
        TransitionOutcome::Denied(PhysicalMutationPreparationDenial::IdempotencyConflict)
    ));
    drop(held);
    serving.close();
}

#[test]
fn rewrite_scope_above_256_kib_is_refused_before_media() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_wide(&root);
    let policy = wide_placement();
    publish(&serving, &policy, 1, 1);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x71; 32]))
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            17,
        )
        .into_raw()
    {
        TransitionOutcome::Denied(denial) => assert!(matches!(
            denial,
            PhysicalMutationPreparationDenial::RecordAppend(RecordAppendDenial::PhysicalPressure)
        )),
        TransitionOutcome::Success(_) => panic!("seventeen pages were admitted"),
        TransitionOutcome::Deferred(_) => panic!("seventeen pages were deferred"),
        TransitionOutcome::Stale(_) => panic!("seventeen pages were stale"),
        TransitionOutcome::RebindRequired(_) => panic!("seventeen pages required rebind"),
        TransitionOutcome::Failed(_) => panic!("seventeen pages failed closed"),
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}

pub(super) fn open_wide(
    root: &std::path::Path,
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
    super::super::success(
        media.initialize_record_store(
            PhysicalRecordInitialization::new(format, wide_placement(), access, durability)
                .with_residency_policy(super::rewrite_axis_residency(format, 1024 * 1024))
                .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
        ),
    )
}

pub(super) fn reopen_wide(
    root: &std::path::Path,
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
    super::super::success(
        media.open_record_store(
            PhysicalRecordOpen::new(format, access, durability)
                .with_residency_policy(super::rewrite_axis_residency(format, 1024 * 1024))
                .with_read_protection_policy(PhysicalReadProtectionPolicy::default()),
        ),
    )
}

pub(super) fn wide_placement() -> worth_store::physical_runtime::AdmittedRecordPlacementPolicy {
    let format = AdmittedPhysicalRecordFormat::admit(
        PhysicalRecordFormatDeclaration::builder().admit().unwrap(),
    );
    PhysicalRecordPlacementPolicy::builder()
        .manifest_capacity(ManifestEntryCapacity::new(4).unwrap())
        .segment_pages(SegmentPageCount::new(16).unwrap())
        .admit(format)
        .unwrap()
}

pub(super) fn publish_span(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    pages: u32,
) -> Vec<PhysicalRecordId> {
    let records: Vec<Vec<u8>> = (0..pages)
        .map(|ordinal| vec![ordinal as u8; RECORD_BYTES])
        .collect();
    let mut material = [0x6A; 32];
    material[..8].copy_from_slice(&u64::from(pages).to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .prepare_durable_append(
            RecordAppendBatch::try_from_iter(records.iter().map(Vec::as_slice)).unwrap(),
            *policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("{pages} records did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("{pages} records denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("{pages} records deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("{pages} records stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("{pages} records rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("{pages} records failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(done) => {
            done.into_acknowledgment().record_ids().collect()
        }
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("{pages} records had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("{pages} records became indeterminate at {:?}", fate.stage())
        }
    }
}

fn publish(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    mark: u8,
) -> PhysicalRecordId {
    let mut bytes = vec![mark; RECORD_BYTES];
    bytes[0] = mark;
    super::append(serving, *policy, ordinal, &bytes)
}

pub(super) fn rewrite(
    serving: &worth_store::physical_runtime::ServingPhysicalRuntime,
    policy: &worth_store::physical_runtime::AdmittedRecordPlacementPolicy,
    ordinal: u64,
    pages: u32,
) {
    let mut material = [0x6B; 32];
    material[..8].copy_from_slice(&ordinal.to_le_bytes());
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    let prepared = match submission
        .rewrite_selected_inline_pages(
            *policy,
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        TransitionOutcome::Success(_) => panic!("{pages} pages did not stay prepared"),
        TransitionOutcome::Denied(denial) => panic!("{pages} pages denied: {denial:?}"),
        TransitionOutcome::Deferred(deferred) => panic!("{pages} pages deferred: {deferred:?}"),
        TransitionOutcome::Stale(stale) => panic!("{pages} pages stale: {stale:?}"),
        TransitionOutcome::RebindRequired(rebind) => panic!("{pages} pages rebind: {rebind:?}"),
        TransitionOutcome::Failed(failure) => panic!("{pages} pages failed: {failure:?}"),
    };
    match prepared.execute() {
        PhysicalMutationOutcome::Completed(_) => {}
        PhysicalMutationOutcome::ProvenNoEffect(fate) => {
            panic!("{pages} pages had no effect: {:?}", fate.cause())
        }
        PhysicalMutationOutcome::Indeterminate(fate) => {
            panic!("{pages} pages became indeterminate at {:?}", fate.stage())
        }
    }
}
