//! The one operation-scoped reader envelope for owned and borrowed projections.
use super::*;

pub(super) fn project<Schema, Operation, Input, Scope, Output>(
    authority: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    product: &crate::basis::WorthQueryProductBranchLease,
    projection: impl FnOnce(
        &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        &WorthQueryInvariantEntityIdentity<Schema, Scope>,
    ) -> Output,
) -> (Output, DecisionReads) {
    reader.selected_product_occurrence = Some(product.observation().lifecycle_incarnation());
    reader.selected_product_generation = Some(product.observation().reference_generation().get());
    reader.selected_source_partition_identity = admission.source_partition_identity();
    let mut decision_facts = admission
        .computation_prior()
        .map_or_else(DecisionReads::default, DecisionReads::with_prior);
    let mut operation_reader = WorthQueryApplicationOperationInvariantProjectionReader {
        reader,
        admitted_contracts: Some(admission.allowed_graph_contract()),
        runtime_authority: authority.runtime_authority,
        binding_identity: &authority.binding_identity,
        admission_identity: Some(admission.admission_identity()),
        admitted_principal: Some(admission.graph_work().principal()),
        query_context: admission.query_context(),
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
}
