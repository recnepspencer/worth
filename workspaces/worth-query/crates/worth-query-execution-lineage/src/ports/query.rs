//! Query, authorization, and graph-integration boundary ports.

use std::any::Any;
use worth_relational::facade::{
    authorization::RelationalAuthorizationTraversalDirection,
    identity::{EntityId, KindId, RelationId},
    runtime::{RelationalAppliedCandidateReadIdentity, RelationalPinnedRead, RelationalRuntime},
    snapshots::SnapshotHandle,
};

/// Lower lineage-side port for owned managed-index denial causes.
pub trait ManagedIndexCausePort: Send + Sync {
    fn cause(
        &self,
    ) -> crate::domain_computation::primary_graph::application_query::managed_index::WorthQueryManagedIndexDenialCause<'_>;
}

/// Lower lineage-side port for diagnostic projection comparison admission.
pub trait DiagnosticComparisonAdmissionPort {
    fn graph(&self) -> &worth_query_installation::facade::WorthQueryInstalledGraphReadContract;
}

/// Conventional authorization locus observed during source read support validation.
pub enum SourceAuthorizationLocus<'a> {
    Entity(EntityId),
    Relation(RelationId),
    Field(EntityId, &'a worth_foundational::facade::AspectFieldLocator),
    Adjacency(EntityId, KindId, RelationalAuthorizationTraversalDirection),
}

/// Lower lineage-side port for retained source read authorization support.
pub trait SourceReadSupportPort: Any + Send + Sync {
    fn belongs_to_session(
        &self,
        session: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool;

    fn validate_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
    ) -> Result<(), crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop>;

    fn same_conventional_support_dyn(
        &self,
        other: &dyn SourceReadSupportPort,
        admit: &mut dyn FnMut(u64) -> Result<(), Box<dyn Any>>,
    ) -> Result<bool, Box<dyn Any>>;

    fn visit_loci_dyn(
        &self,
        visit: &mut dyn FnMut(SourceAuthorizationLocus<'_>) -> Result<(), Box<dyn Any>>,
    ) -> Result<bool, Box<dyn Any>>;

    fn principal_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact];

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Lower lineage-side port for primary-graph integration handle operations.
pub trait GraphIntegrationPort: Any + Send + Sync {
    fn relational_runtime_instance_id(&self) -> u64;

    fn with_runtime_dyn(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        visit: &mut dyn FnMut(&RelationalRuntime),
    );

    fn release_query_snapshot(&self, snapshot: &SnapshotHandle);

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Applied output selection returned across the source-query reader port.
pub struct AppliedOutputsSelection {
    pub replaced: Vec<(Option<crate::equality_type_id::EqualityTypeId>, [u8; 32])>,
    pub outputs:
        Vec<crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputEvidence>,
}

/// Lower lineage-side port for source-query reader access inside operation projection.
pub trait SourceQueryReaderPort {
    fn applied_outputs(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding_identity: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        family: &'static str,
        producer: EntityId,
        pin: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<
        AppliedOutputsSelection,
        crate::domain_computation::primary_graph::WorthQueryCurrentOutputDenial,
    >;

    fn with_committed_runtime_dyn(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        visit: &mut dyn FnMut(&RelationalRuntime),
    );

    fn applied_source_publication(
        &self,
    ) -> Option<(
        &crate::domain_computation::primary_graph::application_contribution::QualifiedOutputSource,
        &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    )>;

    fn observation_control(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial,
    >;

    fn record_observation_failure(
        &self,
        cause: &crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial,
    );

    fn record_predecessor_read(
        &self,
        read: crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryHistoricalProducerRead,
    );

    fn authorize_observation_view(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        installed_view: worth_query_installation::facade::InstalledOperationView<'_>,
    ) -> Result<
        std::sync::Arc<
            crate::domain_computation::primary_graph::application_attempt::SourceReadAuthorization,
        >,
        crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial,
    >;

    fn select_predecessor_input(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
        scope: EntityId,
        partition: [u8; 32],
    ) -> Result<
        (
            worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
            worth_relational::facade::mvcc::RelationalPredecessorInput,
        ),
        crate::domain_computation::primary_graph::application_contribution::producer::observation_reads::WorthQueryProducerObservationReadDenial,
    >;

    /// On a coupled candidate: whether the unpublished envelope already
    /// claims group scheme `scheme`. When it does not, the decision's group
    /// read is retained with the envelope's owner, so any member appended
    /// after it that claims `scheme` refuses the envelope (#361). `None`
    /// when no coupled candidate is lent to this reader.
    fn retain_applied_group_read(&self, scheme: &str) -> Option<bool>;

    /// Returns `(installed_schema, mutation_projection, installed_packages, predecessor_access, original_access)` as `'static` `Any` refs.
    fn observation_components_any(
        &self,
    ) -> (
        &dyn Any,
        &dyn Any,
        &dyn Any,
        Option<&dyn Any>,
        Option<&dyn Any>,
    );
}

/// Lower lineage-side port for workflow compilation reuse and instance progress retention.
pub trait WorkflowRetentionPort: Any + Send + Sync {
    fn compilation_reuse_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowCompilationReuseCounters;

    fn instance_progress_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowInstanceProgressCounters;

    fn release_instance_progress(&self);

    fn release_instance_progress_for_branch(&self, branch_occurrence: u64);

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Default empty workflow retention port.
pub struct EmptyWorkflowRetentionPort;

impl WorkflowRetentionPort for EmptyWorkflowRetentionPort {
    fn compilation_reuse_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowCompilationReuseCounters
{
        Default::default()
    }

    fn instance_progress_counters(
        &self,
    ) -> crate::domain_computation::primary_graph::workflow::WorthQueryWorkflowInstanceProgressCounters
{
        Default::default()
    }

    fn release_instance_progress(&self) {}

    fn release_instance_progress_for_branch(&self, _branch_occurrence: u64) {}

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}
