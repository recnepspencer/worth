//! Actual append failure, original source recovery and one-use output promotion.

use super::super::settle;
use super::*;
use crate::ConsumerProgram;
use std::time::{Duration, Instant};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationProgramOutputProgress,
};
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestInterruption, WorthQueryRequestScope,
    },
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
use worth_query_topology_entry::{
    PlanarAlternateFinalToSummaryConnection, PlanarFinalToSummaryConnection, PlanarOutputRead,
    PlanarOutputToAlternateFinalConnection, PlanarOutputToFinalConnection,
};

pub(in crate::application_invariant_acceptance::proof::application_program) fn original_partial_recovers_required_outputs(
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
    let key = 10_062;
    reset_planar_source_adjustment_contacts(&intent);
    world.application.fail_next_durable_append_for_test();
    let outcome = request
        .mutate(intent.clone())
        .expect_source(source.clone())
        .idempotency(&key)
        .execute_performed::<ConsumerProgram, crate::ConsumerProgramRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationPerformedMutationOutcome::ProductUnpublished(mut recovery) = outcome
    else {
        panic!("the actual append fault must retain the required-output native partial")
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
            .recover_unpublished_required_in_program(&mut recovery, &world.application),
        Err(WorthQueryApplicationRecoveryRequestDenial::Interrupted(
            WorthQueryRequestInterruption::Cancelled
        ))
    ));
    let fresh_scope = authentication::request_scope();
    let wrong_key = key + 1;
    let wrong_input = PlanarSourceAdjustment {
        replacement_y: length(3),
        ..intent.clone()
    };
    for (input, request_key) in [(wrong_input, &key), (intent.clone(), &wrong_key)] {
        assert!(original_request!(&fresh_scope, input, request_key)
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
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
                    .expect("completed query retains full result-set proof")
                    .clone()
            )
            .idempotency(&key)
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
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
        .recover_unpublished_required_in_program(&mut recovery, &other.application)
        .is_err());
    assert!(recovery.performed().is_none());

    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    // Promotion cannot synthesize a performed witness from an unpublished owner.
    let (denial, retained) = original_request!(&fresh_scope, intent.clone(), &key)
        .promote_recovered_required_outputs(recovery, &world.application)
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
            .recover_unpublished_required_in_program(&mut recovery, &world.application)
            .unwrap(),
        WorthQueryDiscoveredRecoveryProgress::Performed
    );
    assert!(recovery.performed().is_some());
    assert_eq!(planar_source_adjustment_contacts(&intent), (1, 1));
    // A refused fresh keyed read must return the complete performed phase too.
    let (_, retained) = original_request!(&fresh_scope, intent.clone(), &wrong_key)
        .promote_recovered_required_outputs(recovery, &world.application)
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
        .promote_recovered_required_outputs(recovery, &world.application)
        .err()
        .expect("the newly observed current source cannot replace the original source proof");
    recovery = retained;
    assert!(recovery.performed().is_some());
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    let (preparation, initial_cause, performed, prior_cleanup) =
        original_request!(&fresh_scope, intent.clone(), &key)
            .promote_recovered_required_outputs(recovery, &world.application)
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
    let source_commit = preparation
        .receipt()
        .committed_product_publication()
        .composite_commit()
        .clone();
    assert_eq!(
        &source_commit,
        receipt.committed_product_publication().composite_commit()
    );
    // A fresh request starts the same typed fixed demand; no original result is invented.
    let mut outputs = {
        let start_scope = authentication::request_scope();
        let start_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &start_scope,
        ))
        .unwrap();
        let start_request = world.application.request(&start_principal, &start_scope);
        preparation
            .start_required_outputs(&world.application, &start_request, controls)
            .unwrap_or_else(|(_, denial)| panic!("fixed recovered source starts: {denial:?}"))
    };
    // The fixed continuation outlives its starting request and retains its prefix
    // through genuine foreign-runtime and canceled fresh-request refusals.
    let foreign_request = other.application.request(&other_principal, &fresh_scope);
    assert!(
        matches!(outputs.advance(&other.application, &foreign_request),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref denial)))
        if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource)
    );
    let stopped_request = world.application.request(&principal, &stopped);
    assert!(
        matches!(outputs.advance(&world.application, &stopped_request),
        Err(worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref denial)))
        if denial.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::Cancelled)
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );
    let fresh_request = world.application.request(&principal, &fresh_scope);
    assert_eq!(
        outputs.source_observation().selected_commit(),
        &source_commit
    );
    let settled = settle(
        || match outputs.advance(&world.application, &fresh_request).unwrap() {
            WorthQueryApplicationProgramOutputProgress::Pending => None,
            WorthQueryApplicationProgramOutputProgress::Settled(settled) => Some(settled),
        },
    );
    // ConsumerProgramRoot declares two branches, each with a final and summary edge.
    assert_eq!(settled.output_count(), 4);
    assert_eq!(
        [
            settled
                .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
                .count(),
            settled
                .outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>()
                .count(),
            settled
                .outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>()
                .count(),
            settled
                .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
                .count(),
        ],
        [1; 4],
        "recovery must settle every declared dependent edge exactly once"
    );
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
    for (body_key, expected) in [("anchor-a", 3), ("final:anchor-a", 4)] {
        assert_eq!(
            fresh_request
                .at(settled.observation())
                .query(PlanarOutputRead {
                    body_key: body_key.to_owned()
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
