#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::span_layout::*;
use {std::path::Path};

use {worth_proof::TransitionOutcome};
use {worth_store::physical_runtime::PhysicalMutationDeadline, worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial, worth_store::physical_runtime::PhysicalMutationPreparationDenial, worth_store::physical_runtime::PhysicalMutationRequest, worth_store::physical_runtime::RecordAppendDenial, worth_store::physical_runtime::RecordByteLimit, worth_store::physical_runtime::RecordReadLimits, worth_store::physical_runtime::ServingPhysicalRuntime};
use {worth_store_physical_backend::MediaOperationRole};

use {crate::physical_record_journeys::maintenance_interference::rewrite_scope::open_wide, crate::physical_record_journeys::maintenance_interference::rewrite_scope::publish_span, crate::physical_record_journeys::maintenance_interference::rewrite_scope::reopen_wide, crate::physical_record_journeys::maintenance_interference::rewrite_scope::rewrite, crate::physical_record_journeys::maintenance_interference::rewrite_scope::wide_placement, crate::physical_record_journeys::maintenance_interference::rewrite_scope::RECORD_BYTES};
use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::segment_files};

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
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let denial = denied_rewrite(&serving, 8);
    assert!(
        matches!(
            denial,
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(worth_store::physical_runtime::RecordAppendDenial::RewriteSpanNotLive)
        ),
        "eight pages reach past the four-frame destination generation: {denial:?}"
    );
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes
    );
    assert_eq!(segment_files(&root), after);
    serving.close();
}
