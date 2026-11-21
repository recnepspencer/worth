mod change_summary;
mod dependency;
pub mod publication;
pub use dependency::ViewDependency;
pub mod reconstruction_denial;
pub mod registration;
pub mod registry;
pub use registry::PredicateNoticeDisposition;
pub mod retention;


pub use change_summary::changes_from_summary;
pub use publication::{
    ViewChange, ViewPublicationBasis,
};
pub use registry::PredicateRegistration;
pub use registry::RegisteredManagedIndex;
pub use registry::{
    mark_prepared_predicates_unavailable, route_prepared_predicates,
};
pub use registry::{
    ManagedDerivedViewRegistry, PredicatePreparationFactory, PredicatePreparationOwner,
    PredicatePreparationRetirement, PredicateWorld, PreparedManagedViewPublication,
    PreparedPredicateNotices, PreparedPredicatePublication, SourcePredicateAxis,
};
pub use retention::{
    WorthQueryManagedDerivedCollectionBatchRefreshDenial, WorthQueryManagedDerivedMemberToken,
    WorthQueryManagedDerivedStorageQuote, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial, WorthQueryRuleReadableCollectionLifecycleForTest,
};

pub use worth_query_execution_lineage::facade::primary_graph::application_query::observed_source::WorthQueryManagedDerivedViewKey;

/// A bounded, runtime-managed cache of values projected from one public query's
/// rows, keyed by the rows' observed-source keys.
///
/// Open it with `open_managed_derived_view` on a product branch, fill it with
/// `reconstruct_managed_derived_view`, and read through a snapshot. Commits mark
/// affected entries for refresh; the view never serves content from a commit it
/// did not observe. Dropping it disposes of its entries.
pub type WorthQueryManagedDerivedView<Query, Value> =
    retention::WorthQueryManagedDerivedView<Query, WorthQueryManagedDerivedViewKey, Value>;
/// A point-in-time read of a managed derived view at one observed commit.
///
/// Get one from `observe_managed_derived_view`. Reads are refused, never stale,
/// once the view moves past that commit.
pub type WorthQueryManagedDerivedViewSnapshot<Value> =
    retention::WorthQueryManagedDerivedViewSnapshot<WorthQueryManagedDerivedViewKey, Value>;

/// Work performed when a certified member set replaces an older one.
pub struct WorthQueryManagedDerivedViewReconciliation {
    pub(crate) keys: Vec<WorthQueryManagedDerivedViewKey>,
    pub(crate) refreshed_entries: usize,
    pub(crate) retained_entries: usize,
    pub(crate) removed_entries: usize,
}

impl WorthQueryManagedDerivedViewReconciliation {
    pub fn keys(&self) -> &[WorthQueryManagedDerivedViewKey] {
        &self.keys
    }
    pub const fn refreshed_entries(&self) -> usize {
        self.refreshed_entries
    }
    pub const fn retained_entries(&self) -> usize {
        self.retained_entries
    }
    pub const fn removed_entries(&self) -> usize {
        self.removed_entries
    }
}
