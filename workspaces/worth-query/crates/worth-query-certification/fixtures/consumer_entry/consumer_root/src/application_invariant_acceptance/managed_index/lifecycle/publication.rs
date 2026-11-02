//! A real Native effect cannot make a successor cache visible through its old World.
use super::super::mutation;
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

#[test]
fn public_cache_native_ahead_of_world_keeps_selected_predecessor_then_requires_rebuild() {
    run(pending_world);
}
fn pending_world() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&index).unwrap();
    let before = request.retain_current_read().unwrap();
    let retained = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    let expectation = request
        .query(find("al"))
        .execute()
        .unwrap()
        .into_result_set_observation()
        .unwrap();
    let rename = mutation::Rename {
        directory: "catalog".into(),
        prefix: "al".into(),
        item: "item-a".into(),
        replacement: "alba".into(),
    };
    let key = 0x919cace1u64;
    let pause = world
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let original_scope = WorthQueryRequestScope::new(None, cancellation.token());
    let outcome = std::thread::scope(|threads| {
        let inspected_world = &world;
        let inspected_index = &index;
        let inspector = threads.spawn(move || {
            // Assertion failure releases the real pause before the caller joins.
            let pause = pause;
            assert!(
                pause.wait_until_reached(Duration::from_secs(10)),
                "real ordinary mutation reaches World compare after Native publication"
            );
            assert_eq!(
                inspected_world.world_active_publication_attempts_for_test(),
                1
            );
            assert_eq!(
                inspected_index.lifecycle(),
                WorthQueryManagedIndexLifecycle::Evicted,
                "Native's actual successor is cold before any late World updater could run"
            );
            cancellation.cancel();
            pause.release();
        });
        let result = world
            .runtime()
            .request(&principal, &original_scope)
            .on_branch(world.current_world())
            .advanced()
            .mutate(rename.clone())
            .expect_result_set(expectation.clone())
            .idempotency(&key)
            .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation);
        inspector.join().unwrap();
        result.unwrap()
    });
    let unpublished = match outcome {
        WorthQueryApplicationMutationOutcome::Commit(
            WorthQueryApplicationUncommitted::ProductUnpublished(unpublished),
        ) => unpublished,
        other => {
            panic!("performed Native mutation retains its unpublished World custody: {other:?}")
        }
    };
    assert_eq!(unpublished.owner_effect_count(), 1);
    let recovery = unpublished.into_recovery();
    assert_eq!(world.world_active_publication_attempts_for_test(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    let actual = request.query(find("al")).execute().unwrap();
    assert_eq!(
        actual.rows(),
        journey::expected(retained.rows(), "al"),
        "fresh current reads reborrow the actual predecessor selected by the old World"
    );
    let totals = calls.totals();
    let recovered = request
        .advanced()
        .mutate(rename)
        .expect_result_set(expectation)
        .idempotency(&key)
        .recover_unpublished_in_program(&recovery, &world)
        .unwrap();
    assert!(matches!(
        recovered,
        WorthQueryManagedApplicationRecoveryOutcome::Performed(_)
    ));
    assert_eq!(
        calls.totals(),
        totals,
        "adoption runs neither provider reconstruction nor selection"
    );
    assert_ne!(
        request.retain_current_read().unwrap().selected_commit(),
        before.selected_commit()
    );
    not_ready(
        request
            .query(find("al"))
            .execute()
            .err()
            .expect("successor cache is cold"),
    );
    assert_eq!(
        calls.totals(),
        totals,
        "a cold successor refuses before provider callbacks"
    );
    request.query(find("al")).rebuild_index(&index).unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    let actual = request.query(find("al")).execute().unwrap();
    assert_eq!(actual.rows(), journey::expected(source.rows(), "al"));
    assert!(actual
        .rows()
        .iter()
        .any(|row| row.identity == "item-a" && row.name == "alba"));
    assert!(retained
        .rows()
        .iter()
        .any(|row| row.identity == "item-a" && row.name == "beta"));
    index.close();
}
