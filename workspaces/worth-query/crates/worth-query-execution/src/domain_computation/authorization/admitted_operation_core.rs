//! The non-generic admitted operation every typed admission brand carries.

use std::sync::Arc;
use std::time::Instant;

use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkPhases,
    WorthQueryCompiledApplicationOperationContracts,
};

use super::authorization_basis::ErasedAuthorizationBasis;
use super::original_input::ErasedOriginalInput;
use super::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::authorization::{
    WorthQueryOperationScopeBinding, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

/// One installed operation authorized for one exact current principal and
/// scope, with no application type. Move-only and owner-minted: it has no
/// `Clone`, `Default`, `Deserialize` or `From`, and only the admission
/// phases that mint a brand construct it.
pub(in crate::domain_computation) struct WorthQueryAdmittedOperation {
    pub(super) home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub(super) data: Box<WorthQueryAdmittedOperationData>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(in crate::domain_computation) struct WorthQueryAdmittedOperationData {
    pub(super) runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) operation: String,
    pub(super) operation_authority_identity: Arc<str>,
    pub(super) operation_authority_identity_bytes: [u8; 32],
    pub(super) operation_definition_identity: [u8; 32],
    pub(super) admission_identity: WorthQueryOperationAdmissionIdentity,
    pub(super) resource_binding_identity: Arc<str>,
    pub(super) operation_scope_binding: WorthQueryOperationScopeBinding,
    pub(super) canonical_work: WorthQueryCanonicalWorkPhases,
    pub(super) scope_entity_id: worth_relational::facade::identity::EntityId,
    pub(super) scope_entity_kind: worth_relational::facade::identity::KindId,
    pub(super) scope_entity_name: String,
    pub(super) authentication_valid_until: Instant,
    pub(super) request_scope:
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub(super) contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
    pub(super) mutation_preconditions: WorthQueryBoundMutationPreconditions,
    pub(super) authorization: Option<WorthQueryRetainedAuthorizationDecisionFacts>,
    pub(super) governed_input_identity: Option<[u8; 32]>,
    pub(super) original_input: Option<ErasedOriginalInput>,
    pub(super) original_scope:
        Option<Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>>,
    pub(super) original_inspection_disclosure: Option<Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>,
    pub(super) authorization_basis: ErasedAuthorizationBasis,
    pub(super) graph_work:
        crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    pub(super) source_partition_identity: Option<[u8; 32]>,
    pub(super) source_expectation_kind:
        Option<crate::domain_computation::primary_graph::WorthQuerySourceExpectationKind>,
    pub(super) source_facts:
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
    pub(super) query_context: Option<
        crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext,
    >,
    pub(super) required_output_demand:
        Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
}

// The core moves across threads on its own; only a brand's typed input slots
// can make an admission thread-bound.
const _: fn() = || {
    fn send<T: Send>() {}
    send::<WorthQueryAdmittedOperation>();
};

impl WorthQueryAdmittedOperation {
    pub(super) fn retain_original(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
        data: WorthQueryAdmittedOperationData,
    ) -> Result<
        Self,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    > {
        let memory = execution
            .reserve_retained_memory(std::mem::size_of::<WorthQueryAdmittedOperationData>() as u64)
            .map_err(|stop| {
                crate::domain_computation::authorization::denial::relational_read_denial(
                    stop.into(),
                    &data.operation,
                )
            })?;
        Ok(Self {
            home_owner,
            data: Box::new(data),
            _memory: memory,
        })
    }
    #[cfg(test)]
    pub(in crate::domain_computation) fn observe_inline_budget(
        census: &mut crate::inline_type_sizes::Census,
    ) {
        let _core=census.owner::<Self>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/admitted_operation_core.rs","WorthQueryAdmittedOperation",128);
        let _data=census.owner::<WorthQueryAdmittedOperationData>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/admitted_operation_core.rs","WorthQueryAdmittedOperationData",16*1024);
    }
}
