pub(crate) use worth_query_testkit_host::facade::host::temporal_conditional_operation::retained_query_batch::direct_limits::*;

#[test]
fn direct_and_retained_batch_reads_consume_exact_installed_slots() {
    let world = CourtroomWorld::publish("ready");

    let head = &world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .expect("installed host call opens");
    for first_retained in [false, true] {
        let batch = batch(&world);
        let first = execute(&world, &batch, first_retained).unwrap();
        assert_eq!(first.result().rows()[0].input, "payload");

        assert!(matches!(
            execute(&world, &batch, !first_retained),
            Err(ReadDenial::Resource(Resource::UnplannedRead))
        ));

        assert_eq!(
            &&world
                .application
                .with_host_advancement(|phase| world
                    .application
                    .on_branch(world.application.current_world())
                    .select(&phase)
                    .unwrap()
                    .product()
                    .selected_commit()
                    .clone())
                .expect("installed host call opens"),
            &head
        );
        drop(first);

        assert!(
            matches!(
                execute(&world, &batch, false),
                Err(ReadDenial::Resource(Resource::UnplannedRead))
            ),
            "dropping a result cannot reopen an exhausted finite schedule"
        );
    }
}
