use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};

use crate::history::data::BranchId;

/// A read-only inspection locator for a branch-local coordination cell.
///
/// The branch identity is the owner-issued allocation key.  There is no
/// process-global counter and no raw integer that can be mistaken for a
/// portable authority token.  Runtime identity keeps equal branch names in
/// cloned runtimes distinct.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationalBranchCoordinationCellId {
    runtime_instance_id: u64,
    branch_id: String,
}

impl RelationalBranchCoordinationCellId {
    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub fn branch_id(&self) -> &str {
        &self.branch_id
    }
}

#[derive(Debug)]
pub(crate) struct RelationalBranchCoordinationCell {
    id: RelationalBranchCoordinationCellId,
    contacts: AtomicU64,
    waits: AtomicU64,
    gate: Mutex<()>,
}

impl RelationalBranchCoordinationCell {
    pub(crate) fn fresh(runtime_instance_id: u64, branch_id: &BranchId) -> Arc<Self> {
        Arc::new(Self {
            id: RelationalBranchCoordinationCellId {
                runtime_instance_id,
                branch_id: branch_id.0.clone(),
            },
            contacts: AtomicU64::new(0),
            waits: AtomicU64::new(0),
            gate: Mutex::new(()),
        })
    }

    pub(crate) fn id(&self) -> RelationalBranchCoordinationCellId {
        self.id.clone()
    }

    pub(crate) fn contact_count(&self) -> u64 {
        self.contacts.load(Ordering::Relaxed)
    }

    pub(crate) fn wait_count(&self) -> u64 {
        self.waits.load(Ordering::Relaxed)
    }

    /// Enter the bounded branch-local publication section. Contention is
    /// recorded only on this exact branch; no runtime-global gate is touched.
    pub(crate) fn enter(&self) -> RelationalBranchCoordinationGuard<'_> {
        self.contacts.fetch_add(1, Ordering::Relaxed);
        match self.gate.try_lock() {
            Ok(guard) => RelationalBranchCoordinationGuard { guard },
            Err(TryLockError::WouldBlock) => {
                self.waits.fetch_add(1, Ordering::Relaxed);
                RelationalBranchCoordinationGuard {
                    guard: self
                        .gate
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner()),
                }
            }
            Err(TryLockError::Poisoned(poisoned)) => RelationalBranchCoordinationGuard {
                guard: poisoned.into_inner(),
            },
        }
    }
}

pub(crate) struct RelationalBranchCoordinationGuard<'cell> {
    guard: MutexGuard<'cell, ()>,
}

impl Drop for RelationalBranchCoordinationGuard<'_> {
    fn drop(&mut self) {
        let _ = &self.guard;
    }
}
