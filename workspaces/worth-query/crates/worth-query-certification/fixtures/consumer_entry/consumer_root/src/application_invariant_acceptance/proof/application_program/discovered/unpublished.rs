//! Actual append failure, original source recovery and one-use output promotion.

use super::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorthQueryApplicationRecoveryRequestDenial, WorthQueryDiscoveredRecoveryProgress,
    },
    primary_graph::{
        WorthQueryApplicationIdempotencyResolution, WorthQueryManagedApplicationRecoveryDenial,
    },
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

pub(in crate::application_invariant_acceptance::proof::application_program) fn original_partial_recovers_discovered_outputs(
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
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let original = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    let source = original.observed_sources()[0].clone();
    let intent = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    let key = 10_060;
    reset_planar_source_adjustment_contacts(&intent);
    world.application.fail_next_durable_append_for_test();
    let outcome = request
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&key)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::ProductUnpublished(mut recovery) = outcome
    else {
        panic!("the actual append fault must retain the discovered native partial")
    };
    assert_eq!(
        recovery.initial_cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert!(recovery.performed().is_none());
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0,
        "unpublished effects have not manufactured a registered output source"
    );

    macro_rules! original_request {
        ($scope:expr, $input:expr, $key:expr) => {
            world
                .application
                .request(&principal, $scope)
                .mutate($input)
                .expect_source(source.clone())
                .idempotency($key)
        };
    }
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(120),
        cancellation.token(),
    );
    cancellation.cancel();
    assert!(matches!(
        original_request!(&stopped, intent.clone(), &key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application),
        Err(WorthQueryApplicationRecoveryRequestDenial::Recovery(worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled))))
    ));
    let fresh_scope = authentication::request_scope();
    let wrong_key = key + 1;
    let wrong_input = PlanarSourceAdjustment {
        replacement_y: length(3),
        ..intent.clone()
    };
    for (input, request_key) in [(wrong_input, &key), (intent.clone(), &wrong_key)] {
        assert!(original_request!(&fresh_scope, input, request_key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application)
            .is_err());
        assert!(recovery.performed().is_none());
        assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    }
    assert!(
        world
            .application
            .request(&principal, &fresh_scope)
            .mutate(intent.clone())
            .expect_result_set(
                original
                    .result_set_observation()
                    .expect("the completed original query retains its full result-set proof")
                    .clone(),
            )
            .idempotency(&key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application)
            .is_err(),
        "the same query's complete result set cannot replace its original row proof"
    );
    assert!(recovery.performed().is_none());
    // The same schema/program in another real runtime does not own this partial.
    let other = installation::install(foreign);
    let other_adapter = authentication::admit(other.application.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    assert!(other
        .application
        .request(&other_principal, &fresh_scope)
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&key)
        .recover_unpublished_discovered_in_program(&mut recovery, &other.application)
        .is_err());
    assert!(recovery.performed().is_none());

    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    // Promotion cannot synthesize a performed witness from an unpublished owner.
    let (denial, retained) = original_request!(&fresh_scope, intent.clone(), &key)
        .promote_recovered_discovered_outputs(recovery, &world.application, controls)
        .err()
        .expect("unpublished output promotion must refuse and return custody");
    assert!(matches!(
        denial,
        WorthQueryApplicationRecoveryRequestDenial::Recovery(
            WorthQueryManagedApplicationRecoveryDenial::BindingMismatch
        )
    ));
    recovery = retained;
    assert_eq!(
        original_request!(&fresh_scope, intent.clone(), &key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application)
            .unwrap(),
        WorthQueryDiscoveredRecoveryProgress::Performed
    );
    assert!(recovery.performed().is_some());
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    // A refused fresh keyed read must return the complete performed phase too.
    let (_, retained) = original_request!(&fresh_scope, intent.clone(), &wrong_key)
        .promote_recovered_discovered_outputs(recovery, &world.application, controls)
        .err()
        .expect("wrong-key promotion must preserve the actual performed owner");
    recovery = retained;
    assert!(recovery.performed().is_some());
    let changed = world
        .application
        .request(&principal, &fresh_scope)
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(changed.rows()[0].y, length(2));
    let (_, retained) = world
        .application
        .request(&principal, &fresh_scope)
        .mutate(intent.clone())
        .expect_source(changed.observed_sources()[0].clone())
        .idempotency(&key)
        .promote_recovered_discovered_outputs(recovery, &world.application, controls)
        .err()
        .expect("the newly observed current source cannot replace the original source proof");
    recovery = retained;
    assert!(recovery.performed().is_some());
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    let (mut outputs, initial_cause, performed, prior_cleanup) =
        original_request!(&fresh_scope, intent.clone(), &key)
            .promote_recovered_discovered_outputs(recovery, &world.application, controls)
            .unwrap_or_else(|(denial, _)| panic!("original output promotion: {denial:?}"))
            .into_parts();
    assert_eq!(initial_cause, ProductUnpublishedCause::SettlementPending);
    assert!(prior_cleanup.is_empty());
    let (read, publication_failure, cleanup_failure) = performed.into_parts();
    assert!(publication_failure.is_none());
    assert!(cleanup_failure.is_none());
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) =
        read.unwrap().into_resolution()
    else {
        panic!("native recovery must publish the exact original keyed receipt")
    };
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );
    let settled = settle(
        || match outputs.advance(&world.application, &request).unwrap() {
            WorthQueryDiscoveredProgramOutputProgress::Pending => None,
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => Some(settled),
        },
    );
    assert_eq!(
        settled.source_observation().selected_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        settled
            .root_outputs()
            .map(|(root, _)| root.body_key().to_owned())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.output_count(), 6);
    for (body_key, expected) in [("sibling-b", 22), ("sibling-c", 31), ("remote-b", 42)] {
        assert_eq!(
            request
                .at(settled.observation())
                .query(PlanarOutputRead {
                    body_key: body_key.to_owned(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(expected)
        );
    }
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
