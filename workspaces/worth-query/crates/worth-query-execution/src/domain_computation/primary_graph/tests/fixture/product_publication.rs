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
    let (prepared, before_lease) =
        prepare_relational_mutation_with_before(application, batch, &request);
    let outcome = prepared.execute();
    let RuntimeWorldPublicationOutcome::Performed(performed) = outcome else {
        panic!("fixture World publication must perform: {outcome:?}");
    };
    let published_basis = performed.commit().basis().relational_basis().clone();
    drop(performed.consume());
    let graph = application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    handle.with_runtime_mut(|runtime| {
        let observation = published_basis.observation();
        let head = observation
            .commit_receipt()
            .expect("the fixture publication has a canonical commit");
        let refreshed = crate::domain_computation::primary_graph::index_maintenance_budget::refresh_with_cold_fallback(
            runtime,
            worth_relational::facade::indexes::DerivedIndexBuildRequest {
                source_commit_id: head.commit_id,
                branch_id: head.branch_id.clone(),
                index_ids: handle.primary_index_ids.to_vec(),
            },
            &published_basis,
            Some(before_lease.snapshot_handle()),
        )
        .expect("fixture publication incrementally refreshes its exact indexes");
        assert_eq!(refreshed.generations.len(), handle.primary_index_ids.len());
        assert_eq!(refreshed.work.cold_record_slots, 0);
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
    prepare_relational_mutation_with_before(application, batch, request).0
}

fn prepare_relational_mutation_with_before<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> (
    crate::domain_computation::execution_runtime::product_world::WorthQueryPreparedProductPublication,
    crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease,
){
    let selected = application
        .select_product_branch(application.product_runtime().default_branch())
        .expect("the fixture product branch remains admitted");
    let (_, product, application_basis) = selected.into_parts();
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

    let prepared = product
        .publication_binding()
        .prepare_relational_candidate(candidate, request, false)
        .expect("fixture World publication prepares");
    (prepared, application_basis)
}
