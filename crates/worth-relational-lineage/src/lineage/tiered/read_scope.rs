//! One read boundary's arena of faulted values and its storage failure latch.
//!
//! Readers of a cold-backed naming structure borrow decoded page values for
//! the boundary's lifetime: each faulted value is placed once in an
//! append-only arena owned by the boundary, so the work and memory a read
//! costs are `O(touched)` and end with the boundary. A fault that fails
//! latches its cause; the boundary refuses its whole answer typed instead of
//! letting a damaged page read as an absent entry.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

use crate::lineage::index::PersistentNameIndexDenial;

const CHUNK: usize = 16;

struct Chunk<T> {
    slots: [OnceLock<T>; CHUNK],
    next: OnceLock<Box<Chunk<T>>>,
}

impl<T> Chunk<T> {
    fn new() -> Self {
        Self {
            slots: std::array::from_fn(|_| OnceLock::new()),
            next: OnceLock::new(),
        }
    }
}

/// An append-only arena: every allocation lives until the arena drops.
pub struct ScopeArena<T> {
    head: Chunk<T>,
    len: AtomicUsize,
}

impl<T> Default for ScopeArena<T> {
    fn default() -> Self {
        Self {
            head: Chunk::new(),
            len: AtomicUsize::new(0),
        }
    }
}

impl<T> ScopeArena<T> {
    /// Places `value` and borrows it for the arena's lifetime.
    ///
    /// Complexity: O(allocations / 16) to reach the tail chunk.
    pub fn alloc(&self, value: T) -> &T {
        let index = self.len.fetch_add(1, Ordering::AcqRel);
        let mut chunk = &self.head;
        for _ in 0..index / CHUNK {
            chunk = chunk.next.get_or_init(|| Box::new(Chunk::new()));
        }
        let slot = &chunk.slots[index % CHUNK];
        let mut value = Some(value);
        slot.get_or_init(|| value.take().expect("one value per fresh slot"))
    }

    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }
}

/// A boundary's faulted values, by type, and its first storage or pruning failure.
#[derive(Default)]
pub struct NamingReadScope {
    failure: Mutex<Option<Cause>>,
    pruned: Mutex<Option<(CommitId, CommitId)>>,
    arena: ScopeArena<Box<dyn std::any::Any + Send + Sync>>,
}

impl std::fmt::Debug for NamingReadScope {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("NamingReadScope")
            .field("faulted", &self.arena.len())
            .field("failure", &self.failure())
            .finish()
    }
}

impl NamingReadScope {
    pub fn new() -> Self {
        Self::default()
    }

    /// Places one faulted value for the scope's lifetime.
    pub fn alloc<T: Send + Sync + 'static>(&self, value: T) -> &T {
        self.arena
            .alloc(Box::new(value))
            .downcast_ref::<T>()
            .expect("the arena returns the value it placed")
    }

    /// Latches the first failure; later ones keep the first cause.
    pub fn fail(&self, cause: Cause) {
        let mut failure = self
            .failure
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        failure.get_or_insert(cause);
    }

    pub fn failure(&self) -> Option<Cause> {
        *self
            .failure
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// Latches the first pruned-basis refusal.
    pub fn fail_pruned(&self, basis: CommitId, retention_floor: CommitId) {
        let mut pruned = self
            .pruned
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        pruned.get_or_insert((basis, retention_floor));
    }

    pub fn pruned(&self) -> Option<(CommitId, CommitId)> {
        *self
            .pruned
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// Settles the boundary: its answer stands only when no fault failed.
    pub fn settle(&self) -> Result<(), Cause> {
        self.failure().map_or(Ok(()), Err)
    }

    /// Settles a name-index read boundary: fails on storage damage first,
    /// then on any pruned-basis access.
    pub fn settle_index(&self) -> Result<(), PersistentNameIndexDenial> {
        if let Some(cause) = self.failure() {
            return Err(PersistentNameIndexDenial::StorageUnavailable(cause));
        }
        if let Some((basis, retention_floor)) = self.pruned() {
            return Err(PersistentNameIndexDenial::BasisPruned {
                basis,
                retention_floor,
            });
        }
        Ok(())
    }

    /// How many values the boundary faulted.
    pub fn faulted(&self) -> usize {
        self.arena.len()
    }
}
