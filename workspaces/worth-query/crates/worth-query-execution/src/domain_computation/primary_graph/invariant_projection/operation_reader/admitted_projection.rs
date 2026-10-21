//! Original admitted projection, with an optional real source-query loan.
use super::admitted::SourceAccess;
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
            .project_with_work_limit(
                execution,
                admission.allowed_graph_contract().projection_work_budget(),
                product.relational_basis().clone(),
                |reader| {
                    super::admitted::project(
                        self,
                        reader,
                        admission,
                        &product,
                        projection,
                        SourceAccess {
                            source_query,
                            source_context_use,
                        },
                    )
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
