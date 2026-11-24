use super::*;

// Exact request grants and canonical eviction are private L6 observers.
#[test]
fn authored_write_generation_funds_nested_postings_and_refuses_evicted_canonical_evidence() {
    let host = crate::test_execution_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let runtime = runtime_with_test_schema();
    let created = create_entity_outcome(&execution, &runtime, "before");
    let entity = changed_entities(&created)[0];
    let original = test_owner_main_basis(&runtime).unwrap();
    let equal = update_entity(&execution, &runtime, entity, "before");
    let changed = update_entity(&execution, &runtime, entity, "after");
    let target = test_owner_main_basis(&runtime).unwrap();
    {
        let generation = runtime
            .history
            .authored_write_generation(target.observation().commit_id().unwrap())
            .unwrap();
        assert!(
            generation
                .prefix
                .accounted_bytes()
                .is_some_and(|bytes| bytes > 0),
            "serial prefix cells retain their original request grants"
        );
        assert!(
            generation
                .writes
                .accounted_bytes()
                .is_some_and(|bytes| bytes > 0),
            "serial key cells retain their original request grants"
        );
        assert!(
            generation
                .writes
                .values()
                .all(|postings| postings.0.accounted_bytes().is_some_and(|bytes| bytes > 0)),
            "nested posting cells are funded too"
        );
    }
    runtime
        .discard_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    runtime
        .rebuild_authored_write_generation(execution, selected(&runtime, &target))
        .unwrap();
    let mut unavailable = cursor(&runtime, execution, &original, &target, entity);
    assert!(runtime
        .history_authority()
        .evict_commit_envelope_for_durable_recovery_test(equal.commit.commit_id));
    for _ in 0..2 {
        assert!(
            matches!(
                unavailable.next_page(&runtime, execution, limits(1)),
                Err(RelationalAuthoredWriteDenial::CanonicalEvidenceUnavailable)
            ),
            "explicit owner eviction never becomes an empty page or advanced continuation"
        );
    }
    for outcome in [&created, &equal, &changed] {
        release_test_commit_snapshot(&runtime, outcome);
    }
}
