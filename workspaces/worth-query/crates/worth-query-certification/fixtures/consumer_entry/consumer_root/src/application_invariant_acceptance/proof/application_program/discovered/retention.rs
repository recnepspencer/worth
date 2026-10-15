//! Real committed discovered retention refusal preserves original unregistered custody.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryRequiredOutputPreparationDenial, WorthQueryRequiredOutputRecoveryPosture,
};
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

#[test]
fn discovered_committed_retention_refuses_twice_then_promotes_original_carrier() {
    let installed = crate::installed_schema();
    let world = installation::install(&installed);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4096).unwrap(),
        NonZeroUsize::new(8192).unwrap(),
    );
    macro_rules! publish {
        ($input:expr, $key:expr) => {{
            let source = request
                .query(PlanarRead {
                    body_key: "anchor-a".to_owned(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone();
            request
                .mutate($input)
                .expect_source(source)
                .idempotency(&$key)
                .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
                    &world.application,
                    worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap_or_else(|cause| panic!("source admission: {cause:?}"))
        }};
    }
    let prior = match publish!(
        PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2)
        },
        109_201
    ) {
        WorthQueryApplicationDiscoveredMutationOutcome::Performed(performed) => performed,
        _ => panic!("the real older source commits and registers"),
    };
    let prior_receipt = prior.receipt().clone();
    drop(prior); // genuine zero-token abandoned source, no started demand interests
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(3),
    };
    reset_planar_source_adjustment_contacts(&input);
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let guard = world
        .application
        .refuse_next_discovered_retention_reservation_for_test()
        .unwrap();
    assert!(world
        .application
        .refuse_next_discovered_retention_reservation_for_test()
        .is_err());
    let outcome = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&109_202)
        .execute_performed_discovered::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap_or_else(|cause| panic!("the real source must reach commit: {cause:?}"));
    let mut failure = match outcome {
        WorthQueryApplicationDiscoveredMutationOutcome::RequiredOutputDenied(failure) => failure,
        _ => panic!("the actual eligible reservation must refuse AFTER commit"),
    };
    drop(guard);
    assert!(
        matches!(failure.denial(), WorthQueryRequiredOutputPreparationDenial::DemandExecution(denial) if denial.kind()==WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded)
    );
    assert_eq!(
        failure.recovery_posture(),
        WorthQueryRequiredOutputRecoveryPosture::Retryable
    );
    assert_eq!(failure.result().changed_vertices, 1);
    let receipt = failure.receipt().clone();
    let contacts = planar_source_adjustment_contacts(&input);
    assert_eq!(contacts, (1, 1));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(3)
    );
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert!(
        matches!(request.advanced().recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(&world.application, &receipt, controls), Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(ref denial)) if denial.kind()==WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable)
    );
    // Inspect prior registration, then release the acquired token before retry eligibility.
    let prior_registered = request
        .advanced()
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &prior_receipt,
            controls,
        )
        .unwrap_or_else(|d| panic!("refusal retired prior source: {d:?}"));
    drop(prior_registered);
    let guard = world
        .application
        .refuse_next_discovered_retention_reservation_for_test()
        .unwrap();
    // These pre-entry denials must leave this live arm for the next actual reservation.
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let cancelled_scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            None,
            cancellation.token(),
        );
    let cancelled_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &cancelled_scope,
    ))
    .unwrap();
    cancellation.cancel();
    let cancelled_request = world
        .application
        .request(&cancelled_principal, &cancelled_scope);
    let (cancelled_denial, returned) = cancelled_request
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&109_202)
        .retry_discovered_output_retention::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            failure,
            &world.application,
        )
        .err()
        .expect("cancelled issuer returns the actual original owner");
    assert!(matches!(cancelled_denial, worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial::Recovery(worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial::ExecutionDenied(worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Interrupted(worth_query_host::facade::application_contribution::WorthQueryManagedComputationInterruption::Cancelled)))));

    failure = returned;
    assert_eq!(failure.result().changed_vertices, 1);
    let other = installation::install(&installed);
    let other_adapter = authentication::admit(other.application.installed_schema());
    let other_principal = authentication::block_on(other_adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let other_request = other.application.request(&other_principal, &scope);
    let (foreign_denial, returned) = other_request
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&109_202)
        .retry_discovered_output_retention::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            failure,
            &world.application,
        )
        .err()
        .expect("foreign runtime returns original native custody");
    assert!(matches!(foreign_denial, worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial::Request(worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch)));
    failure = returned;
    // A fresh wrong key cannot substitute a second source. The same failure returns.
    let (_, returned) = request
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&109_203)
        .retry_discovered_output_retention::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            failure,
            &world.application,
        )
        .err()
        .expect("fresh wrong-key admission returns original custody");
    failure = returned;
    assert_eq!(
        failure
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let (denial, returned) = request
        .advanced()
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency(&109_202)
        .retry_discovered_output_retention::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            failure,
            &world.application,
        )
        .err()
        .expect("a second real reservation refusal returns the same carrier");
    assert!(
        matches!(denial, worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial::Recovery(worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial::Demand(ref denial)) if denial.kind()==WorthQueryOutputDemandDenialKind::PublicationCapacityExceeded)
    );
    failure = returned;
    drop(guard);
    assert_eq!(failure.result().changed_vertices, 1);
    assert_eq!(planar_source_adjustment_contacts(&input), contacts);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    let prior_registered = request
        .advanced()
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &prior_receipt,
            controls,
        )
        .unwrap_or_else(|d| panic!("second refusal retired prior source: {d:?}"));
    drop(prior_registered);
    let promoted = request
        .advanced()
        .mutate(input.clone())
        .expect_source(source)
        .idempotency(&109_202)
        .retry_discovered_output_retention::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            failure,
            &world.application,
        )
        .unwrap_or_else(|(d, _)| panic!("unarmed original carrier promotion: {d:?}"));
    assert_eq!(planar_source_adjustment_contacts(&input), contacts);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        promoted
            .receipt()
            .committed_product_publication()
            .composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    let registered = request
        .advanced()
        .recover_discovered_required_outputs::<ConsumerProgram, ConsumerDiscoveredProgramRoot>(
            &world.application,
            &receipt,
            controls,
        )
        .unwrap_or_else(|d| panic!("promotion installed genuine registration: {d:?}"));
    drop(registered);
    let mut started = promoted
        .start_required_outputs(&world.application, &request, controls)
        .unwrap_or_else(|f| panic!("start original discovery: {:?}", f.denial()));
    assert_eq!(started.result().changed_vertices, 1);
    let settled = settle(|| {
        match started
            .required_output_mut()
            .advance(&world.application, &request)
            .unwrap_or_else(|d| panic!("native traversal: {d:?}"))
        {
            WorthQueryDiscoveredProgramOutputProgress::Pending => None,
            WorthQueryDiscoveredProgramOutputProgress::Settled(settled) => Some(settled),
        }
    });
    assert_eq!(
        settled
            .root_outputs()
            .map(|(d, _)| d.body_key())
            .collect::<Vec<_>>(),
        ["remote-b", "sibling-b", "sibling-c"]
    );
    assert_eq!(settled.output_count(), 6);
    for (key, expected) in [("sibling-b", 23), ("sibling-c", 31), ("remote-b", 42)] {
        assert_eq!(
            request
                .at(settled.observation())
                .query(PlanarOutputRead {
                    body_key: key.to_owned()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            length(expected)
        );
    }
    assert_eq!(
        planar_source_adjustment_contacts(&input),
        contacts,
        "retention and outputs never rerun original source"
    );
}
