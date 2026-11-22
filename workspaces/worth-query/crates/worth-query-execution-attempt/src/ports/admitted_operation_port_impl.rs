use super::*;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;

impl<Schema, Operation, Input, Scope>
    AdmittedOperationExecutionPort<Schema, Operation, Input, Scope>
    for WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    fn mint(
        core: WorthQueryAdmittedOperation,
        tag: BrandTag,
        capability_input: Option<ApplicationSharedInput<Input>>,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        Self::mint(core, tag, capability_input)
    }
    fn into_read_authorization(self) -> Option<WorthQueryRetainedAuthorizationDecisionFacts> {
        Self::into_read_authorization(self)
    }
    fn reentry_parts(&self) -> (&WorthQueryAdmittedOperation, &BrandTag) {
        Self::reentry_parts(self)
    }
    fn belongs_to(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        Self::belongs_to(self, runtime_authority, binding_identity)
    }
    fn validate_projection_authority(
        &self,
        runtime_authority: WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::validate_projection_authority(self, runtime_authority, binding_identity, installed)
    }
    fn validate_installed_mutation_binding(
        &self,
        descriptor: Option<&ApplicationMutationBindingDescriptor>,
    ) -> Result<(), MutationHandlerExecutionDenial> {
        Self::validate_installed_mutation_binding(self, descriptor)
    }
    fn original_input(&self) -> Option<(&ApplicationSharedInput<Input>, [u8; 32])> {
        Self::original_input(self)
    }
    fn bind_delegation_activation(
        self,
        binding: WorthQueryDelegationActivationBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        Self::bind_delegation_activation(self, binding)
    }
    fn bind_capability_revocation(
        self,
        binding: WorthQueryCapabilityRevocationBinding,
        work: WorthQueryCanonicalWorkEvidence,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        Self::bind_capability_revocation(self, binding, work)
    }
    fn bind_elevation_request(
        self,
        binding: WorthQueryElevationRequestBinding,
    ) -> Result<Self, WorthQueryOperationAuthorizationDenial> {
        Self::bind_elevation_request(self, binding)
    }
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
    > {
        Self::bind_elevation_approval(self, execution, binding)
    }
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
    > {
        Self::bind_elevation_close(self, execution, binding)
    }
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
    > {
        Self::bind_mandatory_review(self, execution, binding)
    }
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }
    fn scope_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        Self::scope_entity_kind(self)
    }
    fn has_elevation_lifecycle_binding(&self) -> bool {
        Self::has_elevation_lifecycle_binding(self)
    }
    fn installed_capability_identity(&self) -> Option<[u8; 32]> {
        Self::installed_capability_identity(self)
    }
    fn authorization_mut(&mut self) -> Option<&mut WorthQueryRetainedAuthorizationDecisionFacts> {
        Self::authorization_mut(self)
    }
    fn authorization(&self) -> Option<&WorthQueryRetainedAuthorizationDecisionFacts> {
        Self::authorization(self)
    }
    fn take_authorization_dependencies(
        &mut self,
        bridge: &BridgeAuthorizationRuntime,
    ) -> Result<WorthQueryProviderCommitAuthorization, WorthQueryOperationAuthorizationDenial> {
        Self::take_authorization_dependencies(self, bridge)
    }
    fn retain_installed_operation_fingerprint(&self) -> Arc<str> {
        Self::retain_installed_operation_fingerprint(self)
    }
    fn retain_resource_binding_identity(&self) -> Arc<str> {
        Self::retain_resource_binding_identity(self)
    }
    fn current_authority_lapse(&self) -> Option<WorthQueryAdmissionLapse> {
        Self::current_authority_lapse(self)
    }
    fn retain_admitted_request(&self) -> AdmittedRequestCurrentness {
        Self::retain_admitted_request(self)
    }
    fn publication_request(&self) -> &WorthQueryRequestScope {
        Self::publication_request(self)
    }
    fn bind_source_facts(
        &mut self,
        facts: Vec<WorthQueryApplicationObservedFact>,
        context: Option<ObservedQueryExpressionContext>,
    ) {
        Self::bind_source_facts(self, facts, context)
    }
    fn bind_source_partition(&mut self, identity: [u8; 32]) {
        Self::bind_source_partition(self, identity)
    }
    fn bind_source_expectation_kind(&mut self, kind: WorthQuerySourceExpectationKind) {
        Self::bind_source_expectation_kind(self, kind)
    }
    fn source_partition_identity(&self) -> Option<[u8; 32]> {
        Self::source_partition_identity(self)
    }
    fn source_facts(&self) -> &[WorthQueryApplicationObservedFact] {
        Self::source_facts(self)
    }
    fn signed_creation_principal(&self) -> Option<&Arc<WorthQueryExternalPrincipalIdentity>> {
        Self::signed_creation_principal(self)
    }
    fn graph_work_session_identity(&self) -> WorthQueryGraphWorkSessionIdentity {
        Self::graph_work_session_identity(self)
    }
    fn graph_work_managed_run_identity(&self) -> WorthQueryGraphWorkManagedRunIdentity {
        Self::graph_work_managed_run_identity(self)
    }
    fn graph_work_decision_fact_count(&self) -> usize {
        Self::graph_work_decision_fact_count(self)
    }
    fn graph_work_runtime_ordinal(&self) -> u64 {
        Self::graph_work_runtime_ordinal(self)
    }
    fn graph_work_scope_entity_id(&self) -> Option<EntityId> {
        Self::graph_work_scope_entity_id(self)
    }
    fn graph_work_capability_identity(&self) -> Option<[u8; 32]> {
        Self::graph_work_capability_identity(self)
    }
    fn graph_work_provider(&self) -> &str {
        Self::graph_work_provider(self)
    }
    fn graph_work_mut(&mut self) -> &mut WorthQueryManagedGraphWorkSession {
        Self::graph_work_mut(self)
    }
    fn graph_work(&self) -> &WorthQueryManagedGraphWorkSession {
        Self::graph_work(self)
    }
    fn operation_authority_identity_bytes(&self) -> &[u8; 32] {
        Self::operation_authority_identity_bytes(self)
    }
    fn operation_definition_identity(&self) -> &[u8; 32] {
        Self::operation_definition_identity(self)
    }
    fn governed_input_identity(&self) -> Option<&[u8; 32]> {
        Self::governed_input_identity(self)
    }
    fn governed_proposal_identity(&self) -> Option<&[u8; 32]> {
        Self::governed_proposal_identity(self)
    }
    fn record_decision_context_use(&mut self, use_mask: DecisionContextUse) {
        Self::record_decision_context_use(self, use_mask)
    }
    fn bind_required_output_demand(&mut self, context: RequiredOutputDemandContext) {
        Self::bind_required_output_demand(self, context)
    }
    fn take_required_output_demand(&mut self) -> Option<RequiredOutputDemandContext> {
        Self::take_required_output_demand(self)
    }
    fn computation_prior(&self) -> Option<ComputationPrior> {
        Self::computation_prior(self)
    }
    fn completed_input_prefix(&self) -> Option<usize> {
        Self::completed_input_prefix(self)
    }
    fn insert_input_facts(&mut self, expected: usize, count: usize) -> bool {
        Self::insert_input_facts(self, expected, count)
    }
    fn retain_delegation_proposal_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence) {
        Self::retain_delegation_proposal_canonical_work(self, work)
    }
    fn elevation_request_binding(&self) -> Option<&WorthQueryElevationRequestBinding> {
        Self::elevation_request_binding(self)
    }
    fn take_elevation_request_binding(&mut self) -> Option<WorthQueryElevationRequestBinding> {
        Self::take_elevation_request_binding(self)
    }
    fn elevation_approval_binding(&self) -> Option<&WorthQueryElevationApprovalBinding> {
        Self::elevation_approval_binding(self)
    }
    fn take_elevation_approval_binding(&mut self) -> Option<WorthQueryElevationApprovalBinding> {
        Self::take_elevation_approval_binding(self)
    }
    fn elevation_close_binding(&self) -> Option<&WorthQueryElevationCloseBinding> {
        Self::elevation_close_binding(self)
    }
    fn take_elevation_close_binding(&mut self) -> Option<WorthQueryElevationCloseBinding> {
        Self::take_elevation_close_binding(self)
    }
    fn mandatory_review_binding(&self) -> Option<&WorthQueryMandatoryReviewBinding> {
        Self::mandatory_review_binding(self)
    }
    fn take_mandatory_review_binding(&mut self) -> Option<WorthQueryMandatoryReviewBinding> {
        Self::take_mandatory_review_binding(self)
    }
    fn original_scope(&self) -> Option<&Arc<WorthQueryObservedScopeSelector>> {
        Self::original_scope(self)
    }
    fn original_mutation_binding(&self) -> Option<std::any::TypeId> {
        Self::original_mutation_binding(self)
    }
    fn retain_original_inspection_disclosure(
        &mut self,
        disclosure: Option<Arc<OriginalSourceInspectionDisclosure>>,
    ) {
        Self::retain_original_inspection_disclosure(self, disclosure)
    }
    fn retain_original_program(&mut self, program: Arc<OriginalProgramMeaning>) {
        Self::retain_original_program(self, program)
    }
    fn original_program(&self) -> Option<&Arc<OriginalProgramMeaning>> {
        Self::original_program(self)
    }
    fn record_request_identity_work_inner(&mut self, work: ApplicationCanonicalWork) {
        Self::record_request_identity_work_inner(self, work)
    }
    fn retain_execution_canonical_work(&mut self, work: WorthQueryCanonicalWorkEvidence) {
        Self::retain_execution_canonical_work(self, work)
    }
    fn refuse_stale_home_authority(
        &self,
        current: RuntimeWorldOwnerIdentity,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::refuse_stale_home_authority(self, current)
    }
    fn validate_installed_entry(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::validate_installed_entry(self, runtime, binding, installed)
    }
    fn validate_mutation_entry(
        &self,
        runtime: WorthQueryRuntimeAuthorityIdentity,
        binding: &ApplicationSchemaBindingIdentity,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), MutationHandlerExecutionDenial> {
        Self::validate_mutation_entry(self, runtime, binding, installed)
    }
    fn validate_installed_operation(
        &self,
        installed: &InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationAuthorizationDenial> {
        Self::validate_installed_operation(self, installed)
    }
}
