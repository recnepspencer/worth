use super::*;

pub(in crate::application_invariant_acceptance::proof::application_program) fn required_recovery_cannot_claim_discovered_custody(
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
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source exists")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_038)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the discovered publication retains its own custody");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the discovered source publication is fresh")
    };
    let receipt = performed.receipt().clone();
    drop(performed);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let denial = request
        .recover_required_outputs::<ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            &receipt,
            PlanarOutputDemand::new("sibling-b"),
            controls,
        )
        .err()
        .expect("a required root cannot claim discovered custody");
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial)
    ) = denial else {
        panic!("the cross-kind request preserves the custody denial")
    };
    assert_eq!(
        denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );
    let mut recovered = request
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &receipt,
            controls,
        )
        .expect("the correct root kind still recovers the same receipt");
    let settled = loop {
        match recovered
            .advance(&request)
            .expect("discovered roots advance")
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(settled.root_outputs().count(), 3);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn newer_discovered_source_retires_recovery(
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
    let original = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the authored source is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(original)
        .idempotency(&10_032)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the source publication prepares discovered roots");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication is fresh")
    };
    let receipt = performed.receipt().clone();
    drop(performed);
    let changed = request
        .query(PlanarRead {
            body_key: "sibling-b".to_owned(),
        })
        .execute()
        .expect("the discovered source is readable")
        .observed_sources()[0]
        .clone();
    request
        .mutate(PlanarMutation {
            scope_key: "sibling-b".to_owned(),
            operation: PlanarOperation::Adjust(vec![PlanarAdjustment {
                body_key: "sibling-b".to_owned(),
                replacement_y: length(5),
            }]),
            validator_work: 4_096,
        })
        .expect_source(changed)
        .idempotency(&10_033)
        .execute()
        .expect("an ordinary edit advances the discovered source");
    let mut recovered = request
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &receipt,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .expect("the unaffected sibling remains recoverable");
    let settled = loop {
        match recovered.advance(&request).expect("the sibling advances") {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(
        settled
            .superseded_roots()
            .map(|root| root.body_key())
            .collect::<Vec<_>>(),
        ["sibling-b", "sibling-c"]
    );
    assert_eq!(
        settled
            .root_outputs()
            .map(|(root, _)| root.body_key())
            .collect::<Vec<_>>(),
        ["remote-b"]
    );
    assert_eq!(settled.output_count(), 2);
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn foreign_runtime_cannot_recover_discovered_source(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let owner = installation::install(foreign);
    let other = installation::install(foreign);
    let scope = authentication::request_scope();
    let owner_adapter = authentication::admit(owner.application.installed_schema());
    let owner_principal = authentication::block_on(owner_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the owner authenticates");
    let owner_request = owner.application.request(&owner_principal, &scope);
    let source = owner_request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the owner source exists")
        .observed_sources()[0]
        .clone();
    let outcome = owner_request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_034)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &owner.application,
        )
        .expect("the owner publishes a source");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the owner publication is fresh")
    };
    let receipt = performed.receipt().clone();
    drop(performed);
    let other_adapter = authentication::admit(other.application.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the other runtime authenticates");
    let other_request = other.application.request(&other_principal, &scope);
    let other_source = other_request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the second runtime has the same seeded source")
        .observed_sources()[0]
        .clone();
    let other_outcome = other_request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(other_source)
        .idempotency(&10_034)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &other.application,
        )
        .expect("the second runtime publishes a matching authored edit");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(other_performed) = other_outcome
    else {
        panic!("the second runtime publication is fresh")
    };
    drop(other_performed);
    let denial = other_request
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &other.application,
            &receipt,
            WorthQueryOutputDemandControls::new(
                NonZeroUsize::new(4_096).unwrap(),
                NonZeroUsize::new(8_192).unwrap(),
            ),
        )
        .err()
        .expect("another runtime cannot adopt owner custody");
    assert_eq!(
        denial.recovery_posture(),
        worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal,
    );
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial) = denial else {
        panic!("foreign recovery must preserve the custody denial")
    };
    assert_eq!(denial.kind(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
}

pub(in crate::application_invariant_acceptance::proof::application_program) fn interrupted_discovery_recovers_both_consumed_roots(
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
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_035)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
        )
        .expect("the two-root source publication succeeds");
    let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
        panic!("the source publication is fresh")
    };
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let mut started = performed
        .start_required_outputs(&request, controls)
        .unwrap_or_else(|failure| panic!("the two roots start: {:?}", failure.denial()));
    let receipt = started.receipt().clone();
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "both root identities consumed their exact prepared custody"
    );
    assert!(matches!(
        started.required_output_mut().advance(&request).unwrap(),
        WorthQueryDiscoveredProgramOutputProgress::Pending
    ));
    drop(started);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1,
        "interrupted output work keeps one recoverable source publication"
    );
    let mut recovered = request
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &receipt,
            controls,
        )
        .expect("the exact source receipt re-enters both admitted roots");
    let settled = loop {
        match recovered.advance(&request).expect("both roots recover") {
            WorthQueryDiscoveredProgramOutputProgress::Pending => {}
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => break settled,
        }
    };
    assert_eq!(settled.root_outputs().count(), 3);
    assert_eq!(settled.output_count(), 6);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
