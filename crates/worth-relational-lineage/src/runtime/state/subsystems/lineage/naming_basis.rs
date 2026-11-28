//! Result and audit projection for the branch-local persistent-name basis.

use std::collections::BTreeMap;

use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;

/// Snapshot of live persistent-name holders, live record lineages, and
/// previously retired persistent-name lineages for one branch (test-only
/// basis-equality helper; live commits borrow via `BranchNamingBasisGuard`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoritativeLineageBasis {
    pub live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    pub retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
}

/// Why a branch's naming basis cannot answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingBasisUnavailable {
    /// A cold naming page it needed is missing or damaged.
    Storage(Cause),
    /// It cannot be derived from the resident state: the reopened checkpoint
    /// recorded it unavailable, or a fork needed a rebuild from history the
    /// cold runtime does not hold.
    Unrecoverable,
}

impl From<&crate::lineage::index::PersistentNameIndexDenial> for NamingBasisUnavailable {
    fn from(denial: &crate::lineage::index::PersistentNameIndexDenial) -> Self {
        match denial {
            crate::lineage::index::PersistentNameIndexDenial::StorageUnavailable(cause) => Self::Storage(*cause),
            _ => Self::Unrecoverable,
        }
    }
}
