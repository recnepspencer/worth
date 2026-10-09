//! Real NoSource declaration, append fault and original output-carrier recovery.

use super::*;
use crate::application_program::ConsumerInitialDiscoveredProgramRoot;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryDiscoveredRecoveryProgress,
    },
    primary_graph::WorthQueryApplicationIdempotencyResolution,
    runtime::{ExecutionAllocationPolicy, ProductUnpublishedCause},
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
    PlanarInitialAdjustment,
};

pub(in crate::application_invariant_acceptance::proof::application_program) fn initial_partial_recovers_discovered_outputs(
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
    let intent = PlanarInitialAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    let observed_input = PlanarSourceAdjustment {
        scope_key: intent.scope_key.clone(),
        replacement_y: intent.replacement_y,
    };
    let key = 10_061;
    reset_planar_source_adjustment_contacts(&observed_input);
    world.application.fail_next_durable_append_for_test();
    let outcome = request
        .mutate(intent.clone())
        .without_source()
        .idempotency(&key)
        .execute_performed_discovered::<ConsumerProgram, ConsumerInitialDiscoveredProgramRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::ProductUnpublished(mut recovery) = outcome
    else {
        panic!("a real NoSource append fault must preserve its original discovered preparation")
    };
    assert_eq!(
        recovery.initial_cause(),
        ProductUnpublishedCause::SettlementPending
    );
    assert_eq!(planar_source_adjustment_contacts(&observed_input), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
    let fresh_scope = authentication::request_scope();
    let fresh_request = world.application.request(&principal, &fresh_scope);
    assert_eq!(
        fresh_request
            .mutate(intent.clone())
            .without_source()
            .idempotency(&key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application)
            .unwrap(),
        WorthQueryDiscoveredRecoveryProgress::Performed
    );
    // Re-entering a performed phase cannot take the one-use native carrier again.
    assert_eq!(
        fresh_request
            .mutate(intent.clone())
            .without_source()
            .idempotency(&key)
            .recover_unpublished_discovered_in_program(&mut recovery, &world.application)
            .unwrap(),
        WorthQueryDiscoveredRecoveryProgress::Performed
    );
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    );
    let (mut outputs, initial_cause, performed, prior_cleanup) = fresh_request
        .mutate(intent.clone())
        .without_source()
        .idempotency(&key)
        .promote_recovered_discovered_outputs(recovery, &world.application, controls)
        .unwrap_or_else(|(denial, _)| panic!("initial source promotion: {denial:?}"))
        .into_parts();
    assert_eq!(initial_cause, ProductUnpublishedCause::SettlementPending);
    let (read, publication_failure, cleanup_failure) = performed.into_parts();
    assert!(publication_failure.is_none());
    assert!(cleanup_failure.is_none());
    assert!(prior_cleanup.is_empty());
    let WorthQueryApplicationIdempotencyResolution::AlreadyCommitted(receipt) =
        read.unwrap().into_resolution()
    else {
        panic!("native recovery must publish the exact original NoSource key")
    };
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );
    // An ordinary duplicate returns its actual receipt, never another prepared
    // source or another handler result that could remint discovered custody.
    let repeated = fresh_request
        .mutate(intent)
        .without_source()
        .idempotency(&key)
        .execute_performed_discovered::<ConsumerProgram, ConsumerInitialDiscoveredProgramRoot>(
            &world.application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationDiscoveredMutationOutcome::NotPerformed(
        WorthQueryApplicationMutationOutcome::AlreadyCommitted(repeated_receipt),
    ) = repeated
    else {
        panic!("a consumed original source can only replay its keyed receipt")
    };
    assert_eq!(
        repeated_receipt
            .committed_product_publication()
            .composite_commit(),
        receipt.committed_product_publication().composite_commit()
    );
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        1
    );
    let settled = settle(
        || match outputs.advance(&world.application, &fresh_request).unwrap() {
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
            fresh_request
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
    assert_eq!(planar_source_adjustment_contacts(&observed_input), (1, 1));
    assert_eq!(
        world.application.retained_source_custody_count_for_test(),
        0
    );
}
