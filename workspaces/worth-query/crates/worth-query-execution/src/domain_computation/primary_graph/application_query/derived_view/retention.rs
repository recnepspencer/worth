use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Weak};

use worth_query_declaration::facade::application_query::ApplicationDerivedViewLimits;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::history::BranchId;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
};

use super::dependency::ViewDependency;
use super::registry::{ManagedDerivedViewRegistry, RegisteredDerivedView};

/// A projected value declares its retained heap footprint. Query charges
/// entry and source-provenance overhead in addition to this payload.
pub trait WorthQueryManagedDerivedValue: Send + Sync + 'static {
    fn retained_bytes(&self) -> usize;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryManagedDerivedViewDenial {
    InvalidLimits,
    ViewCapacityExceeded,
    EntryCapacityExceeded,
    RetainedBytesExceeded,
    ForeignApplication,
    ForeignInstallation,
    ForeignQuery,
    AuthorizationRequired,
    ForeignBranch,
    StaleSource,
    IncompleteDependencies,
    ColdReconstructionRequired,
    Disposed,
}

struct RetainedEntry<Value> {
    value: Arc<Value>,
    dependencies: BTreeSet<ViewDependency>,
}

struct RetainedView<Key, Value> {
    current_commit: CompositeCommitIdentity,
    membership: BTreeSet<ViewDependency>,
    entries: BTreeMap<Key, RetainedEntry<Value>>,
    cold: bool,
    disposed: bool,
}

pub(super) struct ManagedDerivedViewState<Key, Value> {
    pub(super) runtime_authority: u64,
    pub(super) binding: ApplicationSchemaBindingIdentity,
    pub(super) query: WorthQueryInstalledApplicationQueryIdentity,
    pub(super) branch: BranchId,
    pub(super) product_branch: ProductBranchIdentity,
    pub(super) incarnation: ProductBranchIncarnation,
    pub(super) limits: ApplicationDerivedViewLimits,
    retained: Mutex<RetainedView<Key, Value>>,
}

impl<Key, Value> ManagedDerivedViewState<Key, Value>
where
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
{
    pub(super) fn new(
        runtime_authority: u64,
        binding: ApplicationSchemaBindingIdentity,
        query: WorthQueryInstalledApplicationQueryIdentity,
        branch: BranchId,
        product_branch: ProductBranchIdentity,
        incarnation: ProductBranchIncarnation,
        commit: CompositeCommitIdentity,
        limits: ApplicationDerivedViewLimits,
    ) -> Self {
        Self {
            runtime_authority,
            binding,
            query,
            branch,
            product_branch,
            incarnation,
            limits,
            retained: Mutex::new(RetainedView {
                current_commit: commit,
                membership: BTreeSet::new(),
                entries: BTreeMap::new(),
                cold: true,
                disposed: false,
            }),
        }
    }

    pub(super) fn reconstruct(
        &self,
        entries: Vec<(Key, Value, BTreeSet<ViewDependency>)>,
        membership: BTreeSet<ViewDependency>,
        observed_commit: &CompositeCommitIdentity,
    ) -> Result<(), WorthQueryManagedDerivedViewDenial> {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if retained.disposed {
            return Err(WorthQueryManagedDerivedViewDenial::Disposed);
        }
        if &retained.current_commit != observed_commit {
            return Err(WorthQueryManagedDerivedViewDenial::StaleSource);
        }
        if membership.is_empty()
            || entries
                .iter()
                .any(|(_, _, dependencies)| dependencies.is_empty())
        {
            return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
        }
        if entries.len() > self.limits.maximum_entries() {
            return Err(WorthQueryManagedDerivedViewDenial::EntryCapacityExceeded);
        }
        let mut replacement = BTreeMap::new();
        let mut bytes = membership
            .len()
            .saturating_mul(std::mem::size_of::<ViewDependency>());
        for (key, value, dependencies) in entries {
            bytes = bytes
                .saturating_add(value.retained_bytes())
                .saturating_add(std::mem::size_of::<(Key, RetainedEntry<Value>)>())
                .saturating_add(
                    dependencies
                        .len()
                        .saturating_mul(std::mem::size_of::<ViewDependency>()),
                );
            if bytes > self.limits.maximum_retained_bytes() {
                return Err(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded);
            }
            if replacement
                .insert(
                    key,
                    RetainedEntry {
                        value: Arc::new(value),
                        dependencies,
                    },
                )
                .is_some()
            {
                return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
            }
        }
        retained.entries = replacement;
        retained.membership = membership;
        retained.cold = false;
        Ok(())
    }

    pub(super) fn current_commit(&self) -> Option<CompositeCommitIdentity> {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        (!retained.disposed).then(|| retained.current_commit.clone())
    }

    pub(super) fn get(
        &self,
        key: &Key,
        commit: &CompositeCommitIdentity,
    ) -> Result<Option<Arc<Value>>, WorthQueryManagedDerivedViewDenial> {
        let retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        verify_observation(&retained, commit)?;
        let entry = retained.entries.get(key);
        if entry.is_some_and(|entry| entry.dependencies.is_empty()) {
            return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
        }
        Ok(entry.map(|entry| Arc::clone(&entry.value)))
    }

    pub(super) fn discard(&self) {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        retained.entries.clear();
        retained.membership.clear();
        retained.cold = true;
    }

    pub(super) fn rebase_cold(&self, commit: CompositeCommitIdentity) {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        retained.entries.clear();
        retained.membership.clear();
        retained.cold = true;
        retained.current_commit = commit;
    }
}

fn verify_observation<Key, Value>(
    retained: &RetainedView<Key, Value>,
    commit: &CompositeCommitIdentity,
) -> Result<(), WorthQueryManagedDerivedViewDenial> {
    if retained.disposed {
        return Err(WorthQueryManagedDerivedViewDenial::Disposed);
    }
    if retained.cold {
        return Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired);
    }
    if &retained.current_commit != commit {
        return Err(WorthQueryManagedDerivedViewDenial::StaleSource);
    }
    if retained.membership.is_empty() {
        return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
    }
    Ok(())
}

impl<Key, Value> RegisteredDerivedView for ManagedDerivedViewState<Key, Value>
where
    Key: Clone + Ord + Send + Sync + 'static,
    Value: WorthQueryManagedDerivedValue,
{
    fn dispose(&self) {
        self.discard();
        self.retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .disposed = true;
    }
}

pub struct WorthQueryManagedDerivedView<Query, Key, Value> {
    pub(super) state: Arc<ManagedDerivedViewState<Key, Value>>,
    pub(super) id: u64,
    pub(super) registry: Weak<ManagedDerivedViewRegistry>,
    pub(super) _query: PhantomData<fn() -> Query>,
}

impl<Query, Key, Value> Drop for WorthQueryManagedDerivedView<Query, Key, Value> {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.unregister(self.id);
        }
        let mut retained = self
            .state
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        retained.entries.clear();
        retained.membership.clear();
        retained.disposed = true;
    }
}

/// A point-in-time view observation. It never asserts future currentness.
pub struct WorthQueryManagedDerivedViewSnapshot<Key, Value> {
    pub(super) state: Arc<ManagedDerivedViewState<Key, Value>>,
    pub(super) commit: CompositeCommitIdentity,
    pub(super) _entry: PhantomData<fn() -> (Key, Value)>,
}

impl<Key, Value> WorthQueryManagedDerivedViewSnapshot<Key, Value>
where
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
{
    pub fn get(&self, key: &Key) -> Result<Option<Arc<Value>>, WorthQueryManagedDerivedViewDenial> {
        self.state.get(key, &self.commit)
    }
}

impl<Query, Key, Value> WorthQueryManagedDerivedView<Query, Key, Value>
where
    Key: Clone + Ord,
    Value: WorthQueryManagedDerivedValue,
{
    pub fn discard(&self) {
        self.state.discard();
    }
}
