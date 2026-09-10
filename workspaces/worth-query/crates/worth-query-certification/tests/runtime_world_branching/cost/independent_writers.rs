use std::num::NonZeroUsize;
use std::sync::Arc;

use crate::adapters::ReplacementPredicate;
use crate::query_probe::{assert_ordinary_work, read};
use crate::world::CourtroomWorld;
use worth_query_host::facade::runtime::CompositeComponentChangePosture;

pub(crate) fn assert_sibling_writer_progress_while_combined_is_parked() {
    let world = CourtroomWorld::publish("ready");
    let root = world.application.current_world();
    let branches = (0..2)
        .map(|_| {
            world
                .application
                .branches()
                .fork(root)
                .components(|components| components.fork_relational().fork_signal())
                .create()
                .expect("each independent product branch must be created")
        })
        .collect::<Vec<_>>();
    let parked = branches[0];
    let sibling = branches[1];
    let pause = world
        .application
        .world_operation_control_for_test()
        .pause_before_product_compare(NonZeroUsize::new(1).unwrap());
    let (parked_done, parked_observation) = std::sync::mpsc::sync_channel(1);

    std::thread::scope(|scope| {
        let world = &world;
        let parked_writer = scope.spawn(move || {
            let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
            let receipt = world.change_input_and_conditional_definition_on_branch(
                parked,
                "parked-combined",
                Arc::new(replacement),
                0x31,
            );
            let publication = receipt.committed_product_publication();
            assert_eq!(
                publication.relational_posture(),
                CompositeComponentChangePosture::Published
            );
            assert_eq!(
                publication.signal_posture(),
                CompositeComponentChangePosture::Published
            );
            parked_done.send(()).unwrap();
        });

        assert!(pause.wait_until_reached(std::time::Duration::from_secs(10)));
        let sibling_writer = scope.spawn(move || {
            world
                .change_input_on_branch_with_ordinal(sibling, "sibling-progress", 0x32)
                .require_committed()
                .expect("the sibling writer must commit while the combined writer is parked");
        });
        sibling_writer
            .join()
            .expect("the independent sibling writer must not panic");
        assert_eq!(read(world, sibling).input, "sibling-progress");
        assert!(
            parked_observation.try_recv().is_err(),
            "the combined writer must remain parked after sibling completion"
        );
        pause.release();
        parked_writer
            .join()
            .expect("the parked combined writer must complete after release");
        parked_observation
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("the combined writer must report completion after release");
    });

    assert_eq!(read(&world, parked).input, "parked-combined");
    for branch in branches {
        assert!(world
            .application
            .on_branch(branch)
            .close()
            .unwrap()
            .is_complete());
    }
}

pub(crate) fn assert_independent_writer_slopes() {
    let mut baseline = None;
    for width in [1, 4, 16] {
        let world = CourtroomWorld::publish("ready");
        let root = world.application.current_world();
        let branches = (0..width)
            .map(|_| {
                world
                    .application
                    .branches()
                    .fork(root)
                    .components(|components| components.fork_relational().fork_signal())
                    .create()
                    .expect("each independent writer branch must be created")
            })
            .collect::<Vec<_>>();
        let control = world.application.world_operation_control_for_test();
        let pause = control.pause_before_product_compare(NonZeroUsize::new(width).unwrap());
        let registered = world
            .application
            .pause_after_application_attempt_registration_for_test(
                NonZeroUsize::new(width).unwrap(),
            );
        let work = std::thread::scope(|scope| {
            let writers = branches
                .iter()
                .copied()
                .enumerate()
                .map(|(ordinal, branch)| {
                    let world = &world;
                    scope.spawn(move || {
                        world
                            .change_input_on_branch_with_ordinal(
                                branch,
                                &format!("writer-{ordinal}"),
                                ordinal as u8 + 1,
                            )
                            .require_committed()
                            .expect("each independent public writer must commit")
                    })
                })
                .collect::<Vec<_>>();
            assert!(registered.wait_until_reached(std::time::Duration::from_secs(10)));
            assert_eq!(
                world.application.active_application_attempts_for_test(),
                width
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
            while world.application.active_application_attempts_for_test() != width
                && std::time::Instant::now() < deadline
            {
                std::thread::yield_now();
            }
            let active_writers = world.application.active_application_attempts_for_test();
            let active_world = world
                .application
                .world_active_publication_attempts_for_test();
            eprintln!(
                "axis=IndependentWriters requested={width} actual={active_writers} A_world={active_world} covariation={width} exact product branches progress concurrently"
            );
            assert_eq!(active_writers, width, "actual W population");
            assert_eq!(
                active_world, width,
                "every independent World publication reaches the parked boundary"
            );
            let work = read(&world, root).work;
            pause.release();
            for writer in writers {
                writer.join().expect("independent writer must not panic");
            }
            work
        });
        assert_ordinary_work(work);
        assert_eq!(*baseline.get_or_insert(work), work, "W={width}");
        for (ordinal, branch) in branches.iter().copied().enumerate() {
            let observed = read(&world, branch);
            assert_eq!(observed.input, format!("writer-{ordinal}"));
            assert_ordinary_work(observed.work);
        }
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
