//! Inline brand forwarders for an admission's phase state: graph work,
//! idempotency identities, output demand, authorization mode and original input.

use super::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::authorization::{
    WorthQueryCapabilityRevocationBinding, WorthQueryDelegationActivationBinding,
    WorthQueryElevationApprovalBinding, WorthQueryElevationCloseBinding,
    WorthQueryElevationRequestBinding, WorthQueryMandatoryReviewBinding,
};
use crate::domain_computation::primary_graph::{
    application_attempt::CompletedHandlerFactBoundary, ComputationPrior,
    RequiredOutputDemandContext, SealedComputationRetention,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    #[inline]
    pub(crate) fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.core.graph_work_session_identity()
    }

    #[inline]
    pub(crate) fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.core.graph_work_managed_run_identity()
    }

    #[inline]
    pub fn graph_work_branch(&self) -> &worth_relational::facade::history::BranchId {
        self.core.graph_work_branch()
    }

    #[inline]
    pub(crate) fn graph_work_decision_fact_count(&self) -> usize {
        self.core.graph_work_decision_fact_count()
    }

    #[inline]
    pub(crate) fn graph_work_runtime_ordinal(&self) -> u64 {
        self.core.graph_work_runtime_ordinal()
    }

    #[inline]
    pub fn graph_work_principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.core.graph_work_principal_entity_id()
    }

    #[inline]
    pub(crate) fn graph_work_scope_entity_id(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.core.graph_work_scope_entity_id()
    }

    #[inline]
    pub(crate) fn graph_work_capability_identity(&self) -> Option<[u8; 32]> {
        self.core.graph_work_capability_identity()
    }

    #[inline]
    pub(crate) fn graph_work_provider(&self) -> &str {
        self.core.graph_work_provider()
    }

    #[inline]
    pub(crate) fn graph_work_mut(
        &mut self,
    ) -> &mut crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        self.core.graph_work_mut()
    }

    #[inline]
    pub(crate) const fn graph_work(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession {
        self.core.graph_work()
    }

    #[inline]
    pub(crate) const fn operation_authority_identity_bytes(&self) -> &[u8; 32] {
        self.core.operation_authority_identity_bytes()
    }

    #[inline]
    pub(crate) const fn operation_definition_identity(&self) -> &[u8; 32] {
        self.core.operation_definition_identity()
    }

    #[inline]
    pub(crate) const fn governed_input_identity(&self) -> Option<&[u8; 32]> {
        self.core.governed_input_identity()
    }

    #[inline]
    pub(crate) const fn admitted_at(
        &self,
    ) -> Result<
        worth_foundational::facade::CanonicalTimestamp,
        crate::domain_computation::runtime_time::WorthQueryRuntimeTimeDenial,
    > {
        self.core.admitted_at()
    }

    #[inline]
    pub(crate) fn governed_proposal_identity(&self) -> Option<&[u8; 32]> {
        self.core.governed_proposal_identity()
    }

    #[inline]
    pub(crate) fn record_decision_context_use(
        &mut self,
        use_mask: crate::domain_computation::primary_graph::DecisionContextUse,
    ) {
        self.core.record_decision_context_use(use_mask)
    }

    #[inline]
    pub(crate) fn bind_required_output_demand(&mut self, context: RequiredOutputDemandContext) {
        self.core.bind_required_output_demand(context)
    }

    #[inline]
    pub(crate) fn take_required_output_demand(&mut self) -> Option<RequiredOutputDemandContext> {
        self.core.take_required_output_demand()
    }

    #[inline]
    pub(crate) fn record_completed_handler_facts(
        &mut self,
        boundary: CompletedHandlerFactBoundary,
        computation: SealedComputationRetention,
    ) {
        self.core
            .record_completed_handler_facts(boundary, computation)
    }

    #[inline]
    pub(crate) fn computation_prior(&self) -> Option<ComputationPrior> {
        self.core.computation_prior()
    }

    pub(crate) fn completed_input_prefix(&self) -> Option<usize> {
        self.core.completed_input_prefix()
    }

    pub(crate) fn insert_input_facts(&mut self, expected: usize, count: usize) -> bool {
        self.core.insert_input_facts(expected, count)
    }

    #[inline]
    pub(crate) fn retain_delegation_proposal_canonical_work(
        &mut self,
        work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
    ) {
        self.core.retain_delegation_proposal_canonical_work(work)
    }

    #[inline]
    pub(crate) fn delegation_activation_binding(
        &self,
    ) -> Option<&WorthQueryDelegationActivationBinding> {
        self.core.delegation_activation_binding()
    }

    #[inline]
    pub(crate) fn capability_revocation_binding(
        &self,
    ) -> Option<&WorthQueryCapabilityRevocationBinding> {
        self.core.capability_revocation_binding()
    }

    #[inline]
    pub(crate) fn elevation_request_binding(&self) -> Option<&WorthQueryElevationRequestBinding> {
        self.core.elevation_request_binding()
    }

    #[inline]
    pub(crate) fn take_elevation_request_binding(
        &mut self,
    ) -> Option<WorthQueryElevationRequestBinding> {
        self.core.take_elevation_request_binding()
    }

    #[inline]
    pub(crate) fn elevation_approval_binding(&self) -> Option<&WorthQueryElevationApprovalBinding> {
        self.core.elevation_approval_binding()
    }

    #[inline]
    pub(crate) fn take_elevation_approval_binding(
        &mut self,
    ) -> Option<WorthQueryElevationApprovalBinding> {
        self.core.take_elevation_approval_binding()
    }

    #[inline]
    pub(crate) fn elevation_close_binding(&self) -> Option<&WorthQueryElevationCloseBinding> {
        self.core.elevation_close_binding()
    }

    #[inline]
    pub(crate) fn take_elevation_close_binding(
        &mut self,
    ) -> Option<WorthQueryElevationCloseBinding> {
        self.core.take_elevation_close_binding()
    }

    #[inline]
    pub(crate) fn mandatory_review_binding(&self) -> Option<&WorthQueryMandatoryReviewBinding> {
        self.core.mandatory_review_binding()
    }

    #[inline]
    pub(crate) fn take_mandatory_review_binding(
        &mut self,
    ) -> Option<WorthQueryMandatoryReviewBinding> {
        self.core.take_mandatory_review_binding()
    }

    #[inline]
    pub(crate) fn original_scope(
        &self,
    ) -> Option<
        &std::sync::Arc<crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector>,
    > {
        self.core.original_scope()
    }

    #[inline]
    pub(crate) fn original_mutation_binding(&self) -> Option<std::any::TypeId> {
        self.core.original_mutation_binding()
    }

    #[inline]
    pub(crate) fn retain_original_inspection_disclosure(
        &mut self,
        disclosure: Option<std::sync::Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>,
    ) {
        self.core.retain_original_inspection_disclosure(disclosure)
    }

    #[inline]
    pub(crate) fn original_inspection_disclosure(
        &self,
    ) -> Option<&std::sync::Arc<crate::domain_computation::primary_graph::source_disclosure::OriginalSourceInspectionDisclosure>>{
        self.core.original_inspection_disclosure()
    }

    #[inline]
    pub(crate) fn original_material_input(
        &self,
    ) -> Option<(
        &worth_query_declaration::facade::application_operation::ApplicationCanonicalInput,
        &'static str,
        [u8; 32],
    )> {
        self.core.original_material_input()
    }
    pub(crate) fn original_retained_input(
        &self,
    ) -> Option<&std::sync::Arc<dyn std::any::Any + Send + Sync>> {
        self.core.original_retained_input()
    }

    #[inline]
    pub(crate) fn retain_original_program(
        &mut self,
        program: std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>,
    ) {
        self.core.retain_original_program(program)
    }

    #[inline]
    pub(crate) fn original_program(
        &self,
    ) -> Option<&std::sync::Arc<crate::domain_computation::primary_graph::OriginalProgramMeaning>>
    {
        self.core.original_program()
    }

    #[inline]
    pub(crate) fn original_correction_disclosure(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::application_aftermath::OriginalCorrectionDisclosure,
        >,
    > {
        self.core.original_correction_disclosure()
    }
}

#[path = "brand_phases_court_doors_for_test.rs"]
pub mod court_doors_for_test;
