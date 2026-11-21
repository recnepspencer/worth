//! Lineage, demand, and managed-view boundary ports.

use std::any::Any;
use std::sync::Arc;

/// Runtime hook implemented by `WorthQueryPrimaryGraphApplicationRuntime<S>` so
/// `WorthQueryOutputReadCleanup::close_in_advancement` and `WorthQueryOutputDemandDenial`
/// remain runtime-free in `worth-query-execution-lineage` without changing any public signature.
pub trait WorthQueryReadCleanupRuntime {
    fn close_product_publication_recovery_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >;
}

/// Home-retirement denial subset constructible by lineage-layer owners.
#[derive(Debug)]
pub enum LineageHomeRetirementDenial {
    Busy(worth_query_execution_contracts::facade::WorthQueryHomeRetirementOwner),
    Unsupported(worth_query_execution_contracts::facade::WorthQueryHomeRetirementOwner),
    Handler {
        binding: &'static str,
        cause: worth_query_execution_contracts::facade::WorthQueryOperationHomeRetirementDenial,
    },
    NativeAlias(
        worth_relational::facade::change_source::RelationalRuntimeHandleAliasRetirementDenial,
    ),
    NativeObservation(
        worth_relational::facade::change_source::RelationalRetainedObservationRetirementDenial,
    ),
    BridgeBasis(worth_runtime_bridge::facade::RelationalBridgeBasisRetirementDenial),
    CanonicalSubscription(worth_relational::facade::mvcc::PublicationCompanionRetirementDenial),
    Bridge(worth_runtime_bridge::facade::BridgeHomeRetirementDenial),
    Publication(worth_runtime_world::facade::ConsumedPublicationRetirementDenial),
    Execution(worth_execution::WorkCeilingDenial),
}

impl From<worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial>
    for LineageHomeRetirementDenial
{
    fn from(
        denial: worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial,
    ) -> Self {
        use worth_query_execution_runtime::facade::ports::RuntimeHomeRetirementDenial as Runtime;
        match denial {
            Runtime::Busy(owner) => Self::Busy(owner),
            Runtime::Unsupported(owner) => Self::Unsupported(owner),
            Runtime::NativeAlias(cause) => Self::NativeAlias(cause),
            Runtime::NativeObservation(cause) => Self::NativeObservation(cause),
            Runtime::BridgeBasis(cause) => Self::BridgeBasis(cause),
            Runtime::CanonicalSubscription(cause) => Self::CanonicalSubscription(cause),
            Runtime::Bridge(cause) => Self::Bridge(cause),
            Runtime::Publication(cause) => Self::Publication(cause),
            Runtime::Execution(cause) => Self::Execution(cause),
        }
    }
}

/// Lower lineage-side port for unpublished idempotency store custody.
pub trait UnpublishedIdempotencyPort: Any + Send + Sync {
    fn abandon_preparing_world(
        &self,
        handle: &worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    ) -> Result<(), ()>;

    fn finish_recovered(
        &self,
        handle: &worth_runtime_world::facade::ProductUnpublishedRecoveryHandle,
    );

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Lower lineage-side port for an in-flight preparing commit guard.
pub trait PreparingCommitPort: Send {
    fn is_unprepared(&self) -> bool;
}

/// Lower lineage-side port for managed derived-view registry operations.
pub trait ManagedDerivedViewPort: Any + Send + Sync {
    fn prepare_source_fork(
        self: Arc<Self>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: &worth_relational::facade::mvcc::ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn Any + Send + Sync>, worth_relational::facade::mvcc::CompanionPreflightStop>;

    fn prepare_sources(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &worth_relational::facade::runtime::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<worth_relational::facade::runtime::PreparedManagedIndexSourceRoot>,
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    >;

    fn reserve_source_image(
        &self,
        context: &mut worth_relational::facade::mvcc::PublicationCompanionPreflight<'_>,
    ) -> Result<
        Option<
            worth_relational::facade::mvcc::ReservedCompanionBranchCell<
                worth_relational::facade::runtime::ManagedIndexBranchImage,
            >,
        >,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    >;

    #[allow(clippy::too_many_arguments)]
    fn retain_native_count(
        self: Arc<Self>,
        runtime: u64,
        binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        definition: &worth_relational::facade::schema::DerivedRelationHandle,
        association: &worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
        root: worth_relational::facade::identity::EntityId,
        product: &crate::basis::WorthQueryProductBranchLease,
        data: &worth_relational::facade::runtime::DerivedRelationData,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<u64>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

/// Default empty managed derived-view registry port.
pub struct EmptyManagedDerivedViewPort;

impl ManagedDerivedViewPort for EmptyManagedDerivedViewPort {
    fn prepare_source_fork(
        self: Arc<Self>,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _target: &worth_relational::facade::mvcc::ManagedIndexSourceForkPreparation<'_>,
    ) -> Result<Box<dyn Any + Send + Sync>, worth_relational::facade::mvcc::CompanionPreflightStop>
    {
        Ok(Box::new(()))
    }

    fn prepare_sources(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _source: &worth_relational::facade::runtime::ManagedIndexSourcePreparation<'_>,
    ) -> Result<
        Vec<worth_relational::facade::runtime::PreparedManagedIndexSourceRoot>,
        worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial,
    > {
        Ok(Vec::new())
    }

    fn reserve_source_image(
        &self,
        _context: &mut worth_relational::facade::mvcc::PublicationCompanionPreflight<'_>,
    ) -> Result<
        Option<
            worth_relational::facade::mvcc::ReservedCompanionBranchCell<
                worth_relational::facade::runtime::ManagedIndexBranchImage,
            >,
        >,
        worth_relational::facade::mvcc::CompanionPreflightStop,
    > {
        Ok(None)
    }

    fn retain_native_count(
        self: Arc<Self>,
        _runtime: u64,
        _binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        _definition: &worth_relational::facade::schema::DerivedRelationHandle,
        _association: &worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
        _root: worth_relational::facade::identity::EntityId,
        _product: &crate::basis::WorthQueryProductBranchLease,
        _data: &worth_relational::facade::runtime::DerivedRelationData,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Option<u64> {
        None
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

/// Port enabling `prepare_publication_for_test` on `ManagedDerivedViewCourtDoor`
/// without a reverse dependency on `worth-query-execution-query`.
pub trait ManagedViewPublicationRequestPort<'a>: Sized {
    type Change;
    type Prepared;

    fn prepare_on_managed_views(
        self,
        port: &dyn ManagedDerivedViewPort,
        changes: &[Self::Change],
        maximum_work_units: usize,
    ) -> Self::Prepared;
}

/// Court-door handle over a primary graph's managed derived views.
pub struct ManagedDerivedViewCourtDoor(pub Arc<dyn ManagedDerivedViewPort>);

impl std::ops::Deref for ManagedDerivedViewCourtDoor {
    type Target = Arc<dyn ManagedDerivedViewPort>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ManagedDerivedViewCourtDoor {
    #[doc(hidden)]
    pub fn prepare_publication_for_test<'a, B: ManagedViewPublicationRequestPort<'a>>(
        &self,
        basis: B,
        changes: &[B::Change],
        maximum_work_units: usize,
    ) -> B::Prepared {
        basis.prepare_on_managed_views(&*self.0, changes, maximum_work_units)
    }
}

/// Lower lineage-side port for a source-query read closure.
pub trait SourceReadClosurePort: Any + Send + Sync {
    fn admits_source_identity(
        &self,
        runtime_authority: u64,
        schema_binding: &worth_query_installation::facade::ApplicationSchemaBindingIdentity,
        selection: &crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisSelectionIdentity,
    ) -> bool;

    fn facts(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;

    fn authorization_facts(
        &self,
    ) -> &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact];

    fn is_unpublished_applied(&self) -> bool;

    fn admits_applied_read(
        &self,
        principal: worth_relational::facade::identity::EntityId,
        scope: worth_relational::facade::identity::EntityId,
        candidate: &worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
    ) -> bool;

    fn applied_candidate(
        &self,
    ) -> Option<&worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity>;

    fn applied_support_equal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        before: &worth_relational::facade::snapshots::SnapshotHandle,
        after_own: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        complete: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<bool, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>;

    fn validate_for_publication(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::invariant_projection::ConsumedOutputVerificationStop,
    >;

    fn source_facts_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<
        bool,
        crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure,
    >;

    fn seal_applied_publication(
        &self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Lower lineage-side port for query disclosure governance.
pub trait QueryGovernancePort {
    fn is_public(&self) -> bool;

    fn is_disclosed(
        &self,
        slot: &worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
    ) -> bool;

    fn omission(
        &self,
        slot: &worth_query_declaration::facade::application_query::ApplicationQueryResultSlotKey,
    ) -> Option<(&str, &worth_foundational::facade::AspectValue)>;
}

/// Lower lineage-side port for an unpublished pending producer publication.
pub trait PendingProducerPublicationPort: Any + Send + Sync {
    fn is_closed(&self) -> bool;

    fn publication(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);

    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

/// Lower lineage-side port for applied managed-source equality checks.
pub trait AppliedManagedSourcePort: Any + Send + Sync {
    fn published(
        &self,
    ) -> Option<
        &Arc<crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead>,
    >;

    fn matches_current(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &crate::domain_computation::primary_graph::application_attempt::ManagedSourceRead,
    ) -> Result<bool, worth_relational::facade::transactions::RelationalExecutionDenialCause>;

    fn seal_published(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        after: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), worth_relational::facade::runtime::ManagedIndexSourcePreparationDenial>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}
