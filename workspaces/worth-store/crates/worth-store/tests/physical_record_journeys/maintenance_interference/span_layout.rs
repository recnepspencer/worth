use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationDenial, PhysicalMutationRequest, RecordAppendDenial,
    RecordByteLimit, RecordReadLimits, ServingPhysicalRuntime,
};
use worth_store_physical_backend::MediaOperationRole;

use super::rewrite_scope::{
    open_wide, publish_span, reopen_wide, rewrite, wide_placement, RECORD_BYTES,
};
use super::segment_files;

const PAGE_BYTES: u64 = 16 * 1024;

#[test]
fn a_span_after_a_live_prefix_becomes_a_compact_generation_beside_it() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let serving = open_wide(&root);
    let policy = wide_placement();
    let ids = publish_span(&serving, &policy, 8);
    let before = segment_files(&root);
    assert_eq!(
        before.len(),
        1,
        "eight pages publish one segment generation"
    );

    rewrite(&serving, &policy, 300, 4);

    let after = segment_files(&root);
    assert!(
        before.is_subset(&after),
        "the source generation still holds the live prefix"
    );
    let created = after.difference(&before).collect::<Vec<_>>();
    assert_eq!(
        created.len(),
        1,
        "the span publishes one destination generation"
    );
    assert_eq!(
        file_len(&root, created[0]),
        4 * PAGE_BYTES,
        "the destination holds only the span, from frame 0"
    );
    assert_eq!(file_len(&root, before.first().unwrap()), 8 * PAGE_BYTES);
    read_all(&serving, &ids);
    serving.close();

    let serving = reopen_wide(&root);
    read_all(&serving, &ids);
    let writes = serving
        .media_counters()
        .attempts_for(MediaOperationRole::PositionedWrite);
    let denial = denied_rewrite(&serving, 8);
    assert!(
        matches!(
            denial,
            PhysicalMutationPreparationDenial::RecordAppend(RecordAppendDenial::RewriteSpanNotLive)
        ),
        "eight pages reach past the four-frame destination generation: {denial:?}"
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(MediaOperationRole::PositionedWrite),
        writes
    );
    assert_eq!(segment_files(&root), after);
    serving.close();
}

fn read_all(
    serving: &ServingPhysicalRuntime,
    ids: &[worth_store::physical_runtime::PhysicalRecordId],
) {
    let limits = RecordReadLimits::new(RecordByteLimit::new(RECORD_BYTES as u32).unwrap());
    let reader = serving.records().unwrap();
    for (ordinal, id) in ids.iter().copied().enumerate() {
        let mut session = reader.open(id, limits).unwrap();
        let chunk = session.next_chunk().unwrap().unwrap();
        assert_eq!(chunk.bytes(), vec![ordinal as u8; RECORD_BYTES].as_slice());
    }
}

fn denied_rewrite(
    serving: &ServingPhysicalRuntime,
    pages: u32,
) -> PhysicalMutationPreparationDenial {
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new([0x7D; 32]))
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            wide_placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(30_000).unwrap(),
            ),
            pages,
        )
        .into_raw()
    {
        TransitionOutcome::Denied(denial) => denial,
        TransitionOutcome::Success(_) => panic!("{pages} pages were admitted"),
        TransitionOutcome::Deferred(_) => panic!("{pages} pages were deferred"),
        TransitionOutcome::Stale(_) => panic!("{pages} pages were stale"),
        TransitionOutcome::RebindRequired(_) => panic!("{pages} pages required rebind"),
        TransitionOutcome::Failed(_) => panic!("{pages} pages failed closed"),
    }
}

fn file_len(root: &Path, name: &str) -> u64 {
    std::fs::metadata(root.join("families/records/segments").join(name))
        .unwrap()
        .len()
}
