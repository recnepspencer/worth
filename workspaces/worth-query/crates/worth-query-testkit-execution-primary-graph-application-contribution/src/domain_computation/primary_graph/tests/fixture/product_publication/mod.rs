use worth_query_execution::court_doors_for_test::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationOutcome as PublicationOutcome;
use worth_relational::facade::transactions::WorkerIntentBatch;

use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;

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
    let request = worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope::new(
        None,
        worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource::new().token(),
        (*(*application.product_runtime().clock_for_test()).admission_clock_for_test()).clone(),
    );

    let (prepared, _before_lease) =
        prepare_relational_mutation_with_before(request_execution, application, batch, &request);
    let outcome = prepared.execute_for_test(*request_execution);
    let Ok(PublicationOutcome::Performed(performed)) = outcome else {
        panic!("fixture World publication must perform: {outcome:?}");
    };
    let published_basis = performed.commit().basis().relational_basis().clone();
    drop(performed.consume());
    let graph = (*application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    handle.with_runtime_mut_for_test(request_execution, |runtime| {
        let observation = published_basis.observation();
        let head = observation
            .commit_receipt()
            .expect("the fixture publication has a canonical commit");
        runtime
            .index_authority()
            .verify_index_basis_for_commit(request_execution, head)
            .expect("native preparation carried this publication's index selections");
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
        .select_product_branch_for_test(
            execution,
            application.product_runtime().default_branch_for_test(),
        )
        .expect("the fixture product branch remains admitted");
    let (_, product, application_basis) = selected.into_parts_for_test();
    let graph = (*application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let candidate = handle.with_runtime_mut_for_test(execution, |runtime| {
        let mut transaction = runtime
            .begin_branch_transaction(
                product.relational_basis_for_test(),
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
        .publication_binding_for_test()
        .prepare_relational_candidate_for_test(candidate, request, false)
        .expect("fixture World publication prepares");
    (prepared, application_basis)
}
