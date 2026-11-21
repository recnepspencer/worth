//! Attempt-bound capability access minted only by capability admission.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkEvidence,
};

use worth_query_execution_attempt::facade::domain_computation::authorization::operation_progression::request_resolution::WorthQueryResolvedCapabilityRequest;
use super::ObservedCapabilityAdmission;
use crate::domain_computation::authorization::{
    WorthQueryRetainedCapabilityAuthorization, WorthQueryRetainedCapabilitySupport,
};

mod exact_observation;
mod inspection;
mod issuance;
pub mod operation_progression;
mod runtime_ext;

pub use exact_observation::{
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
};
pub use issuance::admit_observed_capability;

/// Move-only Query authority proving that one exact capability request was
/// admitted from current graph truth.
///
/// This proof opens no execution surface. A separately installed operation
/// must consume it before an application attempt can begin.
/// (Public compile-fail authority checks are tested on the
/// `worth_query_execution::facade::primary_graph` re-export.)
pub struct WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    authentication: Arc<dyn std::any::Any + Send + Sync>,
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    home_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    binding_identity: ApplicationSchemaBindingIdentity,
    capability: Arc<str>,
    capability_type: Arc<str>,
    operation: Arc<str>,
    principal_entity_id: worth_relational::facade::identity::EntityId,
    input: worth_query_declaration::facade::application_operation::ApplicationSharedInput<Input>,
    governed_input_identity: Option<[u8; 32]>,
    projection: ApplicationCapabilityRequestProjection<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Context,
    >,
    resolved: WorthQueryResolvedCapabilityRequest<
        Schema,
        <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
    >,
    authentication_valid_until: Instant,
    request_scope: WorthQueryRequestScope,
    canonical_work: WorthQueryCanonicalWorkEvidence,
    authorization: WorthQueryRetainedCapabilityAuthorization,
    operation_admission_identity:
        crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    graph_work_session:
        crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    graph_work_managed_run:
        crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity,
    graph_work_branch: worth_relational::facade::history::BranchId,
    graph_work_capability: Option<[u8; 32]>,
    graph_work_decision_facts: usize,
    product: crate::basis::WorthQueryProductBranch,
    security_basis: worth_relational::facade::branch::RelationalBranchBasisAdmissionIdentity,
    _marker: PhantomData<fn() -> (Capability, Operation)>,
}

impl<Schema, Capability, Operation, Input>
    worth_query_execution_attempt::facade::ports::AdmittedCapabilityAccessElevationPort
    for WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work_session
    }

    fn retain_observed_support(
        &mut self,
        supporting: WorthQueryRetainedCapabilitySupport,
    ) -> Result<(), ()> {
        self.retain_observed_support(supporting)
    }
}

pub trait AdmittedCapabilityAccessRuntimeExt<Schema, Capability, Operation, Input>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    fn refuse_stale_incarnation(
        &self,
        receiver: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema;

    fn read_product(
        &self,
        receiver: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Option<crate::basis::WorthQueryProductBranchLease>
    where
        Schema: ApplicationSchema;

    fn validate_workflow_read_authority(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema;

    fn validate_operation_authority(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        progression: crate::domain_computation::authorization::operation_progression::WorthQueryCapabilityOperationProgression,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    where
        Schema: ApplicationSchema,
        Operation: ApplicationOperationMarkerIdentity<Schema>;

    fn bind_operation_preconditions(
        &self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: impl std::borrow::Borrow<
            worth_query_declaration::facade::application_schema::TypedMutationPreconditions<
                Schema,
                Operation,
                <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
            >,
        >,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema;

    fn transition_operation(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        operation: &worth_query_installation::facade::WorthQueryInstalledApplicationOperation<
            Schema,
            Operation,
            Input,
        >,
        preconditions: crate::domain_computation::primary_graph::WorthQueryBoundMutationPreconditions,
    ) -> Result<
        crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema;

    fn with_exact_observation<Output>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        observe: impl FnOnce(
            &exact_observation::WorthQueryExactCapabilityObservation<'_, Schema>,
        ) -> Output,
    ) -> Result<
        Option<Output>,
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    >
    where
        Schema: ApplicationSchema;
}
