#[allow(unused_imports)]
use crate::physical_record_journeys::maintenance_interference::rewrite_scope::*;

#[test]
fn rewrite_scope_selects_one_four_and_sixteen_source_pages() {
    for pages in [1_u32, 4, 16] {
        let parent = tempfile::tempdir().unwrap();
        let root = parent.path().join("store");
        let serving = open_wide(&root);
        let policy = wide_placement();
        let ids = publish_span(&serving, &policy, pages);
        let before = worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads(&root).len();
        rewrite(&serving, &policy, 100 + u64::from(pages), pages);
        let found = worth_store_testkit_store::physical_record_journeys::durability_admission::independent_wal_oracle::rewrite_redo::produced_rewrite_payloads(&root);
        assert_eq!(found.len(), before + 1);
        assert_eq!(
            u64::from(found[found.len() - 1].source_length),
            u64::from(pages) * PAGE_BYTES
        );
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
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new(material),
        )
        .unwrap();
    let held = match submission
        .rewrite_selected_inline_pages(
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key.clone(),
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
            1,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Success(
            worth_store::physical_runtime::PhysicalMutationPreparationSuccess::Prepared(prepared),
        ) => prepared,
        worth_proof::TransitionOutcome::Denied(denial) => {
            panic!("one-page rewrite denied: {denial:?}")
        }
        worth_proof::TransitionOutcome::Deferred(_) => panic!("one-page rewrite deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("one-page rewrite stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => panic!("one-page rewrite rebind"),
        worth_proof::TransitionOutcome::Failed(_) => panic!("one-page rewrite failed"),
        worth_proof::TransitionOutcome::Success(_) => {
            panic!("one-page rewrite did not stay prepared")
        }
    };
    assert!(matches!(
        submission
            .rewrite_selected_inline_pages(
                policy,
                worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                    key,
                    worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(
                        30_000
                    )
                    .unwrap(),
                ),
                4,
            )
            .into_raw(),
        worth_proof::TransitionOutcome::Denied(
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::IdempotencyConflict
        )
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
        .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite);
    let submission = serving.record_submission();
    let key = submission
        .issue_idempotency_key(
            worth_store::physical_runtime::PhysicalMutationIdempotencyMaterial::new([0x71; 32]),
        )
        .unwrap();
    match submission
        .rewrite_selected_inline_pages(
            policy,
            worth_store::physical_runtime::PhysicalMutationRequest::platform_durable(
                key,
                worth_store::physical_runtime::PhysicalMutationDeadline::after_milliseconds(30_000)
                    .unwrap(),
            ),
            17,
        )
        .into_raw()
    {
        worth_proof::TransitionOutcome::Denied(denial) => assert!(matches!(
            denial,
            worth_store::physical_runtime::PhysicalMutationPreparationDenial::RecordAppend(
                worth_store::physical_runtime::RecordAppendDenial::PhysicalPressure
            )
        )),
        worth_proof::TransitionOutcome::Success(_) => panic!("seventeen pages were admitted"),
        worth_proof::TransitionOutcome::Deferred(_) => panic!("seventeen pages were deferred"),
        worth_proof::TransitionOutcome::Stale(_) => panic!("seventeen pages were stale"),
        worth_proof::TransitionOutcome::RebindRequired(_) => {
            panic!("seventeen pages required rebind")
        }
        worth_proof::TransitionOutcome::Failed(_) => panic!("seventeen pages failed closed"),
    }
    assert_eq!(
        serving
            .media_counters()
            .attempts_for(worth_store_physical_backend::MediaOperationRole::PositionedWrite),
        writes
    );
    serving.close();
}
