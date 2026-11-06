//! Real retirement, reclamation and recreated Source identity cross both prepared roots.
use super::super::authoring::Edit;
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn retired_shared_source_is_removed_and_reclaimed_generation_does_not_inherit_membership() {
    lifecycle::run(journey);
}

fn journey() {
    let provider = Arc::new(prefix_observation::Calls::default());
    let rules = Arc::new(rule::Calls::new(Arc::clone(&provider)));
    let world = host::install(Arc::clone(&provider), Arc::clone(&rules));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(find("A", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let original = request.query(find("A", "sh")).execute().unwrap();
    let coordinate = original.rows()[0].coordinate;
    let stale = original.into_result_set_observation().unwrap();
    let result = request
        .advanced()
        .mutate(Edit::RetireItem {
            directory: "A".into(),
            item: "x".into(),
        })
        .idempotency(&0x919320u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    drop(result);
    assert!(request
        .query(find("A", ""))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    // B was never publicly opened; deletion must discover its real old membership.
    {
        let records = rules.records.lock().unwrap();
        assert_eq!(records.len(), 2);
        assert!(records
            .iter()
            .any(|record| values(&record.source.after) == vec![("y".into(), "taken".into())]));
        assert!(records.iter().any(|record| record.source.after.is_empty()));
    }
    rules.records.lock().unwrap().clear();
    let stopped = request
        .advanced()
        .mutate(rename("sh", "wrong"))
        .expect_result_set(stale)
        .idempotency(&0x919321u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = stopped.unwrap_err()
    else {
        panic!(
            "retired complete Source must refuse for currentness before resolving the missing item"
        )
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
    drop(cause);
    assert!(rules.records.lock().unwrap().is_empty());
    let reclaimed = world
        .runtime()
        .with_application_advancement(&scope, |phase| {
            world.runtime().try_reclaim_retired_storage(&phase).unwrap()
        })
        .unwrap();
    assert_eq!(
        reclaimed.entity_reclaimed, 1,
        "Native reclaims the actual retired record"
    );
    let result = request
        .advanced()
        .mutate(Edit::Item {
            directory: "A".into(),
            identity: "x".into(),
            name: "reborn".into(),
        })
        .idempotency(&0x919322u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    drop(result);
    let replacement = request.query(find("A", "")).execute().unwrap();
    assert_eq!(
        values(replacement.rows()),
        vec![("x".into(), "reborn".into())]
    );
    let new_coordinate = replacement.rows()[0].coordinate;
    assert_eq!(
        (coordinate.0, coordinate.1),
        (new_coordinate.0, new_coordinate.1),
        "Native reused the actual partition and slot"
    );
    assert!(
        new_coordinate.2 > coordinate.2,
        "the owner issued a newer generation"
    );
    drop(replacement);
    let builds = provider.builds.load(Ordering::SeqCst);
    let b = request
        .query(find("B", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(
        values(request.query(find("B", "")).execute().unwrap().rows()),
        vec![("y".into(), "taken".into())],
        "the retained dangling edge to the old generation cannot reattach the replacement"
    );
    assert_eq!(
        provider.builds.load(Ordering::SeqCst),
        builds,
        "B attaches the published maintained root"
    );
    let result = request
        .advanced()
        .mutate(Edit::Share {
            directory: "B".into(),
            item: "x".into(),
        })
        .idempotency(&0x919323u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
    assert_eq!(
        values(request.query(find("B", "")).execute().unwrap().rows()),
        expected("B", "reborn", "")
    );
    a.close();
    b.close();
}
