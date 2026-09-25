use super::*;

pub(in crate::application_invariant_acceptance::proof::application_program) fn running_root_supersession_preserves_sibling(
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
        .idempotency(&10_044)
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
    assert!(matches!(
        first.required_output_mut().advance(&request).unwrap(),
        WorthQueryDiscoveredProgramOutputProgress::Pending
    ));
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
        .idempotency(&10_045)
        .execute_in_program(&world.application)
        .expect("the sibling ring changes while the first handle is active");
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
        .idempotency(&10_046)
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
    loop {
        match second
            .required_output_mut()
            .advance(&request)
            .expect("newer roots advance")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => {
                assert_eq!(settled.root_outputs().count(), 3);
                break;
            }
        }
    }
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
