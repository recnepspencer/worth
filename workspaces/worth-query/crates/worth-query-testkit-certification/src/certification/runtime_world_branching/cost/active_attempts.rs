pub use crate::facade::certification::query_probe::{assert_ordinary_work, read};
pub use worth_query_testkit_host::facade::host::world::CourtroomWorld;

pub fn assert_active_attempt_slopes() {
    let mut baseline = None;
    for size in [1, 8, 64] {
        let world = CourtroomWorld::publish("ready");
        let root = world.application.current_world();
        let branches = (0..size)
            .map(|_| {
                world
                    .application
                    .with_host_advancement(|host_phase| {
                        world
                            .application
                            .branches()
                            .fork(root)
                            .components(|components| components.fork_relational().fork_signal())
                            .create(&host_phase)
                            .expect("each active-attempt branch must be created")
                    })
                    .expect("installed host call opens")
            })
            .collect::<Vec<_>>();
        let work = super::parked_publications::with_parked_writers(
            &world,
            &branches,
            "active-attempt",
            || {
                let actual = world.application.active_application_attempts_for_test();
                let publishing = world
                    .application
                    .world_active_publication_attempts_for_test();
                eprintln!(
                    "axis=ActiveAttempts requested={size} actual={actual} covariation={size} independent product/component branches; {publishing} concurrent World publications",
                );
                assert_eq!(actual, size);
                assert_eq!(publishing, size);
                read(&world, root).work
            },
        );
        assert_ordinary_work(work);
        assert_eq!(*baseline.get_or_insert(work), work, "A={size}");
        for branch in branches {
            assert!(world
                .application
                .with_host_advancement(|host_phase| world
                    .application
                    .on_branch(branch)
                    .close(&host_phase)
                    .unwrap()
                    .is_complete())
                .expect("installed host call opens"));
        }
    }
}
