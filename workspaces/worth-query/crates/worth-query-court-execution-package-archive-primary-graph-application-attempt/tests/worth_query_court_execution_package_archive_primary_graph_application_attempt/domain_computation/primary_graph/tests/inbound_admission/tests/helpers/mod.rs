//! Independent Relational and World observations for inbound courts.

use crate::domain_computation::primary_graph::tests::inbound_admission::fixture::InboundWorld;

pub fn owner_commits(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &InboundWorld,
) -> usize {
    (*(*world
        .application
        .primary_provider_for_test())
        .graph_for_test())
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
        .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
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
        .admit_product_branch_for_test(world.application.product_runtime().default_branch_for_test())
        .unwrap();
    let kind = (*(*(*world
        .application
        .primary_provider_for_test())
        .graph_for_test())
        .layout_for_test())
        .provider_inbound_completion()
        .kind;
    (*(*world
        .application
        .primary_provider_for_test())
        .graph_for_test())
        .with_runtime_mut_for_test(execution, |runtime| {
            let snapshot =
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    product.relational_basis_for_test(),
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
            worth_query_execution::court_doors_for_test::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            count
        })
}
