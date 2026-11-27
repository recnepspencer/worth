//! Copy-on-write tiered tables over a checkpoint's cold naming pages.
//!
//! Each table holds an optional cold base link and a structural persistent
//! [`CowMap`] delta where `None` is a tombstone over the base. Forking a
//! table shares the persistent root in `O(1)` with zero cloning, point lookups
//! and misses are strictly `O(log N)` regardless of fork/write depth, and
//! mutations copy only the `O(log N)` nodes along the touched path.
use std::fmt;
use std::sync::Arc;

use worth_relational_contracts::internal_port::history::data::coordinates::BranchId;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use crate::lineage::naming_pages::codec::NamingCodec;
use crate::lineage::naming_pages::store::ColdNamingStore;
use crate::lineage::naming_pages::tables::row_key;
use crate::lineage::naming_pages::BranchNamingRoot;

mod cow_map;
mod merged;
mod pinned_store;
mod read_scope;
mod row_ref;

use cow_map::CowMap;
pub use pinned_store::PinnedNamingStore;
pub use read_scope::NamingReadScope;
pub use row_ref::RowRef;

/// Where cold naming faults obtain the currently selected checkpoint's store.
///
/// Implementations must NOT retain a strong reference to a checkpoint
/// directory or durable source past close; they resolve it on demand from the
/// runtime's current selection and return it custody-pinned.
pub trait ColdNamingSource: fmt::Debug + Send + Sync {
    fn store(&self) -> Option<PinnedNamingStore>;
}

/// The route from a runtime branch to its cold base in the selected checkpoint.
///
/// Stores strictly `O(1)` metadata: this branch's own name and the already
/// checkpointed root branch it inherits from (`checkpointed_root`). Uncheckpointed
/// fork chains of arbitrary depth never accumulate an ancestor chain, so point
/// lookups and misses probe at most 2 entries (`O(1)` in fork depth).
#[derive(Debug, Clone)]
pub struct NamingBaseLink {
    source: Arc<dyn ColdNamingSource>,
    branch: BranchId,
    checkpointed_root: BranchId,
}

impl PartialEq for NamingBaseLink {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
            && self.branch == other.branch
            && self.checkpointed_root == other.checkpointed_root
    }
}

impl Eq for NamingBaseLink {}

impl NamingBaseLink {
    pub fn new(source: Arc<dyn ColdNamingSource>, branch: BranchId) -> Self {
        Self {
            source,
            checkpointed_root: branch.clone(),
            branch,
        }
    }

    /// A route for a fork of this branch, collapsing any uncheckpointed
    /// ancestor directly to the already-checkpointed root (`O(1)` in fork depth).
    pub fn forked(&self, child: BranchId) -> Self {
        let checkpointed_root = if self.branch != self.checkpointed_root
            && self
                .source
                .store()
                .is_some_and(|store| store.state().root(&self.branch).is_some())
        {
            self.branch.clone()
        } else {
            self.checkpointed_root.clone()
        };
        Self {
            source: self.source.clone(),
            branch: child,
            checkpointed_root,
        }
    }

    /// Rebinds this link onto `source` and optionally updates its
    /// `checkpointed_root` (used on detached runtime forks and on checkpoint
    /// acknowledgement for uncheckpointed/dirty descendants).
    pub fn rebind_source(
        &self,
        source: Arc<dyn ColdNamingSource>,
        checkpointed_root: Option<&BranchId>,
    ) -> Self {
        Self {
            source,
            branch: self.branch.clone(),
            checkpointed_root: checkpointed_root
                .cloned()
                .unwrap_or_else(|| self.checkpointed_root.clone()),
        }
    }

    /// Resolves this branch's root in `store`, falling back directly to the
    /// resolved `checkpointed_root` (`O(1)` probes regardless of fork depth).
    pub fn root_in<'a>(&self, store: &'a ColdNamingStore) -> Option<&'a BranchNamingRoot> {
        let state = store.state();
        if self.branch == self.checkpointed_root {
            state.root(&self.checkpointed_root)
        } else {
            state
                .root(&self.branch)
                .or_else(|| state.root(&self.checkpointed_root))
        }
    }

    /// The selected store (custody-pinned) and the first root along the fork chain.
    ///
    /// Complexity: O(chain * log branches).
    pub fn resolve(&self) -> Result<(PinnedNamingStore, Option<BranchNamingRoot>), Cause> {
        let store = self.source.store().ok_or(Cause::SourceUnavailable)?;
        let root = self.root_in(&store).cloned();
        Ok((store, root))
    }

    /// The base value of row `key`, faulting at most one page (plus chunks).
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Cause> {
        let store = self.source.store().ok_or(Cause::SourceUnavailable)?;
        match self.root_in(&store) {
            Some(root) => store.get(&root.pages, key),
            None => Ok(None),
        }
    }

    /// Visits the base rows starting with `prefix`, ascending.
    pub fn scan(
        &self,
        prefix: &[u8],
        visit: impl FnMut(&[u8], &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        let store = self.source.store().ok_or(Cause::SourceUnavailable)?;
        self.scan_in(&store, prefix, visit)
    }

    /// Visits the base rows starting with `prefix` inside a specific pinned `store`.
    pub fn scan_in(
        &self,
        store: &ColdNamingStore,
        prefix: &[u8],
        visit: impl FnMut(&[u8], &[u8]) -> Result<bool, Cause>,
    ) -> Result<(), Cause> {
        match self.root_in(store) {
            Some(root) => store.scan(&root.pages, prefix, visit),
            None => Ok(()),
        }
    }

    pub fn source(&self) -> &Arc<dyn ColdNamingSource> {
        &self.source
    }
}

/// One naming table: an optional cold base and a structural copy-on-write
/// persistent delta (`CowMap`) in which `None` is a removal over the base.
///
/// Forking shares the persistent tree root in `O(1)`; point lookups and misses
/// stay strictly `O(log N)` across arbitrarily many fork/write cycles.
#[derive(Debug)]
pub struct TieredMap<K, V> {
    table: u8,
    base: Option<NamingBaseLink>,
    delta: CowMap<K, Option<V>>,
}

impl<K: Ord + PartialEq, V: PartialEq> PartialEq for TieredMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.table == other.table && self.base == other.base && self.delta == other.delta
    }
}

impl<K: Ord + Eq, V: Eq> Eq for TieredMap<K, V> {}

impl<K, V> TieredMap<K, V>
where
    K: NamingCodec + Ord + Clone,
    V: NamingCodec,
{
    pub fn resident(table: u8) -> Self {
        Self::over(table, None)
    }

    pub fn over(table: u8, base: Option<NamingBaseLink>) -> Self {
        Self {
            table,
            base,
            delta: CowMap::new(),
        }
    }

    /// This table for a fork: shares the persistent delta root in `O(1)`.
    pub fn fork(&self, base: Option<NamingBaseLink>) -> Self {
        Self {
            table: self.table,
            base,
            delta: self.delta.clone(),
        }
    }

    pub fn rebind_clean(&mut self, base: NamingBaseLink) {
        self.base = Some(base);
        self.delta.clear();
    }

    pub fn rebind_source(
        &mut self,
        source: &Arc<dyn ColdNamingSource>,
        checkpointed_root: Option<&BranchId>,
    ) {
        if let Some(base) = &mut self.base {
            *base = base.rebind_source(Arc::clone(source), checkpointed_root);
        }
    }

    pub fn base(&self) -> Option<&NamingBaseLink> {
        self.base.as_ref()
    }

    pub fn has_delta(&self) -> bool {
        !self.delta.is_empty()
    }

    pub fn row_key(&self, key: &K) -> Vec<u8> {
        row_key(self.table, key)
    }

    pub const fn table(&self) -> u8 {
        self.table
    }

    fn base_value(&self, key: &K) -> Result<Option<V>, Cause> {
        let Some(base) = &self.base else {
            return Ok(None);
        };
        base.get(&self.row_key(key))?
            .map(|bytes| V::from_bytes(&bytes))
            .transpose()
    }

    /// The value of `key`, borrowed for `scope`; a failed fault latches in
    /// the scope and reads as absent until the scope settles.
    pub fn get_in<'s>(&'s self, key: &K, scope: &'s NamingReadScope) -> Option<&'s V>
    where
        V: Send + Sync + 'static,
    {
        if let Some(slot) = self.delta.get(key) {
            return slot.as_ref();
        }
        match self.base_value(key) {
            Ok(value) => value.map(|value| scope.alloc(value)),
            Err(cause) => {
                scope.fail(cause);
                None
            }
        }
    }

    /// The value of `key` for a writer, faulting the base on a delta miss.
    pub fn get(&self, key: &K) -> Result<Option<RowRef<'_, V>>, Cause> {
        match self.delta.get(key) {
            Some(slot) => Ok(slot.as_ref().map(RowRef::Borrowed)),
            None => Ok(self.base_value(key)?.map(RowRef::Owned)),
        }
    }

    pub fn contains_key(&self, key: &K) -> Result<bool, Cause> {
        Ok(self.get(key)?.is_some())
    }

    /// Mutable access, faulting a base row or COW-duplicating a shared slot.
    pub fn get_mut_with(
        &mut self,
        key: &K,
        duplicate: impl FnOnce(&V) -> V,
    ) -> Result<Option<&mut V>, Cause> {
        if !self.delta.contains_key(key) {
            let Some(value) = self.base_value(key)? else {
                return Ok(None);
            };
            self.delta.insert(key.clone(), Some(value));
        }
        Ok(self
            .delta
            .get_mut_with(key, |slot| slot.as_ref().map(duplicate))
            .and_then(Option::as_mut))
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.delta.insert(key, Some(value));
    }

    pub fn remove_blind(&mut self, key: &K) {
        if self.base.is_some() {
            self.delta.insert(key.clone(), None);
        } else {
            self.delta.remove(key);
        }
    }

    pub fn delta_arc_and_shared(&self, key: &K) -> Option<(&Arc<Option<V>>, bool)> {
        self.delta.get_arc_and_shared(key)
    }

    pub fn remove_delta(&mut self, key: &K) {
        self.delta.remove(key);
    }

    pub fn restore_delta_arc(&mut self, key: K, arc: Arc<Option<V>>) {
        self.delta.insert_arc(key, arc);
    }
}

impl<K, V> TieredMap<K, V>
where
    K: NamingCodec + Ord + Clone,
    V: NamingCodec + Clone,
{
    pub fn get_mut(&mut self, key: &K) -> Result<Option<&mut V>, Cause> {
        self.get_mut_with(key, Clone::clone)
    }

    pub fn get_or_insert_with(
        &mut self,
        key: K,
        default: impl FnOnce() -> V,
    ) -> Result<&mut V, Cause> {
        if self.get(&key)?.is_none() {
            self.insert(key.clone(), default());
        }
        self.get_mut(&key)?.ok_or(Cause::Damaged)
    }

    pub fn remove(&mut self, key: &K) -> Result<Option<V>, Cause> {
        let prior = self.get(key)?.map(RowRef::into_owned);
        self.remove_blind(key);
        Ok(prior)
    }

    pub fn slot(&self, key: &K) -> Option<Option<V>> {
        self.delta.get(key).cloned()
    }

    pub fn restore_slot(&mut self, key: K, slot: Option<Option<V>>) {
        match slot {
            Some(value) => self.delta.insert(key, value),
            None => self.delta.remove(&key),
        }
    }
}
