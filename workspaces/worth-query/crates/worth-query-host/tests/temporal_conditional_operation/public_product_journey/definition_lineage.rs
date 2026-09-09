use std::sync::{Arc, Barrier};

use worth_query_host::facade::{primary_graph, runtime};

use super::super::adapters::ReplacementPredicate;
use super::super::courtroom_lifecycle::assert_conditional_resources_empty;
use super::super::courtroom_support::outcome_kind;
use super::super::product_query_support::fork_independent_product;
use super::super::world::CourtroomWorld;

pub(crate) fn independent_products_advance_and_retain_exact_definitions() {
    let mut world = CourtroomWorld::publish("blocked");
    let source = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let product_a = fork_independent_product(
        &world,
        &source,
        "definition-lineage-a",
        "definition-lineage-a-data",
        "definition-lineage-a-signal",
    );
    let product_b = fork_independent_product(
        &world,
        &source,
        "definition-lineage-b",
        "definition-lineage-b-data",
        "definition-lineage-b-signal",
    );
    drop(source);

    let pinned_a_d0 = world.application.select_product_branch(&product_a).unwrap();
    let pinned_b_d0 = world.application.select_product_branch(&product_b).unwrap();
    assert_suppressed(
        &world,
        &product_a,
        "A D0 must resolve before either successor",
    );
    assert_suppressed(
        &world,
        &product_b,
        "B D0 must resolve before either successor",
    );

    let start = Arc::new(Barrier::new(2));
    let (published_a, published_b) = std::thread::scope(|scope| {
        let world_ref: &CourtroomWorld = &world;
        let start_a = Arc::clone(&start);
        let product_a_for_publish = product_a.clone();
        let publish_a = scope.spawn(move || {
            let (replacement, _) = ReplacementPredicate::controlled(world_ref.contacts.clone());
            let selected = world_ref
                .application
                .select_product_branch(&product_a_for_publish)
                .unwrap();
            start_a.wait();
            selected
                .publish_conditional_definition(
                    &world_ref.clock,
                    Arc::new(replacement),
                    &runtime::RuntimeWorldCancellationSource::new().token(),
                )
                .unwrap()
        });
        let start_b = Arc::clone(&start);
        let product_b_for_publish = product_b.clone();
        let publish_b = scope.spawn(move || {
            let (replacement, _) = ReplacementPredicate::controlled(world_ref.contacts.clone());
            let selected = world_ref
                .application
                .select_product_branch(&product_b_for_publish)
                .unwrap();
            start_b.wait();
            selected
                .publish_conditional_definition(
                    &world_ref.clock,
                    Arc::new(replacement),
                    &runtime::RuntimeWorldCancellationSource::new().token(),
                )
                .unwrap()
        });
        (publish_a.join().unwrap(), publish_b.join().unwrap())
    });
    let primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(published_a) =
        published_a
    else {
        panic!("A must publish its independent D1")
    };
    assert_eq!(published_a.product_branch_identity(), &product_a);
    assert_eq!(published_a.definition_generation(), 2);

    let primary_graph::WorthQueryConditionalDefinitionPublicationOutcome::Performed(published_b) =
        published_b
    else {
        panic!("B must publish D1 after A without inheriting A's lowering")
    };
    assert_eq!(published_b.product_branch_identity(), &product_b);
    assert_eq!(published_b.definition_generation(), 2);

    assert_suppressed(&world, &product_a, "fresh A must resolve exact D1");
    assert_suppressed(&world, &product_b, "fresh B must resolve exact D1");
    assert_pinned_suppressed(pinned_a_d0, &world, "pinned A must retain exact D0");
    assert_pinned_suppressed(pinned_b_d0, &world, "pinned B must retain exact D0");

    drop(published_a);
    drop(published_b);
    world.application.close_conditional_runtime().unwrap();
    assert_conditional_resources_empty(world.application.inspect_conditional_runtime());
}

fn assert_suppressed(
    world: &CourtroomWorld,
    product: &runtime::ProductBranchIdentity,
    message: &str,
) {
    let selected = world.application.select_product_branch(product).unwrap();
    assert_pinned_suppressed(selected, world, message);
}

fn assert_pinned_suppressed(
    selected: primary_graph::WorthQuerySelectedProductOperation<
        '_,
        super::super::schema::TemporalHostSchema,
    >,
    world: &CourtroomWorld,
    message: &str,
) {
    let outcome = selected
        .conditional_clock(&world.clock)
        .unwrap_or_else(|denial| panic!("{message}: {denial:?}"))
        .observe();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) = outcome
    else {
        panic!("{message}: {}", outcome_kind(&outcome))
    };
    assert_eq!(receipt.retained_suppressed_wake_count(), 1, "{message}");
}
