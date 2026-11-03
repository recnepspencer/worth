use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use worth_relational::facade::transactions::WorkerIntentBatch;

use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

/// Publishes one fixture mutation through the real World owner boundary.
pub fn publish_relational_mutation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    world: &AuthorizationWorld,
    batch: WorkerIntentBatch,
) {
    publish_relational_mutation_on_application(execution, &world.application, batch)
}

pub fn publish_relational_mutation_on_application<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
) {
    let request = live_scope();

    let (prepared, before_lease) =
        prepare_relational_mutation_with_before(request_execution, application, batch, &request);
    let outcome = prepared.execute_for_test(*request_execution);
    let Ok(PublicationOutcome::Performed(performed)) = outcome else {
        panic!("fixture World publication must perform: {outcome:?}");
    };
    let published_basis = performed.commit().basis().relational_basis().clone();
    drop(performed.consume());
    let graph = (*application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle();
    handle.with_runtime_mut(request_execution, |runtime| {
        let observation = published_basis.observation();
        let head = observation
            .commit_receipt()
            .expect("the fixture publication has a canonical commit");
        let refreshed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::index_maintenance_budget::refresh_with_cold_fallback(request_execution,
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

pub fn prepare_relational_mutation_on_application<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::request_control::WorthQueryPreparedProductPublication
{
    prepare_relational_mutation_with_before(execution, application, batch, request).0
}

pub fn prepare_relational_mutation_with_before<
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
execution: &worth_execution::ExecutionRequest<'_, '_>,
    application: &worth_query_execution::facade::domain_computation::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    batch: WorkerIntentBatch,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> (
    worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::request_control::WorthQueryPreparedProductPublication,
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_query::resource_lifecycle::basis_registry::lease::WorthQueryApplicationBasisLease,
){
    let selected = application
        .select_product_branch_for_test(execution, application.product_runtime().default_branch_for_test())
        .expect("the fixture product branch remains admitted");
    let (_, product, application_basis) = selected.into_parts();
    let graph = (*application.runtime_for_test()).primary_graph().unwrap();
    let handle = graph.integration_handle();
    let candidate = handle.with_runtime_mut(execution, |runtime| {
        let mut transaction = runtime
            .begin_branch_transaction(
                product.relational_basis(),
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .expect("selected product basis admits the fixture transaction");
        transaction
            .push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("fixture mutation stays within configured resource budgets");
        runtime
            .prepare_branch_transaction(execution, transaction)
            .expect("fixture mutation prepares a real Relational candidate")
    });

    let prepared = product
        .publication_binding()
        .prepare_relational_candidate(candidate, request, false)
        .expect("fixture World publication prepares");
    (prepared, application_basis)
}
