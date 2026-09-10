use worth_relational::facade::transactions::WorkerIntentBatch;
use worth_runtime_world::facade::RuntimeWorldPublicationOutcome;

use super::{live_scope, AuthorizationWorld};

/// Publishes one fixture mutation through the real World owner boundary.
pub(in crate::domain_computation::primary_graph) fn publish_relational_mutation(
    world: &AuthorizationWorld,
    batch: WorkerIntentBatch,
) {
    publish_relational_mutation_on_application(&world.application, batch)
}

pub(in crate::domain_computation::primary_graph) fn publish_relational_mutation_on_application<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
) {
    let request = live_scope();
    let prepared = prepare_relational_mutation_on_application(application, batch, &request);
    let outcome = prepared.execute();
    let RuntimeWorldPublicationOutcome::Performed(performed) = outcome else {
        panic!("fixture World publication must perform: {outcome:?}");
    };
    let published_basis = performed.commit().basis().relational_basis().clone();
    drop(performed.consume());
    let graph = application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    handle.with_runtime_mut(|runtime| {
        handle
            .ensure_primary_indexes_for_basis(runtime, &published_basis)
            .expect("fixture publication refreshes indexes at the published basis");
    });
}

pub(in crate::domain_computation::primary_graph) fn prepare_relational_mutation_on_application<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication
{
    let selected = application
        .select_product_branch(application.product_runtime().default_branch())
        .expect("the fixture product branch remains admitted");
    let (_, product, application_basis) = selected.into_parts();
    drop(application_basis);

    let graph = application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let candidate = handle.with_runtime_mut(|runtime| {
        let mut transaction = runtime
            .begin_branch_transaction(
                product.relational_basis(),
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("selected product basis admits the fixture transaction");
        transaction
            .push_batch(batch)
            .expect("fixture mutation stays within configured resource budgets");
        runtime
            .prepare_branch_transaction(transaction)
            .expect("fixture mutation prepares a real Relational candidate")
    });

    product
        .publication_binding()
        .prepare_relational_candidate(candidate, request, false)
        .expect("fixture World publication prepares")
}
