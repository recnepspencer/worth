//! Crate-boundary port exposing owner-private `WorthQueryAdmittedApplicationOperation`
//! operations strictly to upper execution split crates (`worth-query-execution-query`,
//! `worth-query-execution`). Never re-exported on `worth-query-execution::facade` or Host.

use std::sync::Arc;

type AdmittedAt = Result<
    worth_foundational::facade::CanonicalTimestamp,
    crate::domain_computation::runtime_time::WorthQueryRuntimeTimeDenial,
>;

use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::application_operation::{
    ApplicationCanonicalWork, ApplicationMutationBindingDescriptor, ApplicationSharedInput,
};
use worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, InstalledOperationCatalog, WorthQueryCanonicalWorkEvidence,
};
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::BridgeAuthorizationRuntime;
use worth_runtime_world::facade::RuntimeWorldOwnerIdentity;

use crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure;
use crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness;
use crate::domain_computation::authorization::operation_admission::{
    BrandTag, WorthQueryAdmittedOperation,
};
use crate::domain_computation::authorization::{
    WorthQueryAdmissionLapse, WorthQueryCapabilityRevocationBinding,
    WorthQueryDelegationActivationBinding, WorthQueryElevationApprovalBinding,
    WorthQueryElevationCloseBinding, WorthQueryElevationRequestBinding,
    WorthQueryMandatoryReviewBinding, WorthQueryOperationAuthorizationDenial,
    WorthQueryProviderCommitAuthorization, WorthQueryRetainedAuthorizationDecisionFacts,
};
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext;
use crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure;
use crate::domain_computation::primary_graph::{
    ComputationPrior, DecisionContextUse, MutationHandlerExecutionDenial, OriginalProgramMeaning,
    RequiredOutputDemandContext, WorthQueryApplicationObservedFact,
    WorthQueryObservedScopeSelector, WorthQuerySourceExpectationKind,
};
use crate::domain_computation::provider_session::{
    WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
    WorthQueryManagedGraphWorkSession,
};

#[path = "admitted_operation_port_impl.rs"]
mod impls;

pub trait AdmittedOperationExecutionPort<Schema, Operation, Input, Scope>: Sized {
    fn mint(
        core: WorthQueryAdmittedOperation,
        tag: BrandTag,
        capability_input: Option<ApplicationSharedInput<Input>>,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>;
    fn into_read_authorization(self) -> Option<WorthQueryRetainedAuthorizationDecisionFacts>;
    fn reentry_parts(&self) -> (&WorthQueryAdmittedOperation, &BrandTag);
    fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool;
    fn validate_projection_authority(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;
    fn validate_installed_mutation_binding(
        &self,
        descriptor: Option<&ApplicationMutationBindingDescriptor>,
    ) -> Result<(), MutationHandlerExecutionDenial>;
    fn original_input(&self) -> Option<(&ApplicationSharedInput<Input>, [u8; 32])>;
    fn bind_delegation_activation(
        self,
        binding: WorthQueryDelegationActivationBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>;
    fn bind_capability_revocation(
        self,
        binding: WorthQueryCapabilityRevocationBinding,
        work: WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>;
    fn bind_elevation_request(
        self,
        binding: WorthQueryElevationRequestBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial>;
    fn bind_elevation_approval(
        self,
        execution: &ExecutionRequest<'_, '_>,
        binding: WorthQueryElevationApprovalBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationApprovalBinding,
        ),
    >;
    fn bind_elevation_close(
        self,
        execution: &ExecutionRequest<'_, '_>,
        binding: WorthQueryElevationCloseBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryElevationCloseBinding,
        ),
    >;
    fn bind_mandatory_review(
        self,
        execution: &ExecutionRequest<'_, '_>,
        binding: WorthQueryMandatoryReviewBinding,
    ) -> Result<
        Self,
        (
            WorthQueryOperationAuthorizationDenial,
            WorthQueryMandatoryReviewBinding,
        ),
    >;
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn scope_entity_kind(&self) -> worth_relational::facade::identity::KindId;
    fn has_elevation_lifecycle_binding(&self) -> bool;
    fn installed_capability_identity(&self) -> Option<[u8; 32]>;
    fn authorization_mut(&mut self) -> Option<&mut WorthQueryRetainedAuthorizationDecisionFacts>;
    fn authorization(&self) -> Option<&WorthQueryRetainedAuthorizationDecisionFacts>;
    fn take_authorization_dependencies(
        &mut self,
        bridge: &BridgeAuthorizationRuntime,
    ) -> Result<WorthQueryProviderCommitAuthorization, WorthQueryOperationAuthorizationDenial>;
    fn retain_installed_operation_fingerprint(&self) -> Arc<str>;
    fn retain_resource_binding_identity(&self) -> Arc<str>;
    fn current_authority_lapse(&self) -> Option<WorthQueryAdmissionLapse>;
    fn retain_admitted_request(&self) -> AdmittedRequestCurrentness;
    fn publication_request(&self) -> &WorthQueryRequestScope;
    fn bind_source_facts(
        &mut self,
        facts: Vec<WorthQueryApplicationObservedFact>,
        context: Option<ObservedQueryExpressionContext>,
    );
    fn bind_source_partition(&mut self, identity: [u8; 32]);
    fn bind_source_expectation_kind(&mut self, kind: WorthQuerySourceExpectationKind);
    fn source_partition_identity(&self) -> Option<[u8; 32]>;
    fn source_facts(&self) -> &[WorthQueryApplicationObservedFact];
    fn signed_creation_principal(&self) -> Option<&Arc<WorthQueryExternalPrincipalIdentity>>;
    fn graph_work_session_identity(&self) -> WorthQueryGraphWorkSessionIdentity;
    fn graph_work_managed_run_identity(&self) -> WorthQueryGraphWorkManagedRunIdentity;
    fn graph_work_decision_fact_count(&self) -> usize;
    fn graph_work_runtime_ordinal(&self) -> u64;
    fn graph_work_scope_entity_id(&self) -> Option<EntityId>;
    fn graph_work_capability_identity(&self) -> Option<[u8; 32]>;
    fn graph_work_provider(&self) -> &str;
    fn graph_work_mut(&mut self) -> &mut WorthQueryManagedGraphWorkSession;
    fn graph_work(&self) -> &WorthQueryManagedGraphWorkSession;
    fn operation_authority_identity_bytes(&self) -> &[u8; 32];
    fn operation_definition_identity(&self) -> &[u8; 32];
    fn governed_input_identity(&self) -> Option<&[u8; 32]>;
    fn governed_proposal_identity(&self) -> Option<&[u8; 32]>;
    fn admitted_at(&self) -> AdmittedAt;
    fn inherit_admitted_at(&mut self, admitted_at: AdmittedAt);
    fn record_decision_context_use(&mut self, use_mask: DecisionContextUse);
    fn bind_required_output_demand(&mut self, context: RequiredOutputDemandContext);
    fn take_required_output_demand(&mut self) -> Option<RequiredOutputDemandContext>;
    fn computation_prior(&self) -> Option<ComputationPrior>;
    fn completed_input_prefix(&self) -> Option<usize>;
    fn insert_input_facts(&mut self, expected: usize, count: usize) -> bool;
    fn retain_delegation_proposal_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence);
    fn elevation_request_binding(&self) -> Option<&WorthQueryElevationRequestBinding>;
    fn take_elevation_request_binding(&mut self) -> Option<WorthQueryElevationRequestBinding>;
    fn elevation_approval_binding(&self) -> Option<&WorthQueryElevationApprovalBinding>;
    fn take_elevation_approval_binding(&mut self) -> Option<WorthQueryElevationApprovalBinding>;
    fn elevation_close_binding(&self) -> Option<&WorthQueryElevationCloseBinding>;
    fn take_elevation_close_binding(&mut self) -> Option<WorthQueryElevationCloseBinding>;
    fn mandatory_review_binding(&self) -> Option<&WorthQueryMandatoryReviewBinding>;
    fn take_mandatory_review_binding(&mut self) -> Option<WorthQueryMandatoryReviewBinding>;
    fn original_scope(&self) -> Option<&Arc<WorthQueryObservedScopeSelector>>;
    fn original_mutation_binding(&self) -> Option<std::any::TypeId>;
    fn retain_original_inspection_disclosure(
        &mut self,
        disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
    );
    fn retain_original_program(&mut self, program: Arc<OriginalProgramMeaning>);
    fn original_program(&self) -> Option<&Arc<OriginalProgramMeaning>>;
    fn record_request_identity_work_inner(&mut self, work: ApplicationCanonicalWork);
    fn retain_execution_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence);
    fn refuse_stale_home_authority(
        &self,
        current: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;
    fn validate_installed_entry(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;
    fn validate_mutation_entry(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), MutationHandlerExecutionDenial>;
    fn validate_installed_operation(
        &self,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial>;
}
