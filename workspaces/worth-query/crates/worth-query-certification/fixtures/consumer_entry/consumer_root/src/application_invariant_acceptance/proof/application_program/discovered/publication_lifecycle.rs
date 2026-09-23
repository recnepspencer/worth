use super::*;

mod running;
pub(in crate::application_invariant_acceptance::proof::application_program) use running::running_root_supersession_preserves_sibling;

pub(in crate::application_invariant_acceptance::proof::application_program) fn older_publication_starts_after_newer_root_binding(
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
        .idempotency(&10_041)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the first source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(first) = first else {
        panic!("the first source publication is fresh")
    };
    let changed = request
        .query(PlanarRead {
            body_key: "sibling-b".to_owned(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "sibling-b".to_owned(),
            operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "sibling-b".to_owned(),
                replacement_y: length(5),
            }]),
            validator_work: 4_096,
        }))
        .expect_source(changed)
        .idempotency(&10_042)
        .execute_in_program(&world.application)
        .expect("the sibling ring changes independently");
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
        .idempotency(&10_043)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the newer source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(second) = second else {
        panic!("the newer source publication is fresh")
    };
    let mut second = second
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("the newer roots bind: {:?}", failure.denial()));
    let mut first = first
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| {
            panic!("the older unaffected root starts: {:?}", failure.denial())
        });
    let first_settled = loop {
        match first
            .required_output_mut()
            .advance(&request)
            .expect("older roots advance")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        first_settled
            .superseded_roots()
            .map(|root| root.body_key())
            .collect::<Vec<_>>(),
        ["sibling-b", "sibling-c"]
    );
    assert_eq!(
        first_settled
            .root_outputs()
            .map(|(root, _)| root.body_key())
            .collect::<Vec<_>>(),
        ["remote-b"]
    );
    let second_settled = loop {
        match second
            .required_output_mut()
            .advance(&request)
            .expect("newer roots advance")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(second_settled.root_outputs().count(), 3);
    assert_eq!(second_settled.superseded_roots().count(), 0);
    let first_remote = first_settled
        .root_outputs()
        .find(|(root, _)| root.body_key() == "remote-b")
        .expect("the older independent root settles")
        .1
        .receipt()
        .committed_product_publication()
        .composite_commit();
    let second_remote = second_settled
        .root_outputs()
        .find(|(root, _)| root.body_key() == "remote-b")
        .expect("the newer publication joins the independent root")
        .1
        .receipt()
        .committed_product_publication()
        .composite_commit();
    assert_eq!(
        first_remote, second_remote,
        "the newer publication joins the older independent root"
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn unchanged_roots_join_new_publication(
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
        .expect("the authored source exists")
        .observed_sources()[0]
        .clone();
    let first_outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(initial)
        .idempotency(&10_036)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the first source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(first) = first_outcome else {
        panic!("the first source publication is fresh")
    };
    let mut first = first
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("the first roots start: {:?}", failure.denial()));
    let current = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the newer authored source exists")
        .observed_sources()[0]
        .clone();
    let second_outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(current)
        .idempotency(&10_037)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the second source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(second) = second_outcome else {
        panic!("the second source publication is fresh")
    };
    let mut second = second
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("unchanged roots join: {:?}", failure.denial()));
    for output in [&mut first, &mut second] {
        let settled = loop {
            match output
                .required_output_mut()
                .advance(&request)
                .expect("shared roots advance")
            {
                WorthQueryDiscoveredProgramOutputProgress::Pending => {}
                WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
            }
        };
        assert_eq!(settled.root_outputs().count(), 3);
        assert_eq!(settled.output_count(), 6);
    }
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn newer_publication_bounds_abandoned_discovery(
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
    for (revision, idempotency) in [(2, 10_039), (3, 10_040)] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the authored source exists")
            .observed_sources()[0]
            .clone();
        let outcome = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(revision),
            })
            .expect_source(source)
            .idempotency(&idempotency)
            .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
                &world.application,
            )
            .expect("the next source publication succeeds");
        let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
            panic!("each source publication is fresh")
        };
        drop(performed);
        assert_eq!(
            world.application.retained_source_custody_count_for_test(),
            1,
            "abandoned discovery never pins more than its latest source observation"
        );
    }
}
