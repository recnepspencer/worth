//! Attempt, admission, and coupled-execution boundary ports.

use std::any::Any;
use std::sync::Arc;

/// Lower lineage-side port for an executed primary-graph application attempt.
pub trait ApplicationAttemptPort: Any + Send {
    fn output_binding_type(&self) -> Option<crate::equality_type_id::EqualityTypeId>;
    fn idempotency(
        &self,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
    fn operation_scope(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
    fn product_publication(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding;
    fn admitted_request(
        &self,
    ) -> &crate::domain_computation::authorization::admitted_request::AdmittedRequestCurrentness;
    fn operation(&self) -> &str;
    fn provider_session(
        &self,
    ) -> &crate::domain_computation::provider_session::WorthQueryProviderSessionTerminalBinding;
    fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    fn retain_consumed_outputs(
        &self,
    ) -> Arc<[crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence]>;
    fn runtime_authority(&self) -> u64;
    fn schema_binding(&self)
        -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity;
    fn required_output_key(
        &self,
    ) -> Option<
        Arc<crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandKey>,
    >;
    fn coupled_preparation_view(&self) -> Option<CoupledPreparationView<'_>>;
    fn has_native_publication_role_entity(
        &self,
        role: &str,
        entity: worth_relational::facade::identity::EntityId,
    ) -> bool;
    fn as_any(&self) -> &(dyn Any + Send);
}

/// Borrowed view of coupled attempt preparation metadata.
pub struct CoupledPreparationView<'a> {
    pub completion:
        &'a crate::domain_computation::primary_graph::output_lineage::CoupledOutputCompletion,
    pub after_own: Option<
        &'a worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    pub output_meaning:
        Option<&'a worth_relational::facade::mvcc::RequiredOutputPublicationMeaning>,
    pub output_producer: Option<&'a str>,
}

/// Lower lineage-side port for a coupled application member during support checks.
pub trait CoupledMemberPort {
    fn position(&self) -> usize;
    fn producer_route(
        &self,
    ) -> Option<(
        &str,
        &worth_relational::facade::mvcc::RequiredOutputPublicationMeaning,
    )>;
    fn extracted_attempt(&self) -> &dyn ApplicationAttemptPort;
}

/// Lower lineage-side port for an admitted application operation.
pub trait AdmittedOperationPort {
    fn operation(&self) -> &str;
    fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
    fn refuse_stale_home_authority(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>;
    fn validate_projection_authority(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed_operations: &worth_query_installation::facade::InstalledOperationCatalog,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial>;
    fn mutation_product(&self) -> Option<&crate::basis::WorthQueryProductBranchLease>;
    fn mutation_snapshot(&self) -> Option<&worth_relational::facade::snapshots::SnapshotHandle>;
    fn projection_work_budget(&self) -> Option<usize>;
    fn source_partition_identity(&self) -> Option<[u8; 32]>;
    fn computation_prior(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_contribution::ComputationPrior>;
    fn retain_graph_contract(
        &self,
    ) -> Arc<worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts>;
    fn admission_identity(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
    fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId;
    fn query_context(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>;
    fn operation_scope_binding(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding;
    fn scope_entity_id(&self) -> worth_relational::facade::identity::EntityId;
    fn scope_entity_kind(&self) -> worth_relational::facade::identity::KindId;
    fn scope_entity_name(&self) -> &str;
    fn operation_definition_identity(&self) -> &[u8; 32];
    fn mutation_preconditions_identity(&self) -> Option<&[u8; 32]>;
    fn governed_input_identity(&self) -> Option<&[u8; 32]>;
    fn governed_proposal_identity(&self) -> Option<&[u8; 32]>;
}

/// Typed extension of `AdmittedOperationPort` preserving `Operation` and `Scope` inference.
pub trait TypedAdmittedOperationPort<Schema>: AdmittedOperationPort {
    type Operation;
    type Scope;
}

impl<T: AdmittedOperationPort + ?Sized> AdmittedOperationPort for Box<T> {
    fn operation(&self) -> &str {
        (**self).operation()
    }
    fn publication_request(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope {
        (**self).publication_request()
    }
    fn refuse_stale_home_authority(
        &self,
        owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    ) -> Result<(), crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial>
    {
        (**self).refuse_stale_home_authority(owner)
    }
    fn validate_projection_authority(
        &self,
        runtime_authority: crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        installed_operations: &worth_query_installation::facade::InstalledOperationCatalog,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryOperationProjectionDenial>
    {
        (**self).validate_projection_authority(
            runtime_authority,
            binding_identity,
            installed_operations,
        )
    }
    fn mutation_product(&self) -> Option<&crate::basis::WorthQueryProductBranchLease> {
        (**self).mutation_product()
    }
    fn mutation_snapshot(&self) -> Option<&worth_relational::facade::snapshots::SnapshotHandle> {
        (**self).mutation_snapshot()
    }
    fn projection_work_budget(&self) -> Option<usize> {
        (**self).projection_work_budget()
    }
    fn source_partition_identity(&self) -> Option<[u8; 32]> {
        (**self).source_partition_identity()
    }
    fn computation_prior(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::application_contribution::ComputationPrior>
    {
        (**self).computation_prior()
    }
    fn retain_graph_contract(
        &self,
    ) -> Arc<worth_query_installation::facade::WorthQueryCompiledApplicationOperationContracts>
    {
        (**self).retain_graph_contract()
    }
    fn admission_identity(
        &self,
    ) -> crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity {
        (**self).admission_identity()
    }
    fn principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        (**self).principal_entity_id()
    }
    fn query_context(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::application_query::ObservedQueryExpressionContext>
{
        (**self).query_context()
    }
    fn operation_scope_binding(
        &self,
    ) -> &crate::domain_computation::authorization::WorthQueryOperationScopeBinding {
        (**self).operation_scope_binding()
    }
    fn scope_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        (**self).scope_entity_id()
    }
    fn scope_entity_kind(&self) -> worth_relational::facade::identity::KindId {
        (**self).scope_entity_kind()
    }
    fn scope_entity_name(&self) -> &str {
        (**self).scope_entity_name()
    }
    fn operation_definition_identity(&self) -> &[u8; 32] {
        (**self).operation_definition_identity()
    }
    fn mutation_preconditions_identity(&self) -> Option<&[u8; 32]> {
        (**self).mutation_preconditions_identity()
    }
    fn governed_input_identity(&self) -> Option<&[u8; 32]> {
        (**self).governed_input_identity()
    }
    fn governed_proposal_identity(&self) -> Option<&[u8; 32]> {
        (**self).governed_proposal_identity()
    }
}

impl<Schema, T: TypedAdmittedOperationPort<Schema> + ?Sized> TypedAdmittedOperationPort<Schema>
    for Box<T>
{
    type Operation = T::Operation;
    type Scope = T::Scope;
}

/// Lower lineage-side port for bound stable observed source facts.
pub trait StableObservedSourceFactsPort {
    fn matches_operation_port(&self, operation: &dyn AdmittedOperationPort) -> bool;
    fn partition_identity(&self) -> [u8; 32];
    fn source_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact];
    fn finish_stable_port(
        self,
        operation: &dyn AdmittedOperationPort,
    ) -> Result<
        (
            crate::domain_computation::primary_graph::application_query::WorthQueryRuntimeSourceIdentity,
            [u8; 32],
        ),
        (),
    >;
}

/// Lower lineage-side port for a certified current proof on required work.
pub trait CurrentProofPort {
    fn certifies_record(
        &self,
        membership: &Arc<
            crate::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership,
        >,
    ) -> bool;
}

/// Lower lineage-side port for test-only committed actor evidence retention.
pub trait WorkflowActorEvidenceRuntimePort {
    fn retain_workflow_actor_evidence_for_test(
        &self,
        changes: &crate::domain_computation::primary_graph::WorthQueryApplicationCommittedChanges,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        approval: worth_relational::facade::identity::EntityId,
    ) -> Result<(), crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>;
}

/// Lower lineage-side port for a shared application-query basis lease.
pub trait BasisLeasePort: Any + Send + Sync {
    fn snapshot_handle(&self) -> &worth_relational::facade::snapshots::SnapshotHandle;
    fn graph_handle(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
    fn release_basis(
        self: Box<Self>,
    ) -> worth_query_execution_runtime::facade::basis_identities::WorthQueryApplicationBasisReleaseOutcome;
}

/// Port enabling `WorthQueryApplicationCommitAuthorityBinding::from_admission`
/// without a reverse dependency on `worth-query-execution-attempt`.
pub trait CommitAuthorityAdmissionPort {
    fn build_commit_authority_binding(
        &self,
        idempotency_binding: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitAuthorityBinding;
}
