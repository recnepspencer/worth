//! The one operation-scoped reader envelope for owned and borrowed projections.
use super::*;
pub mod root;
pub use root::AdmittedReadRoot;

/// The optional real source-query loan an admitted projection may read through.
pub struct SourceAccess<'access> {
    pub source_query: Option<&'access dyn SourceQueryReaderPort>,
    pub source_context_use: Option<
        &'access std::cell::Cell<
            crate::domain_computation::primary_graph::handler::DecisionContextUse,
        >,
    >,
}

impl SourceAccess<'_> {
    pub const fn none() -> Self {
        Self {
            source_query: None,
            source_context_use: None,
        }
    }
}

pub fn project<Schema, Operation, Scope, Output>(
    authority: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    reader: &mut WorthQueryApplicationInvariantProjectionReader<'_, Schema>,
    admission: &dyn AdmittedOperationPort,
    product: &crate::basis::WorthQueryProductBranchLease,
    projection: impl FnOnce(
        &mut WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>,
        &WorthQueryInvariantEntityIdentity<Schema, Scope>,
    ) -> Output,
    source_access: SourceAccess<'_>,
) -> (Output, DecisionReads) {
    reader.selected_product_occurrence = Some(product.observation().lifecycle_incarnation());
    reader.selected_product_generation = Some(product.observation().reference_generation().get());
    reader.selected_source_partition_identity = admission.source_partition_identity();
    let mut decision_facts = admission
        .computation_prior()
        .map_or_else(DecisionReads::default, DecisionReads::with_prior);
    let mut operation_reader = WorthQueryApplicationOperationInvariantProjectionReader {
        reader,
        source_query: source_access.source_query,
        source_context_use: source_access.source_context_use,
        admitted_contracts: Some(admission.retain_graph_contract()),
        runtime_authority: authority.runtime_authority,
        binding_identity: &authority.binding_identity,
        admission_identity: Some(admission.admission_identity()),
        selected_product: Some(product),
        admitted_root: Some(AdmittedReadRoot::from_admission(admission)),
        admitted_principal: Some(admission.principal_entity_id()),
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
        phase: operation_reader
            .reader
            .native
            .candidate_identity()
            .map(|id| id.qualification()),
        _marker: PhantomData,
    };
    let output = projection(&mut operation_reader, &scope);
    (output, decision_facts)
}
