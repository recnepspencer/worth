use super::*;

pub(crate) fn preserved_noop_output_completes_readiness_without_a_signal_successor(
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
    .expect("the program application authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let mut initial = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerSecondaryProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the initial output demand starts");
    loop {
        match initial
            .advance(&request)
            .expect("the initial output settles")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(_) => break,
        }
    }
    drop(initial);

    request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the preserved output source remains uniquely resolvable");

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source including its related vertex is observed");
    let outcome = request
        .mutate(PlanarEdit(adjust("anchor-b", 5, 4_096)))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&10_021)
        .execute_in_program(&world.application)
        .expect("the related dependency changes");
    assert!(matches!(
        outcome,
        worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. }
    ));

    let mut preserved = request
        .start_program_outputs::<crate::ConsumerProgram, crate::ConsumerSecondaryProgramRoot>(
            &world.application,
            PlanarOutputDemand::new("anchor-a"),
            controls(),
        )
        .expect("the drifted source selects its preserve producer");
    let settlement = loop {
        match preserved
            .advance(&request)
            .expect("an exact preserved no-op still completes readiness")
        {
            WorthQueryApplicationProgramOutputProgress::Pending => {}
            WorthQueryApplicationProgramOutputProgress::Settled(settlement) => break settlement,
        }
    };
    let delivery = settlement
        .root_readiness_delivery()
        .expect("preserved output readiness carries delivery evidence");
    assert!(
        !delivery.has_conditional_successor(),
        "the proof must exercise the no-successor preserve path"
    );
    assert_eq!(
        request
            .at(settlement.root_observation())
            .query(PlanarOutputRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the preserved output remains readable")
            .rows()[0]
            .value,
        length(2)
    );
    let committed = settlement
        .root_receipt()
        .expect("the produced root retains its commit")
        .committed_product_publication()
        .composite_commit()
        .clone();
    let readiness_attempts = world.application.output_readiness_attempt_count_for_test();
    drop(settlement);
    drop(preserved);
    let mut ready = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(controls())
        .start()
        .expect("the preserved output remains in ready custody");
    world
        .application
        .press_next_ready_read_with_world_snapshots_for_test();
    let cause = match ready.advance(&request) {
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause)) => cause,
        Err(other) => panic!("no-change ready read denied unexpectedly: {other:?}"),
        Ok(_) => panic!("real World capacity must deny the no-change ready read"),
    };
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ProductSelection(
            WorthQueryProductBranchAdmissionDenial::ObservationCapacityExhausted
        )
    );
    let reopened = match ready
        .advance(&request)
        .expect("the no-change ready output reopens")
    {
        WorthQueryApplicationOutputDemandProgress::Pending => {
            panic!("no-change readiness must not rerun")
        }
        WorthQueryApplicationOutputDemandProgress::Settled(value) => value,
    };
    assert_eq!(
        reopened
            .application_commit_receipt()
            .expect("the reopened output retains its commit")
            .committed_product_publication()
            .composite_commit(),
        &committed,
    );
    assert!(!reopened
        .readiness_delivery()
        .unwrap()
        .has_conditional_successor());
    assert_eq!(
        world.application.output_readiness_attempt_count_for_test(),
        readiness_attempts,
    );
    ready.close();
}
