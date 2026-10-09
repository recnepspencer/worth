use super::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial;
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;

pub(in crate::application_invariant_acceptance::proof::application_program) fn owned_outputs_outlive_requests_and_reject_foreign_and_cancelled_advance(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let other = installation::install(foreign);
    let outputs = {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone();
        let outcome = request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length(2),
            })
            .expect_source(source)
            .idempotency(&10_050)
            .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
                &world.application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        let WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) = outcome else {
            panic!("the real source mutation publishes once")
        };
        performed
            .start_required_outputs(
                &world.application,
                &request,
                WorthQueryOutputDemandControls::new(
                    NonZeroUsize::new(4096).unwrap(),
                    NonZeroUsize::new(8192).unwrap(),
                ),
            )
            .unwrap_or_else(|failure| panic!("source setup: {:?}", failure.denial()))
    };
    // The principal, request and original scope above have all been dropped.
    let (receipt, result, mut continuation) = outputs.into_parts();
    assert_eq!(result.changed_vertices, 1);
    let other_scope = authentication::request_scope();
    let other_adapter = authentication::admit(other.application.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &other_scope,
    ))
    .unwrap();
    let other_request = other.application.request(&other_principal, &other_scope);
    let error = continuation
        .advance(&other.application, &other_request)
        .err()
        .expect("the same schema/program in another runtime cannot advance owner custody");
    assert!(
        matches!(error, WorthQueryRequiredOutputPreparationDenial::DemandExecution(ref denial)
        if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignSource)
    );
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    assert_eq!(
        other.application.retained_source_custody_count_for_test(),
        0
    );

    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let branch = world
        .application
        .branches()
        .fork(world.application.current_world())
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .unwrap();
    let wrong_branch = world
        .application
        .request(&principal, &scope)
        .on_branch(branch);
    let error = continuation
        .advance(&world.application, &wrong_branch)
        .err()
        .unwrap();
    assert!(matches!(error,
        WorthQueryRequiredOutputPreparationDenial::DemandExecution(ref denial)
        if denial.kind() == WorthQueryOutputDemandDenialKind::ForeignSource));
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    world.application.on_branch(branch).close().unwrap();
    world
        .application
        .delay_next_output_readiness_delivery_for_test();
    assert!(matches!(
        continuation.advance(&world.application, &request).unwrap(),
        WorthQueryDiscoveredProgramOutputProgress::Pending
    ));
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "real admission consumed the discovered root identities"
    );
    let attempts = world.application.output_readiness_attempt_count_for_test();
    cancellation.cancel();
    let error = continuation
        .advance(&world.application, &request)
        .err()
        .unwrap();
    assert!(
        matches!(error, WorthQueryRequiredOutputPreparationDenial::DemandExecution(ref denial)
        if denial.kind() == WorthQueryOutputDemandDenialKind::Cancelled)
    );
    assert_eq!(
        world.application.output_readiness_attempt_count_for_test(),
        attempts
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );

    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh_request = world.application.request(&fresh_principal, &fresh_scope);
    let settled = settle(|| {
        match continuation
            .advance(&world.application, &fresh_request)
            .unwrap()
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => None,
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        settled
            .root_outputs()
            .map(|(root, _)| root.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.output_count(), 6);
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    assert_eq!(
        fresh_request
            .at_commit(&receipt, NonZeroUsize::new(128).unwrap())
            .unwrap()
            .query(PlanarRead {
                body_key: "anchor-a".to_owned()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
}
