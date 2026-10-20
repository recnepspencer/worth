//! A requested upstream resolution does not strand another prepared prerequisite.
use super::*;

fn tail_seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    seed(graph);
    let keys = ["diamond-tail", "diamond-tail-b", "diamond-tail-c"];
    for (key, (x, y)) in keys.into_iter().zip([(0, 0), (9, 0), (0, 9)]) {
        graph
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Body::reference::<CheckpointSchema>(),
                    entity_key(key),
                )
                .field(BodyKey::reference::<CheckpointSchema>(), key.to_owned())
                .field(Length::reference::<CheckpointSchema>(), length(151 + y))
                .field(PositionX::reference::<CheckpointSchema>(), length(150 + x))
                .field(PositionY::reference::<CheckpointSchema>(), length(150 + y)),
            )
            .unwrap();
    }
    for (from, to) in [(0, 1), (1, 2), (2, 0)] {
        graph
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                PlanarSuccessor::reference::<CheckpointSchema>(),
                format!("{}-to-{}", keys[from], keys[to]),
                entity_key(keys[from]),
                entity_key(keys[to]),
            ))
            .unwrap();
    }
}

#[test]
fn a_dropped_prerequisite_is_inspected_after_the_earlier_requested_output_resolves() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        4096,
        128 * 1024 * 1024,
        profile
            .limits()
            .source_currentness_work()
            .map(|work| u64::try_from(work).unwrap()),
        tail_seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut left = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut right = request
        .demand(PlanarOutputDemand::new("diamond-right"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut join = request
        .demand(ChainDemand("diamond-join".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut tail = request
        .demand(ChainDemand("diamond-tail".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(left, request);
    settle!(right, request);
    settle!(join, request);
    settle!(tail, request);
    take_all_decisions();
    change_root!(request, application, "diamond-left", 5, 0x9176_4406_u64);
    change_root!(request, application, "diamond-join", 104, 0x9176_4407_u64);
    change_root!(request, application, "diamond-right", 54, 0x9176_4408_u64);
    settled_in_one_advance!(tail, request, "the caller with two prepared prerequisites");
    assert_eq!(
        take_decisions("diamond-join"),
        [[6, 55]],
        "the earlier member resolves both actual upstreams before it completes"
    );
    assert_eq!(
        output_lengths!(request, ["diamond-left", "diamond-right"]),
        [6, 55]
    );
}
