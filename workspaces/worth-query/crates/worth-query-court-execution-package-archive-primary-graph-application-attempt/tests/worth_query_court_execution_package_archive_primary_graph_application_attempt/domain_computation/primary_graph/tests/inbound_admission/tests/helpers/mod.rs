//! Independent Relational and World observations for inbound courts.

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::InboundWorld;

pub fn owner_commits(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &InboundWorld,
) -> usize {
    world
        .application
        .primary_provider
        .graph
        .with_runtime(request_execution, |runtime| {
            runtime.history().immutable_commit_count()
        })
}

pub fn product_commit(
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

pub fn completion_records(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &InboundWorld,
) -> usize {
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
        .with_runtime_mut(execution, |runtime| {
            let snapshot =
                worth_query_execution::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    product.relational_basis(),
                )
                .unwrap();
            let count = runtime
                .read_truth()
                .project_snapshot(&snapshot)
                .unwrap()
                .bounded_entities_of_kind(execution, kind, 100)
                .expect("fixture read request remains live")
                .unwrap()
                .records()
                .len();
            worth_query_execution::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            count
        })
}
