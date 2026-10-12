use std::sync::Arc;
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph, runtime,
};

use super::super::adapters::{CompletingExternalTransport, ReplacementPredicate};
use super::super::courtroom_lifecycle::assert_conditional_resources_empty;
use super::super::courtroom_support::outcome_kind;
use super::super::product_query_support::{
    assert_security_work, controls, fork_relational_product, principal, product_identity,
};
use super::super::schema::*;
use super::super::world::{self, CourtroomWorld};
pub(crate) fn publishes_delivers_executes_and_cleans_up(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let world = CourtroomWorld::publish("blocked");
    let transport = Arc::new(CompletingExternalTransport::default());
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let probe = world.application.conditional_runtime_lifecycle_probe();
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .managed_clock_count(),
        0
    );
    let request = world::request_scope();
    let principal = principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let source = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let source_commit = source.product().selected_commit().clone();
    let source_branch = source.product().product_branch();
    let sibling = fork_relational_product(
        &world,
        &source,
        "public-journey-sibling",
        "public-journey-sibling-data",
    );
    let warm_product = source;
    let scope = world
        .application
        .with_host_advancement(|phase| {
            warm_product
                .resolve_entity(
                    &phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_string(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);

    let warm = world
        .application
        .with_host_advancement(|phase| {
            warm_product
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls(&request),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let warm = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, warm)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(warm.rows()[0].input, "payload");
    assert_eq!(
        product_identity(warm.receipt()).selected_commit(),
        &source_commit
    );
    assert_security_work(warm.receipt());
    drop(warm);

    let retained_product = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(source_branch)
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let retained = world
        .application
        .with_host_advancement(|phase| {
            retained_product
                .admit_application_query(
                    &phase,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    controls(&request),
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(suppressed) = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(source_branch)
                .select(&phase)
                .unwrap()
                .conditional_clock(
                    &phase
                        .execution_request_for(world.application.product_runtime())
                        .expect("phase belongs to application"),
                    &world.clock,
                )
                .unwrap()
        })
        .expect("installed host call opens")
        .observe(request_execution)
    else {
        panic!("the initial blocked observation must be accepted")
    };
    assert_eq!(suppressed.retained_suppressed_wake_count(), 1);
    drop(suppressed);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .managed_clock_count(),
        1
    );

    let sibling_warm = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling)
                .select(&phase)
                .unwrap()
                .conditional_clock(
                    &phase
                        .execution_request_for(world.application.product_runtime())
                        .expect("phase belongs to application"),
                    &world.clock,
                )
                .unwrap()
        })
        .expect("installed host call opens")
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(sibling_warm) =
        sibling_warm
    else {
        panic!(
            "the sibling D0 projection must warm while its Signal basis is current: {}",
            outcome_kind(&sibling_warm)
        )
    };
    assert_eq!(sibling_warm.retained_suppressed_wake_count(), 1);
    drop(sibling_warm);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .managed_clock_count(),
        2
    );

    publication_delivery::publish_and_deliver(&world, source_branch, &transport);
    world.clock_control.push(3, 11);
    let published = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(published.product().product_branch(), source_branch);
    assert_ne!(published.product().selected_commit(), &source_commit);
    drop(published);

    let outcome = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(source_branch)
                .select(&phase)
                .unwrap()
                .conditional_clock(
                    &phase
                        .execution_request_for(world.application.product_runtime())
                        .expect("phase belongs to application"),
                    &world.clock,
                )
                .unwrap()
        })
        .expect("installed host call opens")
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(mut executed) =
        outcome
    else {
        panic!(
            "the post-publication observation must execute the conditional: {}",
            outcome_kind(&outcome)
        )
    };
    assert_eq!(executed.committed_operation_count(), 1);
    assert_eq!(executed.authoritative_commit_count(), 1);
    let [provenance] = executed.execution_provenance() else {
        panic!("the public conditional must retain one exact execution provenance")
    };
    assert_eq!(
        provenance.signal_decision(),
        Some(primary_graph::WorthQueryConditionalSignalDecision::Eligible)
    );
    let batch = executed.take_granular_invalidation_batch();
    assert_eq!(batch.observation().direct_truth_delivery_count(), 1);
    assert_eq!(batch.observation().signal_performed_delivery_count(), 1);
    let [delivery] =
        batch
            .into_bridge_deliveries()
            .try_into()
            .unwrap_or_else(|deliveries: Vec<_>| {
                panic!(
                    "expected one committed Bridge patch, found {}",
                    deliveries.len()
                )
            });
    assert_eq!(delivery.truth().change_set().changes().len(), 1);
    delivery
        .performed_signal()
        .expect("the public journey must carry Signal owner-service evidence");
    drop(delivery);
    drop(executed);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_direct_delivery_count(),
        0
    );
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .managed_clock_count(),
        3
    );

    let sibling_outcome = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(sibling)
                .select(&phase)
                .unwrap()
                .conditional_clock(
                    &phase
                        .execution_request_for(world.application.product_runtime())
                        .expect("phase belongs to application"),
                    &world.clock,
                )
                .unwrap()
        })
        .expect("installed host call opens")
        .observe(request_execution);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(sibling_progress) =
        sibling_outcome
    else {
        panic!(
            "the D0 sibling must continue independently: {}",
            outcome_kind(&sibling_outcome)
        )
    };
    assert_eq!(sibling_progress.retained_suppressed_wake_count(), 1);
    drop(sibling_progress);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .managed_clock_count(),
        3
    );

    retained_bindings::assert_retained_bindings(&world, request_execution, source_branch, sibling);

    for (branch, expected) in [
        (sibling, "payload"),
        (source_branch, "changed-on-a"),
        (sibling, "payload"),
    ] {
        product_readback::assert_product_input(&world, &request, &principal, branch, expected);
    }
    let retained = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot(&phase, retained)
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(retained.rows()[0].input, "payload");
    assert_eq!(
        product_identity(retained.receipt()).selected_commit(),
        &source_commit
    );
    assert_security_work(retained.receipt());
    drop(retained);
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    cleanup::assert_closed(world, &probe);
}

#[path = "conditional_execution/cleanup.rs"]
mod cleanup;

#[path = "conditional_execution/product_readback.rs"]
mod product_readback;

#[path = "conditional_execution/publication_delivery.rs"]
mod publication_delivery;

#[path = "conditional_execution/retained_bindings.rs"]
mod retained_bindings;
