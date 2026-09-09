use worth_query_host::facade::primary_graph;

use super::super::product_query_support::reuse_exact_product;
use super::super::world::CourtroomWorld;

pub(crate) fn shared_component_sibling_cannot_claim_another_product_commit() {
    let world = CourtroomWorld::publish("ready");
    let source = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let source_identity = source.branch_identity().clone();
    drop(source);

    let original = world.change_input_on_product(&source_identity, "source-committed");
    let same_product = world.retry_input_change_on_product(&source_identity, "source-committed");
    let primary_graph::WorthQueryApplicationCommitOutcome::AlreadyCommitted(recovered) =
        same_product
    else {
        panic!("the same product occurrence must retain idempotent recovery: {same_product:?}")
    };
    assert!(recovered.is_same_authoritative_commit(&original));

    let committed_source = world
        .application
        .select_product_branch(&source_identity)
        .expect("the committed source remains selectable");
    let sibling_identity = reuse_exact_product(
        &world,
        committed_source.product(),
        "post-commit-idempotency-reuse-sibling",
    );
    assert_eq!(
        committed_source.product().relational_basis_descriptor(),
        world
            .application
            .select_product_branch(&sibling_identity)
            .unwrap()
            .product()
            .relational_basis_descriptor(),
        "the adversarial sibling must inherit the Relational row containing the source key",
    );
    drop(committed_source);

    let sibling = world.retry_input_change_on_product(&sibling_identity, "source-committed");
    let primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial) = sibling else {
        panic!("a post-commit sibling cannot claim the source product receipt: {sibling:?}")
    };
    assert_eq!(
        denial.stage(),
        primary_graph::WorthQueryApplicationCommitDenialStage::Idempotency,
    );
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::IdempotencyIntentDrift,
    );
}

pub(crate) fn shared_component_sibling_revalidates_its_own_security() {
    let world = CourtroomWorld::publish("ready");
    let source = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let sibling_identity = reuse_exact_product(&world, &source, "security-reuse-sibling");
    drop(source);

    world.revoke_principal_on_default_product();
    let sibling = world.retry_input_change_on_product(&sibling_identity, "sibling-attempt");
    let primary_graph::WorthQueryApplicationCommitOutcome::Denied(denial) = sibling else {
        panic!("the sibling must reach invariant product-basis validation: {sibling:?}")
    };
    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryApplicationCommitDenialKind::ProductBasisStale,
    );
    assert_eq!(
        denial.stage(),
        primary_graph::WorthQueryApplicationCommitDenialStage::InvariantExecution,
        "selected-product security must pass before component staleness is diagnosed",
    );
}
