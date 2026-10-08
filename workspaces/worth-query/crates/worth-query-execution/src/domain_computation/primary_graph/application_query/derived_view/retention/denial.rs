//! Refusals a managed derived view returns when it is opened, observed, read,
//! reconstructed, or refreshed.

/// Why a managed derived view refused to open, observe, reconstruct, refresh,
/// or serve an entry.
///
/// A refusal returns no value and never serves stale content. Most causes
/// name the next step: reconstruct a cold view from a fresh query result,
/// reconcile its membership, refresh an entry, or open a new view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// The view's membership or an entry has no recorded dependencies.
    IncompleteDependencies,
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
    /// The view's revision counter is exhausted; open a new view.
    ViewRevisionExhausted,
    /// The view was dropped; its snapshots can no longer read.
    Disposed,
}

/// A shared-loan pair refresh preserves either managed-view provenance and
/// lifecycle refusal, or the actual ordinary execution/aggregate budget cause.
/// No refusal installs a partially projected entry.
#[derive(Debug)]
pub enum WorthQueryManagedDerivedCollectionBatchRefreshDenial {
    /// The intact second result was executed under a different shared loan.
    ForeignBatch,
    View(WorthQueryManagedDerivedViewDenial),
    Read(crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBatchReadDenial),
}

impl From<WorthQueryManagedDerivedViewDenial>
    for WorthQueryManagedDerivedCollectionBatchRefreshDenial
{
    fn from(denial: WorthQueryManagedDerivedViewDenial) -> Self {
        Self::View(denial)
    }
}
