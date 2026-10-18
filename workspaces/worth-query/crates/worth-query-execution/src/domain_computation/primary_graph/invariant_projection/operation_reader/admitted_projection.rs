//! Original admitted projection, with an optional real source-query loan.
use super::*;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<Schema> {
    pub(in crate::domain_computation::primary_graph) fn project_admitted_operation_with_source_access<
        Operation,
        Input,
        Scope,
        Output,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
            &WorthQueryInvariantEntityIdentity<Schema, Scope>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        source_query: Option<
            &crate::domain_computation::primary_graph::application_query::SourceQueryReaderAccess<
                '_,
                Schema,
            >,
        >,
        source_context_use: Option<
            &std::cell::Cell<crate::domain_computation::primary_graph::handler::DecisionContextUse>,
        >,
    ) -> Result<
        WorthQueryCompletedOperationInvariantProjection<Schema, Operation, Output>,
        WorthQueryOperationProjectionDenial,
    > {
        admission.validate_projection_authority(self.runtime_authority, &self.binding_identity)?;
        let product = admission
            .graph_work()
            .mutation_lease()
            .expect("admitted operation retains its selected mutation lease")
            .product()
            .retained_clone();
        let allocation_control = crate::domain_computation::primary_graph::request_allocation_control::RequestAllocationControl::new(admission.publication_request(), allocation_policy);
        let completed = self
            .project_bounded(
                execution,
                admission.allowed_graph_contract().projection_work_budget(),
                product.relational_basis().clone(),
                |reader| {
                    reader.selected_product_occurrence =
                        Some(product.observation().lifecycle_incarnation());
                    reader.selected_product_generation =
                        Some(product.observation().reference_generation().get());
                    reader.selected_source_partition_identity =
                        admission.source_partition_identity();
                    let mut decision_facts = admission
                        .computation_prior()
                        .map_or_else(DecisionReads::default, DecisionReads::with_prior);
                    let mut operation_reader =
                        WorthQueryApplicationOperationInvariantProjectionReader {
                            reader,
                            source_query,
                            source_context_use,
                            admitted_graph_reads: Some(
                                admission.allowed_graph_contract().graph_reads(),
                            ),
                            runtime_authority: self.runtime_authority,
                            binding_identity: &self.binding_identity,
                            admission_identity: Some(admission.admission_identity()),
                            operation_scope: Some(admission.operation_scope_binding().clone()),
                            decision_facts: &mut decision_facts,
                            _operation: PhantomData,
                            request_local: PhantomData,
                        };
                    operation_reader
                        .reader
                        .realized_scope
                        .record(admission.scope_entity_id());
                    let scope = WorthQueryInvariantEntityIdentity {
                        entity_id: admission.scope_entity_id(),
                        kind: admission.scope_entity_kind(),
                        entity: Arc::from(admission.scope_entity_name()),
                        authority_identity: operation_reader.reader.authority_identity,
                        _marker: PhantomData,
                    };
                    let output = projection(&mut operation_reader, &scope);
                    (output, decision_facts)
                },
                Some(admission.publication_request()),
                allocation_control.policy(),
            )
            .map_err(|denial| {
                WorthQueryOperationProjectionDenial::from_invariant(denial, admission.operation())
            })?;
        Ok(WorthQueryCompletedOperationInvariantProjection {
            completed,
            admission_identity: admission.admission_identity(),
            product,
            _operation: PhantomData,
        })
    }
}
