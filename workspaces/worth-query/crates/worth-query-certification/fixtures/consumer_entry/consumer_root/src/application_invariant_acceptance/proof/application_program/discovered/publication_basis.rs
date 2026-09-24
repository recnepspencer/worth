use super::*;
use worth_query_topology_entry::PlanarOutputToLateFinalConnection;

pub(in crate::application_invariant_acceptance::proof::application_program) fn joined_roots_discover_at_their_own_publication(
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
    .expect("the source owner authenticates");
    let request = world.application.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let initial = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let first = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(initial)
        .idempotency(&10_047)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the first source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(first) = first else {
        panic!("the first source publication is fresh")
    };
    let mut first = first
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("the first roots start: {:?}", failure.denial()));
    let first_settled = loop {
        match first
            .required_output_mut()
            .advance(&request)
            .expect("first program advances")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(first_settled.root_outputs().count(), 3);
    assert_eq!(
        first_settled
            .outputs_for::<ConsumerSchema, PlanarOutputToLateFinalConnection>()
            .map(|(demand, _)| demand.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "remote-b", "remote-b"]
    );
    let current = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let second = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(current)
        .idempotency(&10_048)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the later source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(second) = second else {
        panic!("the later source publication is fresh")
    };
    let second_commit = second
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .ordinal();
    let mut second = second
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("unchanged roots join: {:?}", failure.denial()));
    let second_settled = loop {
        match second
            .required_output_mut()
            .advance(&request)
            .expect("later program advances")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    for ((first_demand, first_root), (second_demand, second_root)) in first_settled
        .root_outputs()
        .zip(second_settled.root_outputs())
    {
        assert_eq!(first_demand.body_key(), second_demand.body_key());
        assert_eq!(
            first_root
                .application_commit_receipt()
                .expect("the completed first root retains its commit")
                .committed_product_publication()
                .composite_commit(),
            second_root
                .application_commit_receipt()
                .expect("the joined root retains its commit")
                .committed_product_publication()
                .composite_commit(),
            "the later publication joins the older completed root output"
        );
    }
    let children = second_settled
        .outputs_for::<ConsumerSchema, PlanarOutputToLateFinalConnection>()
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 3);
    for (demand, settled) in children {
        assert_eq!(
            demand.body_key(),
            "sibling-b",
            "dependent discovery reads the later anchor edit"
        );
        assert!(settled.observation().selected_commit().ordinal() >= second_commit);
    }
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
