mod change_summary;
mod dependency;
mod publication;
mod registration;
mod registry;
mod retention;

use worth_foundational::facade::CanonicalDigestId;
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation::primary_graph) use change_summary::changes_from_summary;
pub(in crate::domain_computation::primary_graph) use publication::{
    ViewChange, ViewPublicationBasis,
};
pub(in crate::domain_computation::primary_graph) use registry::{
    ManagedDerivedViewRegistry, PreparedManagedViewPublication,
};
pub use retention::{
    WorthQueryManagedDerivedCollectionBatchRefreshDenial, WorthQueryManagedDerivedMemberToken,
    WorthQueryManagedDerivedStorageQuote, WorthQueryManagedDerivedValue,
    WorthQueryManagedDerivedViewDenial,
};

/// Query-issued row identity: an exact source root and parameter binding,
/// never a consumer-selected scene key.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryManagedDerivedViewKey {
    model_root: EntityId,
    root: EntityId,
    parameters: CanonicalDigestId,
}

impl WorthQueryManagedDerivedViewKey {
    const fn new(model_root: EntityId, root: EntityId, parameters: CanonicalDigestId) -> Self {
        Self {
            model_root,
            root,
            parameters,
        }
    }

    pub const fn model_root(&self) -> EntityId {
        self.model_root
    }

    pub const fn root(&self) -> EntityId {
        self.root
    }

    pub const fn parameters(&self) -> &CanonicalDigestId {
        &self.parameters
    }
}

impl<Query> super::WorthQueryObservedSource<Query> {
    /// A lookup key from Query's issued row source, not a scene-authored key.
    pub fn managed_derived_view_key(&self) -> WorthQueryManagedDerivedViewKey {
        WorthQueryManagedDerivedViewKey::new(
            self.model_root,
            self.source_root(),
            self.parameter_binding_identity,
        )
    }
}

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
    pub(super) keys: Vec<WorthQueryManagedDerivedViewKey>,
    pub(super) refreshed_entries: usize,
    pub(super) retained_entries: usize,
    pub(super) removed_entries: usize,
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
