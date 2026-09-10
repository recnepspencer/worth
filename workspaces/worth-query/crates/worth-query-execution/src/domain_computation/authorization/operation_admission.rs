//! Move-only installed operation admission authority.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[path = "operation_admission/capability_admission/mod.rs"]
mod capability_admission;
pub(in crate::domain_computation) use capability_admission::admit_capability_access;
pub use capability_admission::WorthQueryAdmittedApplicationCapabilityAccess;
pub(in crate::domain_computation::authorization) use capability_admission::{
    WorthQueryCapabilityContextKey, WorthQueryCurrentCapabilityObservation,
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
    WorthQueryResolvedCapabilityRequest,
};

use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkPhases,
    WorthQueryCompiledApplicationOperationContracts,
};

use crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts;
use crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions;

#[path = "operation_admission/access.rs"]
mod access;
#[path = "operation_admission/authorization_basis.rs"]
mod authorization_basis;
#[path = "operation_admission/capability_revocation.rs"]
mod capability_revocation;
#[path = "operation_admission/conventional.rs"]
mod conventional;
#[path = "operation_admission/delegation_activation.rs"]
mod delegation_activation;
#[path = "operation_admission/elevation_approval.rs"]
mod elevation_approval;
#[path = "operation_admission/elevation_close.rs"]
mod elevation_close;
#[path = "operation_admission/elevation_request.rs"]
mod elevation_request;
#[path = "operation_admission/graph_work_inspection.rs"]
mod graph_work_inspection;
#[path = "operation_admission/idempotency_binding.rs"]
mod idempotency_binding;
#[path = "operation_admission/mandatory_review.rs"]
mod mandatory_review;

pub(in crate::domain_computation::authorization) use authorization_basis::WorthQueryOperationAuthorizationBasis;

static NEXT_OPERATION_ADMISSION_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryOperationAdmissionIdentity(u64);

impl WorthQueryOperationAdmissionIdentity {
    pub(in crate::domain_computation::authorization) fn mint() -> Option<Self> {
        Self::mint_from(&NEXT_OPERATION_ADMISSION_IDENTITY)
    }

    pub(in crate::domain_computation::authorization) fn resource_binding_identity(
        self,
    ) -> Arc<str> {
        Arc::from(format!("worth-query-application-admission:{}", self.0))
    }

    fn mint_from(counter: &AtomicU64) -> Option<Self> {
        counter
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .ok()
            .map(Self)
    }
}

use crate::domain_computation::authorization::WorthQueryOperationScopeBinding;

/// Query-owned proof that one installed operation was authorized for one exact
/// current principal and typed scope.
///
/// The proof is move-only, has private fields, and is not deserializable.
/// Descriptive identities, token claims, or decision enums cannot mint it.
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// let _: WorthQueryAdmittedApplicationOperation<(), (), (), ()> =
///     serde_json::from_str("{}").unwrap();
/// ```
///
/// Operation-session topology remains owner-private and cannot be used as
/// caller authority:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationOperation;
///
/// fn cannot_extract_session<Schema, Operation, Input, Scope>(
///     admitted: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
/// ) {
///     let _session = admitted.graph_work_session_identity();
/// }
/// ```
pub struct WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope> {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    operation: String,
    operation_authority_identity: Arc<str>,
    operation_authority_identity_bytes: [u8; 32],
    admission_identity: WorthQueryOperationAdmissionIdentity,
    resource_binding_identity: Arc<str>,
    operation_scope_binding: WorthQueryOperationScopeBinding,
    canonical_work: WorthQueryCanonicalWorkPhases,
    scope_entity_id: worth_relational::facade::identity::EntityId,
    scope_entity_kind: worth_relational::facade::identity::KindId,
    scope_entity_name: String,
    authentication_valid_until: Instant,
    request_scope: worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    contracts: WorthQueryCompiledApplicationOperationContracts,
    mutation_preconditions: WorthQueryBoundMutationPreconditions,
    authorization: Option<WorthQueryRetainedAuthorizationDecisionFacts>,
    governed_input_identity: Option<[u8; 32]>,
    authorization_basis: WorthQueryOperationAuthorizationBasis<Input>,
    graph_work: crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    _marker: PhantomData<fn(Input) -> (Schema, Operation, Scope)>,
}

pub(in crate::domain_computation::authorization::operation_progression) fn transition_conventional_operation<
    Schema,
    Principal,
    PrincipalIdentity,
    Operation,
    Input,
    Scope,
>(
    bound: super::super::PreconditionBoundConventionalOperation<
        '_,
        Schema,
        Principal,
        PrincipalIdentity,
        Operation,
        Input,
        Scope,
    >,
) -> Result<
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    conventional::transition(bound)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::WorthQueryOperationAdmissionIdentity;

    #[test]
    fn operation_admission_identity_exhaustion_cannot_wrap() {
        let counter = AtomicU64::new(u64::MAX);
        assert!(WorthQueryOperationAdmissionIdentity::mint_from(&counter).is_none());
        assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), u64::MAX);
    }
}
