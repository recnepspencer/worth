//! Independent Relational and World observations for inbound courts.

use super::super::fixture::InboundWorld;

pub(crate) fn owner_commits(world: &InboundWorld) -> usize {
    world
        .application
        .primary_provider
        .graph
        .with_runtime(|runtime| runtime.history().immutable_commit_count())
}

pub(crate) fn product_commit(
    world: &InboundWorld,
) -> worth_runtime_world::facade::CompositeCommitIdentity {
    world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap()
        .selected_commit()
        .clone()
}

pub(crate) fn completion_records(world: &InboundWorld) -> usize {
    let product = world
        .application
        .product_runtime()
        .admit_product_branch(world.application.product_runtime().default_branch())
        .unwrap();
    let kind = world
        .application
        .primary_provider
        .graph
        .layout
        .provider_inbound_completion()
        .kind;
    world
        .application
        .primary_provider
        .graph
        .with_runtime_mut(|runtime| {
            let snapshot =
                super::super::super::super::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    product.relational_basis(),
                )
                .unwrap();
            let count = runtime
                .read_truth()
                .project_snapshot(&snapshot)
                .unwrap()
                .bounded_entities_of_kind(kind, 100)
                .unwrap()
                .records()
                .len();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            count
        })
}
