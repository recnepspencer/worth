//! Actual Native-performed/World-pending custody cannot publish prepared wakes.
use super::*;
use std::{num::NonZeroUsize, time::Duration};

pub(super) fn run(
    world: &host::Host,
    principal: &WorthQueryAuthenticatedExternalPrincipal<super::super::super::schema::Catalog>,
    scope: &WorthQueryRequestScope,
    calls: &prefix::Calls,
) {
    let request = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world());
    let (initial, mut watch) = request
        .query(find("A", "race"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let source = initial.result_set_observation().unwrap().clone();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let pause = world
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let edit = rename("race", "recovered");
    let key = 0x9198_0400u64;
    let outcome = std::thread::scope(|threads| {
        let inspector = threads.spawn(move || {
            let pause = pause;
            assert!(
                pause.wait_until_reached(Duration::from_secs(10)),
                "real Native effect reaches World compare"
            );
            assert_eq!(world.world_active_publication_attempts_for_test(), 1);
            cancellation.cancel();
            pause.release();
        });
        let result = world
            .runtime()
            .request(principal, &stopped)
            .on_branch(world.current_world())
            .advanced()
            .mutate(edit.clone())
            .expect_result_set(source.clone())
            .idempotency(&key)
            .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        inspector.join().unwrap();
        result
    });
    let WorthQueryApplicationMutationOutcome::Commit(
        WorthQueryApplicationUncommitted::ProductUnpublished(unpublished),
    ) = outcome
    else {
        panic!("World failure retains performed Native custody: {outcome:?}");
    };
    assert_eq!(unpublished.owner_effect_count(), 1);
    let recovery = unpublished.into_recovery();
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert!(matches!(
        request
            .query(find("A", "race"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    assert!(
        request
            .query(find("A", "race"))
            .watch_index::<prefix::PrefixIndex>()
            .is_err(),
        "pending preparation excludes a new opening on its predecessor"
    );
    let recovered = request
        .advanced()
        .mutate(edit.clone())
        .expect_result_set(source.clone())
        .idempotency(&key)
        .recover_unpublished_in_program(&recovery, world)
        .unwrap();
    assert!(matches!(
        recovered,
        WorthQueryManagedApplicationRecoveryOutcome::Performed(_)
    ));
    let WorthQueryManagedIndexWatchOutcome::Delivered(update) = request
        .query(find("A", "race"))
        .next_watch(&mut watch)
        .unwrap()
    else {
        panic!("matching real adoption publishes its retained before-membership wake");
    };
    assert!(update.read().rows().is_empty());
    assert_eq!(
        update.notification().composite_commit(),
        request.retain_current_read().unwrap().selected_commit()
    );
    assert!(matches!(
        request
            .query(find("A", "race"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    let completed_head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let maintenance = (
        calls.builds.load(Ordering::SeqCst),
        calls.updates.load(Ordering::SeqCst),
    );
    let replay = request
        .advanced()
        .mutate(edit)
        .expect_result_set(source.clone())
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            replay,
            WorthQueryApplicationMutationOutcome::AlreadyCommitted(_)
                | WorthQueryApplicationMutationOutcome::PreviouslyCommitted(_)
        ),
        "completed replay must reach actual recorded resolution: {replay:?}"
    );
    let changed = request
        .advanced()
        .mutate(rename("race", "wrong-replay"))
        .expect_result_set(source)
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::IdempotencyIntentDrift
        ),
        "{changed:?}"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &completed_head
    );
    assert_eq!(
        (
            calls.builds.load(Ordering::SeqCst),
            calls.updates.load(Ordering::SeqCst)
        ),
        maintenance,
        "recorded replay and drift cannot publish another provider root"
    );
    assert!(
        matches!(
            request
                .query(find("A", "race"))
                .next_watch(&mut watch)
                .unwrap(),
            WorthQueryManagedIndexWatchOutcome::Pending
        ),
        "recorded resolution cannot enqueue another occurrence"
    );
    assert_eq!(values(initial.rows()), vec![("x".into(), "race".into())]);
    watch.close().unwrap();
}
