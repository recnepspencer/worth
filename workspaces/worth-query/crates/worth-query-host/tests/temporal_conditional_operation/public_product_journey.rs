use std::sync::Arc;

use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet, primary_graph, product, runtime,
};

use super::adapters::ReplacementPredicate;
use super::courtroom_support::outcome_kind;
use super::product_query_support::{
    controls, fork_relational_product, principal, product_identity,
};
use super::schema::*;
use super::world::{self, CourtroomWorld};

pub(super) fn selected_sibling_mutation_carries_one_world_occurrence() {
    let world = CourtroomWorld::publish("blocked");
    let request = world::request_scope();
    let principal = principal(&world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let source = world
        .application
        .on_branch(world.application.current_world())
        .select()
        .unwrap();
    let source_branch = source.product().product_branch();
    let source_commit = source.product().selected_commit().clone();
    let sibling = fork_relational_product(
        &world,
        &source,
        "operation-journey-sibling",
        "operation-journey-sibling-data",
    );
    drop(source);

    let selected = world.application.on_branch(sibling).select().unwrap();
    let sibling_commit = selected.product().selected_commit().clone();
    let scope = selected
        .resolve_entity(
            IntentIdentityField::reference(),
            "intent-1".to_string(),
            &request,
            primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap();
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let retained = selected
        .admit_application_query(
            &query,
            &access,
            ApplicationQueryParameterSet::new(),
            controls(&request),
        )
        .unwrap();

    let warm = world
        .application
        .on_branch(sibling)
        .select()
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(warm) = warm else {
        panic!("the sibling condition must warm: {}", outcome_kind(&warm))
    };
    assert_eq!(warm.retained_suppressed_wake_count(), 1);
    drop(warm);
    let selected_sibling = world.application.on_branch(sibling).select().unwrap();
    let (replacement, _) = ReplacementPredicate::controlled(world.contacts.clone());
    let definition = selected_sibling
        .publish_conditional_definition(
            &world.clock,
            Arc::new(replacement),
            &runtime::RuntimeWorldCancellationSource::new().token(),
        )
        .unwrap();
    let primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(definition) =
        definition
    else {
        panic!("the sibling conditional definition must publish")
    };
    drop(definition);

    let mut publication = world
        .change_input_on_branch(sibling, "changed-on-sibling")
        .require_committed()
        .expect("the selected sibling mutation must commit");
    assert_eq!(publication.product_branch(), sibling);
    let change = publication
        .take_performed_relational_product_change()
        .expect("the application commit must retain World's performed publication");
    let sibling_after = world.application.on_branch(sibling).select().unwrap();
    assert_ne!(sibling_after.product().selected_commit(), &sibling_commit);
    assert_eq!(
        change.product_commit(),
        sibling_after.product().selected_commit()
    );
    let delivery = sibling_after
        .deliver_relational_change_to_conditional(&world.clock, 0, change)
        .expect("the performed sibling patch belongs to this selected product");
    let runtime::WorthQueryPerformedRelationalProductChangeDeliveryOutcome::Success(delivery) =
        delivery
    else {
        panic!("unexpected sibling delivery outcome: {delivery:?}")
    };
    assert_eq!(delivery.source_envelopes_loaded(), 1);
    assert_eq!(delivery.signal_seeds_emitted(), 1);
    drop(delivery);
    drop(sibling_after);

    let predecessor_interlude = selected_sibling
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(
        predecessor_interlude,
    ) = predecessor_interlude
    else {
        panic!("the retained predecessor product must remain independently observable")
    };
    assert_eq!(predecessor_interlude.committed_operation_count(), 0);
    drop(predecessor_interlude);

    world.clock_control.push(3, 11);
    let execution = world
        .application
        .on_branch(sibling)
        .select()
        .unwrap()
        .conditional_clock(&world.clock)
        .unwrap()
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(execution) =
        execution
    else {
        panic!(
            "conditional reentry must use sibling truth: {}",
            outcome_kind(&execution)
        )
    };
    assert_eq!(execution.committed_operation_count(), 1);
    assert_eq!(execution.authoritative_commit_count(), 1);
    drop(execution);

    let read = |branch: product::WorthQueryProductBranch| {
        let selected = world.application.on_branch(branch).select().unwrap();
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
        world
            .application
            .execute_application_query_one_shot(plan)
            .unwrap()
    };
    let sibling_fresh = read(sibling);
    assert_eq!(sibling_fresh.rows()[0].input, "changed-on-sibling");
    assert_eq!(
        product_identity(sibling_fresh.receipt()).product_branch(),
        sibling
    );
    drop(sibling_fresh);

    let source_fresh = read(source_branch);
    assert_eq!(source_fresh.rows()[0].input, "payload");
    assert_eq!(
        product_identity(source_fresh.receipt()).selected_commit(),
        &source_commit
    );
    drop(source_fresh);

    let retained = world
        .application
        .execute_application_query_one_shot(retained)
        .unwrap();
    assert_eq!(retained.rows()[0].input, "payload");
    assert_eq!(
        product_identity(retained.receipt()).selected_commit(),
        &sibling_commit
    );
    drop(retained);
    assert_eq!(
        world
            .application
            .application_query_basis_observer()
            .observe()
            .active(),
        0
    );
}

#[path = "public_product_journey/branch_facade.rs"]
mod branch_facade;
#[path = "public_product_journey/combined_publication.rs"]
mod combined_publication;
#[path = "public_product_journey/conditional_execution.rs"]
mod conditional_execution;
#[path = "public_product_journey/definition_lineage.rs"]
mod definition_lineage;
#[path = "public_product_journey/idempotency_affinity.rs"]
mod idempotency_affinity;
#[path = "public_product_journey/projection_and_outbox.rs"]
mod projection_and_outbox;
pub(super) use branch_facade::creates_and_selects_all_component_postures;
pub(super) use combined_publication::application_commits_relational_and_signal_in_one_world_publication;
pub(super) use conditional_execution::publishes_delivers_executes_and_cleans_up;
pub(super) use definition_lineage::independent_products_advance_and_retain_exact_definitions;
pub(super) use idempotency_affinity::shared_component_sibling_cannot_claim_another_product_commit;
pub(super) use idempotency_affinity::shared_component_sibling_revalidates_its_own_security;
