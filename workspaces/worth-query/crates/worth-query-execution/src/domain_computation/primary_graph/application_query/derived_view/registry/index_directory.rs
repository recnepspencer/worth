//! One indexed owner, with original grants and allocation-free exact-id retirement.
use super::*;
use crate::domain_computation::primary_graph::application_query::managed_index::retention::ManagedIndexRegistrationKey as Key;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

const BTREE_MAXIMUM_NODE_COMPARISONS: u64 = 11;

#[derive(Default)]
pub(super) struct ManagedIndexDirectory {
    routes: BTreeMap<Key, u64>,
    owned: BTreeMap<u64, RegisteredIndex>,
}
pub(super) struct RegisteredIndex {
    pub(super) key: Key,
    pub(super) state: Arc<dyn Any + Send + Sync>,
    cleanup: Arc<dyn RegisteredManagedIndex>,
    _memory: Option<ExecutionMemoryReservation>,
}
pub(super) struct RetiredIndex {
    // Both real backings close before the last original registration grant.
    _empty_routes: Option<BTreeMap<Key, u64>>,
    _empty_owned: Option<BTreeMap<u64, RegisteredIndex>>,
    _entry: Option<RegisteredIndex>,
}
impl RegisteredIndex {
    pub(super) fn prepare(
        execution: &ExecutionRequest<'_, '_>,
        key: Key,
        state: Arc<dyn Any + Send + Sync>,
        cleanup: Arc<dyn RegisteredManagedIndex>,
    ) -> Result<Self, Cause> {
        let route_node = 11 * size_of::<(Key, u64)>() + 14 * size_of::<usize>();
        let owned_node = 11 * size_of::<(u64, Self)>() + 14 * size_of::<usize>();
        let bytes = route_node
            .checked_add(owned_node)
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(Self {
            key,
            state,
            cleanup,
            _memory: memory,
        })
    }
}
impl ManagedIndexDirectory {
    pub(super) fn len(&self) -> usize {
        self.owned.len()
    }

    /// Pre-admit both fixed-width tree routes on the consuming original request.
    pub(super) fn lookup_work(&self) -> Result<u64, Cause> {
        let count = self
            .len()
            .checked_add(1)
            .ok_or(Cause::WorkCounterOverflow)?;
        let levels = u64::from(usize::BITS - count.leading_zeros());
        levels
            .checked_mul((size_of::<Key>() + size_of::<u64>() + 3) as u64)
            .and_then(|work| work.checked_mul(BTREE_MAXIMUM_NODE_COMPARISONS))
            .ok_or(Cause::WorkCounterOverflow)
    }
    pub(super) fn get(&self, key: &Key) -> Option<(u64, &RegisteredIndex)> {
        let id = *self.routes.get(key)?;
        self.owned.get(&id).map(|entry| (id, entry))
    }
    pub(super) fn insert(&mut self, id: u64, entry: RegisteredIndex) {
        // The issuer already proved the route absent and the id fresh.
        debug_assert_eq!(self.routes.len(), self.owned.len());
        self.routes.insert(entry.key, id);
        self.owned.insert(id, entry);
    }
    pub(super) fn remove(&mut self, id: u64) -> Option<RetiredIndex> {
        let entry = self.owned.remove(&id)?;
        self.routes.remove(&entry.key);
        let empty = self.owned.is_empty();
        Some(RetiredIndex {
            _empty_routes: empty.then(|| std::mem::take(&mut self.routes)),
            _empty_owned: empty.then(|| std::mem::take(&mut self.owned)),
            _entry: Some(entry),
        })
    }
    /// All actual map backing closes while its original grants are still live.
    /// Capacity already bounds this stack scratch; no cleanup heap is allocated.
    pub(super) fn dispose(self) {
        let Self { routes, owned } = self;
        let mut retired: [Option<RegisteredIndex>; MAXIMUM_REGISTERED_VIEWS] =
            std::array::from_fn(|_| None);
        for (position, (_, entry)) in owned.into_iter().enumerate() {
            retired[position] = Some(entry);
        }
        drop(routes);
        for entry in retired.into_iter().flatten() {
            entry.cleanup.dispose();
        }
    }
}
