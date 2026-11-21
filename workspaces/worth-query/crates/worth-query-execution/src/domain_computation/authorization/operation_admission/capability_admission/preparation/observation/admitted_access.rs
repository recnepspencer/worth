//! Attempt-bound capability access minted only by capability admission.

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityValidityTimeline,
};
use worth_query_declaration::facade::application_operation::ApplicationCanonicalWork;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity, WorthQueryCanonicalWorkEvidence,
};
use worth_relational::facade::authorization::RelationalAuthorizationObservationCounters;

use super::request_resolution::WorthQueryResolvedCapabilityRequest;
use super::ObservedCapabilityAdmission;
use crate::domain_computation::authorization::{
    WorthQueryRetainedCapabilityAuthorization, WorthQueryRetainedCapabilitySupport,
};
mod exact_observation;
mod issuance;
pub(in super::super) use issuance::admit_observed_capability;
mod inspection;
pub(crate) mod operation_progression;

pub(in crate::domain_computation::authorization) use exact_observation::{
    WorthQueryDelegationResolvedRequest, WorthQueryExactCapabilityObservationContext,
};

/// Move-only Query authority proving that one exact capability request was
/// admitted from current graph truth.
///
/// This proof opens no execution surface. A separately installed operation
/// must consume it before an application attempt can begin.
///
/// Authentication is not admitted capability access:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAuthenticatedPrincipal,
/// };
///
/// fn cannot_substitute_authentication<Schema, Capability, Operation, Input, Principal, Identity>(
///     principal: WorthQueryAuthenticatedPrincipal<Schema, Principal, Identity>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = principal;
/// }
/// ```
///
/// A descriptive capability reference is not admitted capability access:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_substitute_description<Schema, Capability, Operation, Input>(
///     capability: ApplicationCapabilityRef<Schema, Capability>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = capability;
/// }
/// ```
///
/// The access proof is move-only:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_clone_access<Schema, Capability, Operation, Input>(
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _copied = access.clone();
/// }
/// ```
///
/// Exact delegation lineage is private decision evidence, not a consumer
/// inspection or reconstruction surface:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_extract_lineage<Schema, Capability, Operation, Input>(
///     access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _lineage = access.delegation_lineage();
/// }
/// ```
///
/// A portable provenance marker cannot be promoted into access authority:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_capability::{
///     ApplicationCapabilityProvenanceRef, ApplicationCapabilityRequest,
/// };
/// use worth_query_execution::facade::primary_graph::WorthQueryAdmittedApplicationCapabilityAccess;
///
/// fn cannot_promote_provenance<Schema, Capability, Operation, Input, Provenance>(
///     provenance: ApplicationCapabilityProvenanceRef<Schema, Provenance>,
/// ) where
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     let _: WorthQueryAdmittedApplicationCapabilityAccess<
///         Schema, Capability, Operation, Input,
///     > = provenance;
/// }
/// ```
///
/// The operation marker is exact and cannot be repurposed:
///
/// ```compile_fail
/// use worth_query_declaration::facade::{
///     application_capability::ApplicationCapabilityRequest,
///     application_schema::TypedMutationPreconditions,
/// };
/// use worth_query_execution::facade::primary_graph::{
///     WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryPrimaryGraphApplicationRuntime,
/// };
/// use worth_query_installation::facade::{
///     ApplicationSchema, WorthQueryInstalledApplicationOperation,
/// };
///
/// fn cannot_change_operation<Schema, Capability, Granted, Other, Input>(
///     runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
///     access: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Granted, Input>,
///     other: &WorthQueryInstalledApplicationOperation<Schema, Other, Input>,
/// ) where
///     Schema: ApplicationSchema,
///     Input: ApplicationCapabilityRequest<Schema, Capability>,
/// {
///     runtime.authorize_capability_operation(
///         access,
///         other,
///         TypedMutationPreconditions::new(),
///     );
/// }
/// ```
pub use worth_query_execution_attempt::facade::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;


pub(in crate::domain_computation) trait AdmittedCapabilityAccessRuntimeExt<
    Schema,
    Capability,
    Operation,
    Input,
> where
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

mod runtime_ext;
