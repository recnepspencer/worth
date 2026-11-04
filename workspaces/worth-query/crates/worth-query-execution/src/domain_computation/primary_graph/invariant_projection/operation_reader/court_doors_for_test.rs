//! Sealed test doors over this module's private members.
use super::*;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
{
    #[doc(hidden)]
    pub fn admission_identity_for_test(&self) -> &Option<WorthQueryOperationAdmissionIdentity> {
        &self.admission_identity
    }
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
{
    #[doc(hidden)]
    pub fn admitted_contracts_for_test(
        &self,
    ) -> &Option<Arc<WorthQueryCompiledApplicationOperationContracts>> {
        &self.admitted_contracts
    }
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
{
    #[doc(hidden)]
    pub fn decision_facts_for_test(&self) -> &&'reader mut DecisionReads {
        &self.decision_facts
    }
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
{
    #[doc(hidden)]
    pub fn reader_for_test(
        &self,
    ) -> &&'reader mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema> {
        &self.reader
    }
}

impl<Schema> WorthQueryApplicationInvariantProjectionAuthority<Schema>
where
    Schema: ApplicationSchema,
{
    #[doc(hidden)]
    pub fn project_operation_on_product_for_test<Operation, Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::basis::WorthQueryProductBranchLease,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        ) -> Output,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryInspectedOperationInvariantProjection<Operation, Output>,
        WorthQueryInvariantProjectionDenial,
    > {
        Self::project_operation_on_product(self, execution, product, projection, allocation_policy)
    }
}

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
{
    #[doc(hidden)]
    pub fn decision_facts_mut_for_test(&mut self) -> &mut &'reader mut DecisionReads {
        &mut self.decision_facts
    }
}
