//! Refusals a managed derived view returns when it is opened, observed, read,
//! reconstructed, or refreshed.

/// Why a managed derived view refused to open, observe, reconstruct, refresh,
/// or serve an entry.
///
/// A refusal returns no value and never serves stale content. Most causes
/// name the next step: reconstruct a cold view from a fresh query result,
/// reconcile its membership, refresh an entry, or open a new view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedDerivedViewDenial {
    /// The view definition has an empty name or a zero entry or byte limit.
    InvalidLimits,
    /// The runtime cannot register another managed derived view.
    ViewCapacityExceeded,
    /// The reconstruction has more entries than the view's limit.
    EntryCapacityExceeded,
    /// Retaining the values would exceed the view's byte limit.
    RetainedBytesExceeded,
    /// The view or runtime belongs to a different application runtime.
    ForeignApplication,
    /// The query or view belongs to a different installation.
    ForeignInstallation,
    /// The query does not match the view definition.
    ForeignQuery,
    /// The query is governed; managed derived views accept only public queries.
    AuthorizationRequired,
    /// The view's product branch cannot be observed or was re-incarnated.
    ForeignBranch,
    /// The branch commit differs from the one the view or snapshot observed.
    StaleSource,
    /// Required dependency evidence, a read session, its basis or a closed
    /// root binding is missing, foreign or inconsistent.
    IncompleteDependencies,
    /// Membership declares the same reconstruction root more than once.
    DuplicateRoot { root: worth_relational::facade::identity::EntityId },
    /// The view is cold and must be reconstructed from a fresh query result.
    ColdReconstructionRequired,
    /// A commit changed the view's membership; reconcile it before reading.
    MembershipReconciliationRequired,
    /// The entry's refresh state does not allow this step: an invalidated
    /// entry must be refreshed before it is read, and only an invalidated
    /// entry can be refreshed.
    EntryRefreshRequired,
    /// The query the view needed to run was refused.
    QueryExecutionDenied,
    /// The request asks for more workers than its parent permits.
    WorkerLimitExceedsParent,
    /// The request asks for more memory than its parent permits.
    MemoryLimitExceedsParent,
    /// The request asks for more work than its parent permits.
    WorkLimitExceedsParent,
    /// A policy memory limit refused the reservation; ancestor depth is retained.
    PolicyMemoryExhausted(worth_execution::MemoryLimitDenial),
    /// The process memory limit refused the reservation.
    ProcessMemoryExhausted(worth_execution::MemoryLimitDenial),
    /// The map's declared memory capacity refused the reservation.
    DeclaredMemoryExhausted(worth_execution::MemoryLimitDenial),
    /// Charged bytes cannot be represented by the memory counter.
    ChargedBytesOverflow,
    /// An allocation capacity cannot be represented.
    CapacityOverflow,
    /// The allocator refused the owner stage's declaration storage.
    AllocationUnavailable,
    /// An owner-stage result or denial exceeded its declared byte capacity.
    OwnerResultCapacityExceeded,
    /// A nested request does not descend from the active lease.
    UnrelatedNestedLease,
    /// A serial operation has no active bounded execution scope.
    NoActiveExecutionScope,
    /// The request's equivalence contract is unavailable.
    EquivalenceContractUnavailable,
    /// Cancellation stopped the request.
    Cancelled,
    /// The request deadline elapsed.
    DeadlineElapsed,
    /// The work counter overflowed.
    WorkCounterOverflow,
    /// A worker or owner stage refused a work increment, or settlement refused
    /// a completed root's whole work without charging it. A root is retained
    /// when known; None identifies a stage without a root identity.
    WorkExhausted { root: Option<worth_relational::facade::identity::EntityId> },
    /// Nested execution stopped, overflowed its counter, or exceeded the
    /// calling stage's remaining work ceiling.
    NestedStopped,
    /// A worker panicked while running a closed pair read.
    KernelPanic { root: worth_relational::facade::identity::EntityId },
    /// Owner preparation or projection panicked.
    OwnerPanic,
    /// A worker's result exceeded its declared byte capacity.
    ResultCapacityExceeded { root: worth_relational::facade::identity::EntityId },
    /// A Query read refused this root; the original semantic denial is retained.
    ReadDenied {
        root: worth_relational::facade::identity::EntityId,
        denial: crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotDenial,
    },
    /// The view's revision counter is exhausted; open a new view.
    ViewRevisionExhausted,
    /// The view was dropped; its snapshots can no longer read.
    Disposed,
}
