//! A read-only operation envelope never receives an admission or a mutation root.
use super::*;
impl<S: ApplicationSchema> WorthQueryApplicationInvariantProjectionAuthority<S> {
    pub(super) fn inspect_reader<O, T>(
        &self,
        reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, S>,
        contracts: Option<Arc<WorthQueryCompiledApplicationOperationContracts>>,
        product: Option<&crate::basis::WorthQueryProductBranchLease>,
        projection: impl FnOnce(
            &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>,
        ) -> T,
    ) -> (T, DecisionReads) {
        let mut decision_facts = DecisionReads::default();
        let mut operation_reader = WorthQueryApplicationOperationInvariantProjectionReader {
            reader,
            source_query: None,
            source_context_use: None,
            admitted_contracts: contracts,
            runtime_authority: self.runtime_authority,
            binding_identity: &self.binding_identity,
            admission_identity: None,
            selected_product: product,
            admitted_root: None,
            admitted_principal: None,
            query_context: None,
            operation_scope: None,
            decision_facts: &mut decision_facts,
            _operation: PhantomData,
            request_local: PhantomData,
        };
        let output = projection(&mut operation_reader);
        (output, decision_facts)
    }
}
