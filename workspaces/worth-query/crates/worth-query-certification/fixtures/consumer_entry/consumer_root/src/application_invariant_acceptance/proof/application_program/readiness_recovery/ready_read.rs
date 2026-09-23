use super::*;

pub(crate) fn ready_read_capacity_preserves_completion(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let mut program = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the declared program starts its output");
    assert!(matches!(
        program.settle(&request).expect("the program output settles"),
        WorthQueryApplicationProgramOutputProgress::Settled(_)
    ));
    drop(program);
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start()
        .expect("the actual output demand starts");
    let first = (0..32)
        .find_map(
            |_| match demand.advance(&request).expect("the output completes") {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
            },
        )
        .expect("the output settles within the bounded progression");
    let committed = first
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    let readiness_attempts = world.application.output_readiness_attempt_count_for_test();
    assert_eq!(
        request
            .at(first.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the ready output is readable")
            .rows()[0]
            .value,
        length(2)
    );
    drop(first);

    world
        .application
        .press_next_ready_read_with_world_snapshots_for_test();
    let cause = match demand.advance(&request) {
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause)) => cause,
        Err(other) => panic!("ready read denied for an unexpected reason: {other:?}"),
        Ok(_) => panic!("a saturated World must deny a new ready-output read"),
    };
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(
            WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted
        ),
    );
    assert_eq!(
        cause.recovery_posture(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
    );

    let reopened = match demand
        .advance(&request)
        .expect("released capacity admits the same ready output")
    {
        WorthQueryApplicationOutputDemandProgress::Settled(value) => value,
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("ready output cannot restart delivery or validation")
        }
    };
    assert_eq!(
        reopened
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        &committed,
        "opening another read cannot republish the output"
    );
    assert_eq!(
        world.application.output_readiness_attempt_count_for_test(),
        readiness_attempts,
        "opening a ready output cannot validate it again",
    );
    assert_eq!(
        request
            .at(reopened.observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the reopened read observes the same output")
            .rows()[0]
            .value,
        length(2)
    );
}
