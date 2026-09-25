use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, Weak};

use worth_query_declaration::facade::application_query::ApplicationDerivedViewLimits;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
};

use super::dependency::ViewDependency;
use super::publication::DependencyIndex;
use super::registry::ManagedDerivedViewRegistry;

mod member_token;
mod publication;
mod reconcile;
mod refresh;

pub use member_token::WorthQueryManagedDerivedMemberToken;
pub(in crate::domain_computation::primary_graph::application_query::derived_view) use member_token::RetainedMemberToken;
use member_token::token_charge;

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
    MembershipReconciliationRequired,
    EntryRefreshRequired,
    QueryExecutionDenied,
    ViewRevisionExhausted,
    Disposed,
}

struct RetainedEntry<Value> {
    value: Arc<Value>,
    dependencies: BTreeSet<ViewDependency>,
}

struct RetainedView<Key, Value> {
    current_commit: CompositeCommitIdentity,
    membership_key: Option<Key>,
    member_tokens: Option<BTreeMap<EntityId, RetainedMemberToken>>,
    membership: BTreeSet<ViewDependency>,
    entries: BTreeMap<Key, RetainedEntry<Value>>,
    index: DependencyIndex<Key>,
    dirty: BTreeSet<Key>,
    charged_bytes: usize,
    revision: u64,
    revision_exhausted: bool,
    cold: bool,
    membership_dirty: bool,
    disposed: bool,
}

pub(super) struct ManagedDerivedViewState<Key, Value> {
    pub(super) runtime_authority: u64,
    pub(super) binding: ApplicationSchemaBindingIdentity,
    pub(super) query: WorthQueryInstalledApplicationQueryIdentity,
    pub(super) entry_query: Option<WorthQueryInstalledApplicationQueryIdentity>,
    pub(super) secondary_entry_query: Option<WorthQueryInstalledApplicationQueryIdentity>,
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
        entry_query: Option<WorthQueryInstalledApplicationQueryIdentity>,
        secondary_entry_query: Option<WorthQueryInstalledApplicationQueryIdentity>,
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
            entry_query,
            secondary_entry_query,
            branch,
            product_branch,
            incarnation,
            limits,
            retained: Mutex::new(RetainedView {
                current_commit: commit,
                membership_key: None,
                member_tokens: None,
                membership: BTreeSet::new(),
                entries: BTreeMap::new(),
                index: DependencyIndex::default(),
                dirty: BTreeSet::new(),
                charged_bytes: 0,
                revision: 0,
                revision_exhausted: false,
                cold: true,
                membership_dirty: false,
                disposed: false,
            }),
        }
    }

    pub(super) fn reconstruct(
        &self,
        membership_key: Key,
        member_tokens: Option<BTreeMap<EntityId, RetainedMemberToken>>,
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
        if retained.revision_exhausted {
            return Err(WorthQueryManagedDerivedViewDenial::ViewRevisionExhausted);
        }
        if retained.revision == u64::MAX {
            retained.revision_exhausted = true;
            retained.cold = true;
            return Err(WorthQueryManagedDerivedViewDenial::ViewRevisionExhausted);
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
        let mut bytes = std::mem::size_of::<Key>().saturating_add(
            membership
                .iter()
                .map(ViewDependency::retained_bytes)
                .fold(0usize, usize::saturating_add),
        );
        for (key, value, dependencies) in entries {
            bytes = bytes
                .saturating_add(value.retained_bytes())
                .saturating_add(std::mem::size_of::<(Key, RetainedEntry<Value>)>())
                .saturating_add(4 * std::mem::size_of::<usize>())
                .saturating_add(
                    dependencies
                        .iter()
                        .map(ViewDependency::retained_bytes)
                        .fold(0usize, usize::saturating_add),
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
        let index = DependencyIndex::build(
            &membership,
            replacement
                .iter()
                .map(|(key, entry)| (key, &entry.dependencies)),
        );
        let maximum_dirty_bytes = replacement
            .len()
            .saturating_mul(std::mem::size_of::<Key>() + 4 * std::mem::size_of::<usize>());
        let index_bound = membership
            .iter()
            .chain(
                replacement
                    .values()
                    .flat_map(|entry| entry.dependencies.iter()),
            )
            .fold(0usize, |bytes, dependency| {
                bytes.saturating_add(DependencyIndex::<Key>::entry_insertion_bound(dependency))
            });
        bytes = bytes
            .saturating_add(index_bound)
            .saturating_add(maximum_dirty_bytes)
            .saturating_add(member_tokens.as_ref().map_or(0, token_charge));
        if bytes > self.limits.maximum_retained_bytes() {
            return Err(WorthQueryManagedDerivedViewDenial::RetainedBytesExceeded);
        }
        retained.entries = replacement;
        retained.membership_key = Some(membership_key);
        retained.member_tokens = member_tokens;
        retained.membership = membership;
        retained.index = index;
        retained.dirty.clear();
        retained.charged_bytes = bytes;
        retained.advance_revision();
        retained.cold = false;
        retained.membership_dirty = false;
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
        if retained.dirty.contains(key) {
            return Err(WorthQueryManagedDerivedViewDenial::EntryRefreshRequired);
        }
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
        retained.membership_key = None;
        retained.member_tokens = None;
        retained.membership.clear();
        retained.index = DependencyIndex::default();
        retained.dirty.clear();
        retained.charged_bytes = 0;
        retained.advance_revision();
        retained.cold = true;
        retained.membership_dirty = false;
    }

    pub(super) fn rebase_cold(&self, commit: CompositeCommitIdentity) {
        let mut retained = self
            .retained
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        retained.entries.clear();
        retained.membership_key = None;
        retained.member_tokens = None;
        retained.membership.clear();
        retained.index = DependencyIndex::default();
        retained.dirty.clear();
        retained.charged_bytes = 0;
        retained.advance_revision();
        retained.cold = true;
        retained.membership_dirty = false;
        retained.current_commit = commit;
    }
}

impl<Key, Value> RetainedView<Key, Value> {
    fn advance_revision(&mut self) {
        match self.revision.checked_add(1) {
            Some(next) => self.revision = next,
            None => {
                self.revision_exhausted = true;
                self.cold = true;
            }
        }
    }
}

fn verify_observation<Key, Value>(
    retained: &RetainedView<Key, Value>,
    commit: &CompositeCommitIdentity,
) -> Result<(), WorthQueryManagedDerivedViewDenial> {
    if retained.disposed {
        return Err(WorthQueryManagedDerivedViewDenial::Disposed);
    }
    if retained.revision_exhausted {
        return Err(WorthQueryManagedDerivedViewDenial::ViewRevisionExhausted);
    }
    if retained.cold {
        return Err(WorthQueryManagedDerivedViewDenial::ColdReconstructionRequired);
    }
    if retained.membership_dirty {
        return Err(WorthQueryManagedDerivedViewDenial::MembershipReconciliationRequired);
    }
    if &retained.current_commit != commit {
        return Err(WorthQueryManagedDerivedViewDenial::StaleSource);
    }
    if retained.membership.is_empty() {
        return Err(WorthQueryManagedDerivedViewDenial::IncompleteDependencies);
    }
    Ok(())
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
        retained.member_tokens = None;
        retained.membership.clear();
        retained.index = DependencyIndex::default();
        retained.dirty.clear();
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
