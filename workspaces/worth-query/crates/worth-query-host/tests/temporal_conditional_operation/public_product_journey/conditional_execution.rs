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
pub(crate) fn publishes_delivers_executes_and_cleans_up() {
    let mut world = CourtroomWorld::publish("blocked");
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
        .application_query(TemporalIntentQuery::reference())
        .unwrap();
    let source = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let source_commit = source.selected_commit().clone();
    let source_branch = source.branch_identity().clone();
    let sibling = fork_relational_product(
        &world,
        &source,
        "public-journey-sibling",
        "public-journey-sibling-data",
    );
    let warm_product = world.application.on_product(source).unwrap();
    let scope = warm_product
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);

    let warm = warm_product
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();
    let warm = world
        .application
        .execute_application_query_one_shot(warm)
        .unwrap();
    assert_eq!(warm.rows()[0].input, "payload");
    assert_eq!(
        product_identity(warm.receipt()).selected_commit(),
        &source_commit
    );
    assert_security_work(warm.receipt());
    drop(warm);

    let retained_product = world
        .application
        .select_product_branch(&source_branch)
        .unwrap();
    let retained = retained_product
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(suppressed) = world
        .application
        .select_product_branch(&source_branch)
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe()
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
        .select_product_branch(&sibling)
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
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

    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
    let mut publication = world.change_input_and_conditional_definition_on_product(
        &source_branch,
        "changed-on-a",
        Arc::new(replacement),
    );
    assert_eq!(
        publication
            .committed_product_publication()
            .relational_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication.committed_product_publication().signal_posture(),
        runtime::CompositeComponentChangePosture::Published
    );
    assert_eq!(
        publication
            .committed_product_publication()
            .conditional_definition_generation(),
        Some(2)
    );
    assert!(publication.dispatch_outbox().is_some());
    assert_eq!(
        publication.external_dispatch().unwrap().posture().kind(),
        primary_graph::WorthQueryExternalDispatchPostureKind::Completed
    );
    assert_eq!(transport.contact_count(), 1);
    let performed = publication
        .take_performed_relational_product_change()
        .expect("the combined publication retains its unique Relational delivery");
    let selected_source = world
        .application
        .select_product_branch(&source_branch)
        .unwrap();
    let failed = selected_source
        .deliver_relational_change_to_conditional(&world.clock, usize::MAX, performed)
        .expect_err("an undeclared dependency ordinal must fail before retention");
    assert_eq!(
        failed.kind(),
        runtime::WorthQueryPerformedRelationalProductChangeDeliveryDenialKind::Bridge
    );
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_direct_delivery_count(),
        0
    );
    let delivery = selected_source
        .deliver_relational_change_to_conditional(&world.clock, 0, failed.into_change())
        .expect("the performed patch belongs to the selected product");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("unexpected public delivery posture: {delivery:?}")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    assert_eq!(delivery.signal_seeds_emitted(), 1);
    drop(delivery);
    drop(selected_source);
    assert_eq!(
        world
            .application
            .inspect_conditional_runtime()
            .retained_direct_delivery_count(),
        1
    );
    world.clock_control.push(3, 11);
    let published = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    assert_eq!(published.branch_identity(), &source_branch);
    assert_ne!(published.selected_commit(), &source_commit);
    drop(published);

    let outcome = world
        .application
        .select_product_branch(&source_branch)
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
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
        .select_product_branch(&sibling)
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
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

    for (label, branch, expected_suppressed, expected_clocks) in [
        ("B", sibling.clone(), 1, 3),
        ("new A occurrence", source_branch.clone(), 0, 4),
        ("B-return", sibling.clone(), 1, 4),
    ] {
        let outcome = world
            .application
            .select_product_branch(&branch)
            .unwrap()
            .conditional_clock(&world.clock)
            .unwrap()
            .observe();
        let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
            outcome
        else {
            panic!(
                "a retained product binding must remain observable: {}",
                outcome_kind(&outcome)
            )
        };
        assert_eq!(
            receipt.retained_suppressed_wake_count(),
            expected_suppressed
        );
        assert_eq!(
            world
                .application
                .inspect_conditional_runtime()
                .managed_clock_count(),
            expected_clocks,
            "{label} must create or reuse the binding for its exact product occurrence"
        );
    }

    let assert_product_input = |branch, expected| {
        let selected = world.application.select_product_branch(&branch).unwrap();
        let scope = selected
            .resolve_entity(
                IntentIdentityField::reference(),
                "intent-1".to_string(),
                &request,
                primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access =
            primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
        let plan = selected
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::new(),
                controls(&request),
            )
            .unwrap();
        let result = world
            .application
            .execute_application_query_one_shot(plan)
            .unwrap();
        assert_eq!(result.rows()[0].input, expected);
        assert_eq!(
            product_identity(result.receipt()).branch_identity(),
            &branch
        );
        assert_security_work(result.receipt());
    };
    for (branch, expected) in [
        (sibling.clone(), "payload"),
        (source_branch.clone(), "changed-on-a"),
        (sibling, "payload"),
    ] {
        assert_product_input(branch, expected);
    }
    let retained = world
        .application
        .execute_application_query_one_shot(retained)
        .unwrap();
    assert_eq!(retained.rows()[0].input, "payload");
    assert_eq!(
        product_identity(retained.receipt()).selected_commit(),
        &source_commit
    );
    assert_security_work(retained.receipt());
    drop(retained);
    drop(access);
    drop(scope);
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
    world.application.close_conditional_runtime().unwrap();
    assert_conditional_resources_empty(world.application.inspect_conditional_runtime());
    drop(world);
    let live = probe.live_inventory();
    assert!(
        live.is_empty(),
        "live conditional inventory after cleanup: {live:?}"
    );
}
