//! Typed internal ports bridging the runtime substrate to higher primary-graph
//! composition without circular dependencies.

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_foundational::facade::{
    DurableCompositeCommitIdentity, DurableRecoveryClaimIdentity, ManagedRunObligationIdentity,
    RecoveryClaimDisposition,
};
use worth_query_execution_contracts::facade::{
    HeadCellRegistrationStop, QueryDispatchJournalDenial, QueryRecordedRecoveryClaim,
    WorthQueryHomeRetirementOwner,
};
use worth_query_execution_obligations::facade::query_obligations::{
    QueryManagedCleanupOriginal, QueryManagedRunPosture, QueryManagedRunRecovery,
    QueryObligationDenial,
};
use worth_relational::facade::branch::RelationalBranchIdentity;
use worth_relational::facade::mvcc::RelationalPublicationCompanion;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_runtime_bridge::facade::RelationalBridgeCanonicalSubscription;

use crate::basis::WorthQueryProductBranchLease;

/// Lower runtime-side invalidation port for one installed Relational source.
pub trait RelationalSourceInvalidationPort:
    RelationalPublicationCompanion + Debug + Send + Sync + 'static
{
    fn mint_cell_at_head(
        &self,
        subscription: &RelationalBridgeCanonicalSubscription,
        runtime: &RelationalRuntime,
        branch: &RelationalBranchIdentity,
    ) -> Result<(), HeadCellRegistrationStop>;

    fn as_publication_companion(self: Arc<Self>) -> Arc<dyn RelationalPublicationCompanion>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);

    fn into_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;

    #[doc(hidden)]
    fn latest_position_for_test(
        &self,
    ) -> Option<worth_relational::facade::publication::PatchStreamPosition>;

    #[doc(hidden)]
    fn touched_commits_after_for_test(
        &self,
        branch: &worth_relational::facade::history::BranchId,
        cursor: Option<worth_relational::facade::publication::PatchStreamPosition>,
        ceiling: Option<worth_relational::facade::history::CommitId>,
        limit: usize,
        interest: crate::commit_touches::CommitTouchInterest<'_>,
    ) -> crate::commit_touches::TouchedCommits;
}

/// Lower runtime-side maintenance hook invoked after an advancement phase ends.
pub trait HomeMaintenanceAfterAdvancement: Any + Debug + Send + Sync {
    fn after_advancement(&self, request: &ExecutionRequest<'_, '_>);

    fn as_any(&self) -> &(dyn Any + Send + Sync);

    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);
}

/// Lower runtime-side hook recording a product-branch fork on output lineage.
pub trait ForkLineagePort: Send + Sync {
    fn register_fork(
        &self,
        source: &worth_runtime_world::facade::ProductBranchObservation,
        destination: &worth_runtime_world::facade::ProductBranchObservation,
    );
}

/// Lower runtime-side hook entering per-occurrence branch commit coordination.
pub trait BranchCommitCoordinationPort: Send + Sync {
    fn with_occurrence_coordination(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
        body: &mut dyn FnMut(),
    );
}

/// Primary-graph read-set registration custody retained by a completed receipt.
pub trait PrimaryReadCustody: Any + Send + Sync {
    fn grouped_count(&self) -> usize;
    fn contains_key(&self, identity: &str) -> bool;
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Primary-graph mutation snapshot lease carried inside a managed graph-work session.
pub trait PrimaryMutationLease: Any + Send + Sync {
    fn snapshot(&self) -> &SnapshotHandle;
    fn product(&self) -> &WorthQueryProductBranchLease;
    fn as_any(&self) -> &(dyn Any + Send + Sync);
    fn as_any_mut(&mut self) -> &mut (dyn Any + Send + Sync);
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

/// Lower runtime-side port for recording managed-run obligation history.
pub trait ManagedRunObligationPort: Any + Send + Sync {
    fn retain_managed_run(
        &self,
        request: &ExecutionRequest<'_, '_>,
        logical: &str,
        attempt: &str,
    ) -> Result<
        Option<(ManagedRunObligationIdentity, Option<QueryObligationDenial>)>,
        QueryObligationDenial,
    >;

    fn settle_managed_run_retention(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
    ) -> Result<bool, QueryObligationDenial>;

    fn associate_managed_run_original(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        original: QueryManagedCleanupOriginal,
    ) -> Result<(), QueryObligationDenial>;

    fn associate_managed_run_recovery(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        cause: QueryManagedRunRecovery,
    ) -> Result<(), QueryObligationDenial>;

    fn record_managed_run_handoff(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ManagedRunObligationIdentity,
        posture: QueryManagedRunPosture,
        current_attempt: &str,
    ) -> Result<(), QueryObligationDenial>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Lower runtime-side port for recording durable recovery claim dispositions.
pub trait RecoveryDispatchJournalPort: Any + Send + Sync {
    fn dispose_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
        identity: DurableRecoveryClaimIdentity,
        terminal: RecoveryClaimDisposition,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryDispatchJournalDenial>;

    fn recorded_recovery_claim(
        &self,
        original: &DurableCompositeCommitIdentity,
    ) -> Result<Option<QueryRecordedRecoveryClaim>, QueryDispatchJournalDenial>;

    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Lower runtime-side home retirement refusal for runtime-owned handles.
#[derive(Debug)]
pub enum RuntimeHomeRetirementDenial {
    Busy(WorthQueryHomeRetirementOwner),
    Unsupported(WorthQueryHomeRetirementOwner),
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

/// Opaque payload carried across the session-commit boundary when output adoption
/// was prepared by the primary provider.
pub struct PreparedOutputAdoptionPayload(Box<dyn Any + Send>);

impl PreparedOutputAdoptionPayload {
    pub fn new<T: Any + Send>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn downcast<T: Any + Send>(self) -> Result<Box<T>, Self> {
        self.0.downcast::<T>().map_err(Self)
    }

    pub fn downcast_ref<T: Any + Send>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }
}

impl Debug for PreparedOutputAdoptionPayload {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PreparedOutputAdoptionPayload")
            .finish_non_exhaustive()
    }
}

/// Opaque primary-graph product-publication terminal carried by commit outcomes.
pub struct OpaqueProductPublicationOutcome {
    payload: Box<dyn Any + Send>,
    recovery_required: bool,
}

impl OpaqueProductPublicationOutcome {
    pub fn new<T: Any + Send>(value: T, recovery_required: bool) -> Self {
        Self {
            payload: Box::new(value),
            recovery_required,
        }
    }

    pub const fn recovery_required(&self) -> bool {
        self.recovery_required
    }

    pub fn downcast<T: Any + Send>(self) -> Result<Box<T>, Self> {
        let recovery_required = self.recovery_required;
        self.payload.downcast::<T>().map_err(|payload| Self {
            payload,
            recovery_required,
        })
    }

    pub fn downcast_ref<T: Any + Send>(&self) -> Option<&T> {
        self.payload.downcast_ref::<T>()
    }
}

impl Debug for OpaqueProductPublicationOutcome {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpaqueProductPublicationOutcome")
            .field("recovery_required", &self.recovery_required)
            .finish_non_exhaustive()
    }
}

/// Opaque primary-graph inspection target handle stored on a read session.
#[derive(Clone)]
pub struct OpaqueInspectionTarget(Arc<dyn Any + Send + Sync>);

impl OpaqueInspectionTarget {
    pub fn new<T: Any + Send + Sync>(value: Arc<T>) -> Self {
        Self(value)
    }

    pub fn ptr_eq_arc<T: Any + Send + Sync>(&self, other: &Arc<T>) -> bool {
        let other: Arc<dyn Any + Send + Sync> = Arc::clone(other) as Arc<dyn Any + Send + Sync>;
        Arc::ptr_eq(&self.0, &other)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }

    pub fn downcast_arc<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        Arc::clone(&self.0).downcast::<T>().ok()
    }

    pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }
}

impl Debug for OpaqueInspectionTarget {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("OpaqueInspectionTarget")
            .finish_non_exhaustive()
    }
}

/// Lower runtime-side port for an opaque application inspection target.
pub trait InspectionTargetPort: Any + Send + Sync {
    fn contract(
        &self,
    ) -> &worth_query_installation::facade::WorthQueryInstalledApplicationInspectionContract;
    fn schema_binding(&self)
        -> &worth_query_installation::facade::ApplicationSchemaBindingIdentity;
    fn runtime_authority(&self) -> u64;
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

/// Borrow access to a runtime's underlying product runtime.
pub trait ProductRuntimeRef {
    fn product_runtime(
        &self,
    ) -> &crate::domain_computation::execution_runtime::product_world::WorthQueryProductRuntime;
}

/// Port for an opaque output-demand denial carried across session-commit deferral.
pub trait PrerequisiteDenialPort: Any + Debug + Send + Sync {
    fn clone_box(&self) -> Box<dyn PrerequisiteDenialPort>;
    fn eq_port(&self, other: &dyn PrerequisiteDenialPort) -> bool;
    fn as_any(&self) -> &(dyn Any + Send + Sync);
    fn into_any(self: Box<Self>) -> Box<dyn Any + Send + Sync>;
}

/// Opaque output-demand denial carried by a deferred provider-session commit.
pub struct OpaquePrerequisiteDenial(Box<dyn PrerequisiteDenialPort>);

impl OpaquePrerequisiteDenial {
    pub fn new<T: PrerequisiteDenialPort>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn downcast<T: Any + Send + Sync>(self) -> Result<Box<T>, Box<dyn Any + Send + Sync>> {
        self.0.into_any().downcast::<T>()
    }

    pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }
}

impl Clone for OpaquePrerequisiteDenial {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

impl Debug for OpaquePrerequisiteDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.as_ref(), f)
    }
}

impl PartialEq for OpaquePrerequisiteDenial {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_port(other.0.as_ref())
    }
}

impl Eq for OpaquePrerequisiteDenial {}

/// Opaque output-demand denial kind carried by a deferred provider-session commit kind.
pub struct OpaquePrerequisiteDenialKind(Box<dyn PrerequisiteDenialPort>);

impl OpaquePrerequisiteDenialKind {
    pub fn new<T: PrerequisiteDenialPort>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn downcast<T: Any + Send + Sync>(self) -> Result<Box<T>, Box<dyn Any + Send + Sync>> {
        self.0.into_any().downcast::<T>()
    }

    pub fn downcast_ref<T: Any + Send + Sync>(&self) -> Option<&T> {
        self.0.as_any().downcast_ref::<T>()
    }
}

impl Clone for OpaquePrerequisiteDenialKind {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

impl Debug for OpaquePrerequisiteDenialKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.0.as_ref(), f)
    }
}

impl PartialEq for OpaquePrerequisiteDenialKind {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_port(other.0.as_ref())
    }
}

impl Eq for OpaquePrerequisiteDenialKind {}

/// Port allowing an application-attempt basis to bind onto session-bound reads
/// without introducing a reverse crate edge into `worth-query-execution-attempt`.
pub trait SessionBoundAttemptBasisPort: Sized {
    type Affinity;

    fn bind_to_session_bound_reads(
        self,
        reads: &crate::domain_computation::provider_session::WorthQuerySessionBoundReadsAndEffects<
            '_,
        >,
    ) -> Result<Self::Affinity, ()>;
}
