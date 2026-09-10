use std::num::NonZeroUsize;

use crate::query_probe::{assert_ordinary_work, read};
use crate::world::CourtroomWorld;

pub(crate) fn assert_active_attempt_slopes() {
    let mut baseline = None;
    for size in [1, 8, 64] {
        let world = CourtroomWorld::publish("ready");
        let root = world.application.current_world();
        let branches = (0..size)
            .map(|_| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| components.fork_relational().fork_signal())
                    .create()
                    .expect("each active-attempt branch must be created")
            })
            .collect::<Vec<_>>();
        let control = world.application.world_operation_control_for_test();
        let pause = control.pause_before_product_compare(NonZeroUsize::new(size).unwrap());
        let registered = world
            .application
            .pause_after_application_attempt_registration_for_test(
                NonZeroUsize::new(size).unwrap(),
            );
        let work = std::thread::scope(|scope| {
            let attempts = branches
                .iter()
                .copied()
                .enumerate()
                .map(|(ordinal, branch)| {
                    let world = &world;
                    scope.spawn(move || {
                        world.change_input_on_branch_with_ordinal(
                            branch,
                            &format!("active-attempt-{ordinal}"),
                            ordinal as u8 + 1,
                        )
                    })
                })
                .collect::<Vec<_>>();
            assert!(registered.wait_until_reached(std::time::Duration::from_secs(10)));
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
            registered.release();
            assert!(pause.wait_until_reached(std::time::Duration::from_secs(10)));
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while world.application.active_application_attempts_for_test() != size
                && std::time::Instant::now() < deadline
            {
                std::thread::yield_now();
            }
            let actual = world.application.active_application_attempts_for_test();
            let publishing = world
                .application
                .world_active_publication_attempts_for_test();
            eprintln!(
                "axis=ActiveAttempts requested={size} actual={actual} covariation={size} independent product/component branches; {publishing} concurrent World publications",
            );
            assert_eq!(actual, size);
            assert_eq!(publishing, size);
            let work = read(&world, root).work;
            pause.release();
            for attempt in attempts {
                attempt
                    .join()
                    .expect("active attempt must not panic")
                    .require_committed()
                    .expect("each independent active attempt must commit");
            }
            work
        });
        assert_ordinary_work(work);
        assert_eq!(*baseline.get_or_insert(work), work, "A={size}");
        for branch in branches {
            assert!(world
                .application
                .on_branch(branch)
                .close()
                .unwrap()
                .is_complete());
        }
    }
}
