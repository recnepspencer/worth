//! The non-generic admitted operation every typed admission brand carries.

use std::ops::{Deref, DerefMut};
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

trait AdmittedOperationDataBacking: Send {
    fn as_data(&self) -> &WorthQueryAdmittedOperationData;
    fn as_data_mut(&mut self) -> &mut WorthQueryAdmittedOperationData;
}

impl AdmittedOperationDataBacking for WorthQueryAdmittedOperationData {
    fn as_data(&self) -> &WorthQueryAdmittedOperationData {
        self
    }

    fn as_data_mut(&mut self) -> &mut WorthQueryAdmittedOperationData {
        self
    }
}

pub(super) struct ErasedAdmittedOperationData(Box<dyn AdmittedOperationDataBacking>);

impl ErasedAdmittedOperationData {
    fn new(data: WorthQueryAdmittedOperationData) -> Self {
        Self(Box::new(data))
    }
}

impl Deref for ErasedAdmittedOperationData {
    type Target = WorthQueryAdmittedOperationData;

    fn deref(&self) -> &Self::Target {
        self.0.as_data()
    }
}

impl DerefMut for ErasedAdmittedOperationData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_data_mut()
    }
}

/// One installed operation authorized for one exact current principal and
/// scope, with no application type. Move-only and owner-minted: it has no
/// `Clone`, `Default`, `Deserialize` or `From`, and only the admission
/// phases that mint a brand construct it.
pub struct WorthQueryAdmittedOperation {
    pub(super) home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub(super) data: ErasedAdmittedOperationData,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub struct WorthQueryAdmittedOperationData {
    pub runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    pub binding_identity: ApplicationSchemaBindingIdentity,
    pub operation: String,
    pub operation_authority_identity: Arc<str>,
    pub operation_authority_identity_bytes: [u8; 32],
    pub operation_definition_identity: [u8; 32],
    pub admission_identity: WorthQueryOperationAdmissionIdentity,
    pub resource_binding_identity: Arc<str>,
    pub operation_scope_binding: WorthQueryOperationScopeBinding,
    pub canonical_work: WorthQueryCanonicalWorkPhases,
    pub scope_entity_id: worth_relational::facade::identity::EntityId,
    pub scope_entity_kind: worth_relational::facade::identity::KindId,
    pub scope_entity_name: String,
    pub authentication_valid_until: Instant,
    pub request_scope:
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    pub contracts: Arc<WorthQueryCompiledApplicationOperationContracts>,
    pub mutation_preconditions: WorthQueryBoundMutationPreconditions,
    pub authorization: Option<WorthQueryRetainedAuthorizationDecisionFacts>,
    pub governed_input_identity: Option<[u8; 32]>,
    pub original_input: Option<ErasedOriginalInput>,
    pub original_scope:
        Option<Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>>,
    pub original_inspection_disclosure: Option<Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>,
    pub authorization_basis: ErasedAuthorizationBasis,
    pub graph_work:
        crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    pub source_partition_identity: Option<[u8; 32]>,
    pub source_expectation_kind:
        Option<crate::domain_computation::primary_graph::WorthQuerySourceExpectationKind>,
    pub source_facts:
        Vec<crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact>,
    pub query_context: Option<
        crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext,
    >,
    pub required_output_demand:
        Option<crate::domain_computation::primary_graph::RequiredOutputDemandContext>,
}

// The core moves across threads on its own; only a brand's typed input slots
// can make an admission thread-bound.
const _: fn() = || {
    fn send<T: Send>() {}
    send::<WorthQueryAdmittedOperation>();
};

impl WorthQueryAdmittedOperation {
    pub fn retain_original(
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
            data: ErasedAdmittedOperationData::new(data),
            _memory: memory,
        })
    }
    #[cfg(test)]
    pub fn observe_inline_budget(census: &mut crate::inline_type_sizes::Census) {
        let _core=census.owner::<Self>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/admitted_operation_core.rs","WorthQueryAdmittedOperation",128);
        let _data=census.owner::<WorthQueryAdmittedOperationData>("workspaces/worth-query/crates/worth-query-execution/src/domain_computation/authorization/admitted_operation_core.rs","WorthQueryAdmittedOperationData",16*1024);
    }
}
