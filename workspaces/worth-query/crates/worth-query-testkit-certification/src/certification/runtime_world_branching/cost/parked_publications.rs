//! Hold exact Application and World writer populations without overlapping native cutovers.
use std::num::NonZeroUsize;
use std::sync::{Arc, OnceLock};

use super::{CourtroomWorld, QueryWork};
use worth_query_host::facade::product::WorthQueryProductBranch;

pub(super) fn with_parked_writers(
    world: &CourtroomWorld,
    branches: &[WorthQueryProductBranch],
    input_prefix: &str,
    observe: impl FnOnce() -> QueryWork,
) -> QueryWork {
    let size = branches.len();
    let control = world.application.world_operation_control_for_test();
    std::thread::scope(|scope| {
        let mut attempts = Vec::new();
        let mut comparisons = Vec::new();
        let first_comparison =
            Arc::new(control.pause_before_product_compare_for_test(NonZeroUsize::MIN));
        let mut comparison_slots = Vec::with_capacity(size);
        comparison_slots.push(Arc::new(OnceLock::from(Arc::clone(&first_comparison))));
        comparison_slots.resize_with(size, || Arc::new(OnceLock::new()));
        comparisons.push(first_comparison);
        let observed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (ordinal, branch) in branches.iter().copied().enumerate() {
                let registered = Arc::new(
                    world
                        .application
                        .pause_after_application_attempt_registration_for_test(NonZeroUsize::MIN),
                );
                let comparison = Arc::clone(&comparison_slots[ordinal]);
                let worker_registration = Arc::clone(&registered);
                let worker_comparison = Arc::clone(&comparison);
                let attempt = scope.spawn(move || {
                    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        world.change_input_on_branch_with_ordinal(
                            branch,
                            &format!("{input_prefix}-{ordinal}"),
                            ordinal as u8 + 1,
                        )
                    }));
                    // An early refusal wakes its coordinator before cleanup
                    // releases every other retained attempt.
                    worker_registration.release();
                    if let Some(pause) = worker_comparison.get() {
                        pause.release();
                    }
                    outcome.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
                });
                attempts.push((Arc::clone(&registered), comparison, attempt));
                assert!(registered.wait_for_arrivals());
            }
            assert_eq!(
                world.application.active_application_attempts_for_test(),
                size
            );
            assert_eq!(
                world
                    .application
                    .world_active_publication_attempts_for_test(),
                0
            );
            // Memory-only native cutovers may legitimately defer on their
            // shared patch-position reservation. Stage those cutovers, then
            // retain every real World attempt at its own comparison pause.
            for (ordinal, (registered, comparison, _)) in attempts.iter().enumerate() {
                let pause = if ordinal == 0 {
                    Arc::clone(&comparisons[0])
                } else {
                    let pause =
                        Arc::new(control.pause_before_product_compare_for_test(NonZeroUsize::MIN));
                    assert!(comparison.set(Arc::clone(&pause)).is_ok());
                    comparisons.push(Arc::clone(&pause));
                    pause
                };
                registered.release();
                assert!(pause.wait_for_arrivals());
                assert_eq!(
                    world.application.active_application_attempts_for_test(),
                    size
                );
                assert_eq!(
                    world
                        .application
                        .world_active_publication_attempts_for_test(),
                    ordinal + 1
                );
            }
            observe()
        }));
        // Release before resuming a coordinator panic: thread::scope joins
        // its children before the outer pause guards can be dropped.
        for (registered, _, _) in &attempts {
            registered.release();
        }
        for pause in comparisons {
            pause.release();
        }
        // Preserve the worker's typed early refusal before a coordinator
        // arrival assertion can hide the operation that released its peers.
        for (_, _, attempt) in attempts {
            attempt
                .join()
                .expect("active attempt must not panic")
                .require_committed()
                .expect("each independent active attempt must commit");
        }
        observed.unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}
