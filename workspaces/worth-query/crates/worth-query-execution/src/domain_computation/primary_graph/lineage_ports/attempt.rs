//! Attempt, admission, basis-lease, and coupled-execution port implementations.

use std::any::Any;
use std::sync::Arc;

use worth_query_execution_lineage::facade::ports::{
    AdmittedOperationPort, ApplicationAttemptPort, BasisLeasePort, CoupledMemberPort,
    CoupledPreparationView, CurrentProofPort, StableObservedSourceFactsPort,
    TypedAdmittedOperationPort,
};
use worth_query_execution_lineage::facade::primary_graph::{
    application_output_demand::{RequiredWorkMembership, WorthQueryOutputDemandKey},
    application_query::WorthQueryRuntimeSourceIdentity,
    WorthQueryOperationProjectionDenial,
};

use crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::application_contribution::producer::demand::progression::required_wave::performed::certification::CurrentProof;
use crate::domain_computation::primary_graph::application_query::observed_source::prepared_expectation::BoundStableObservedSourceFacts;
use crate::domain_computation::primary_graph::application_query::resource_lifecycle::WorthQueryApplicationBasisLease;
use crate::domain_computation::primary_graph::provider::application_attempt_state::coupled_preparation::CoupledApplicationMember;

impl ApplicationAttemptPort for WorthQueryPrimaryGraphApplicationAttempt {
    fn output_binding_type(&self) -> Option<crate::equality_type_id::EqualityTypeId> {
        self.output_binding_type()
    }

    fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding
    {
        self.idempotency()
    }

    fn operation_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        self.affinity().operation_scope()
    }

    fn product_publication(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding
    {
        self.affinity().product_publication()
    }

    fn admitted_request(
        &self,
    ) -> &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness {
        self.affinity().admitted_request()
    }

    fn operation(&self) -> &str {
        self.affinity().operation()
    }

    fn provider_session(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding {
        self.affinity().provider_session()
    }

    fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.affinity().publication_request()
    }

    fn retain_consumed_outputs(
        &self,
    ) -> Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>
    {
        self.retain_consumed_outputs()
    }

    fn runtime_authority(&self) -> u64 {
        self.affinity().runtime_authority_u64()
    }

    fn schema_binding(&self) -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity {
        self.affinity().installed_binding()
    }

    fn required_output_key(&self) -> Option<Arc<WorthQueryOutputDemandKey>> {
        self.required_output_key()
    }

    fn coupled_preparation_view(&self) -> Option<CoupledPreparationView<'_>> {
        let prep = self.coupled_preparation()?;
        Some(CoupledPreparationView {
            completion: &prep.completion,
            after_own: prep.after_own.as_ref(),
            output_meaning: prep.outputs.as_ref().map(|outputs| &outputs.meaning),
            output_producer: self.output_producer(),
        })
    }

    fn has_native_publication_role_entity(
        &self,
        role: &str,
        entity: worth_relational::facade::identity::EntityId,
    ) -> bool {
        self.native_publication_roles().any(|(candidate_role, _, _, reference)| {
            candidate_role == role
                && matches!(
                    reference,
                    worth_relational::facade::transactions::EntityReference::Existing(existing)
                        if *existing == entity
                )
        })
    }

    fn as_any(&self) -> &(dyn Any + Send) {
        self
    }
}

impl CoupledMemberPort for CoupledApplicationMember {
    fn position(&self) -> usize {
        self.position
    }

    fn producer_route(
        &self,
    ) -> Option<(&str, &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning)> {
        self.producer_route()
    }

    fn extracted_attempt(&self) -> &dyn ApplicationAttemptPort {
        self.extracted_attempt()
    }
}

impl<Schema, Operation, Input, Scope> AdmittedOperationPort
    for WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    fn operation(&self) -> &str {
        self.operation()
    }

    fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        self.publication_request()
    }

    fn refuse_stale_home_authority(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        self.refuse_stale_home_authority(owner)
    }

    fn validate_projection_authority(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed_operations: &worth_query_installation::facade::InstalledOperationCatalog,
    ) -> Result<(), WorthQueryOperationProjectionDenial> {
        self.validate_projection_authority(runtime_authority, binding_identity, installed_operations)
            .map_err(WorthQueryOperationProjectionDenial::from)
    }

    fn mutation_product(&self) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        self.graph_work().mutation_product()
    }

    fn mutation_snapshot(&self) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        self.graph_work().mutation_snapshot()
    }

    fn projection_work_budget(&self) -> Option<usize> {
        self.allowed_graph_contract().projection_work_budget()
    }

    fn source_partition_identity(&self) -> Option<[u8; 32]> {
        self.source_partition_identity()
    }

    fn computation_prior(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_contribution::ComputationPrior>
    {
        self.computation_prior()
    }

    fn retain_graph_contract(
        &self,
    ) -> Arc<worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts> {
        self.retain_graph_contract()
    }

    fn admission_identity(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity {
        self.admission_identity()
    }

    fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.graph_work_principal_entity_id()
    }

    fn query_context(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>
    {
        self.query_context()
    }

    fn operation_scope_binding(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        self.operation_scope_binding()
    }

    fn scope_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.scope_entity_id()
    }

    fn scope_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        self.scope_entity_kind()
    }

    fn scope_entity_name(&self) -> &str {
        self.scope_entity_name()
    }

    fn operation_definition_identity(&self) -> &[u8; 32] {
        self.operation_definition_identity()
    }

    fn mutation_preconditions_identity(&self) -> Option<&[u8; 32]> {
        self.mutation_preconditions().identity()
    }

    fn governed_input_identity(&self) -> Option<&[u8; 32]> {
        self.governed_input_identity()
    }

    fn governed_proposal_identity(&self) -> Option<&[u8; 32]> {
        self.governed_proposal_identity()
    }
}

impl<Schema, Operation, Input, Scope> TypedAdmittedOperationPort<Schema>
    for WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>
{
    type Operation = Operation;
    type Scope = Scope;
}

impl StableObservedSourceFactsPort for BoundStableObservedSourceFacts {
    fn matches_operation_port(&self, operation: &dyn AdmittedOperationPort) -> bool {
        self.matches_admission_identity_and_product(
            operation.admission_identity(),
            operation.mutation_product(),
        )
    }

    fn partition_identity(&self) -> [u8; 32] {
        self.bound_source().partition_identity()
    }

    fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact] {
        self.source_facts()
    }

    fn finish_stable_port(
        self,
        operation: &dyn AdmittedOperationPort,
    ) -> Result<(WorthQueryRuntimeSourceIdentity, [u8; 32]), ()> {
        if !self.matches_operation_port(operation) {
            return Err(());
        }
        let bound = self.bound_source();
        Ok((bound.recorded_source_identity(), bound.partition_identity()))
    }
}

impl<T> CurrentProofPort for CurrentProof<T> {
    fn certifies_record(&self, membership: &Arc<RequiredWorkMembership>) -> bool {
        self.certifies_record(membership)
    }
}

impl BasisLeasePort for WorthQueryApplicationBasisLease {
    fn snapshot_handle(&self) -> &worth_relational::facade::snapshots::SnapshotHandle {
        self.snapshot_handle()
    }

    fn graph_handle(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle {
        self.graph_handle()
    }

    fn release_basis(
        self: Box<Self>,
    ) -> worth_query_execution_runtime::facade::basis_identities::WorthQueryApplicationBasisReleaseOutcome
    {
        self.release().outcome()
    }
}
