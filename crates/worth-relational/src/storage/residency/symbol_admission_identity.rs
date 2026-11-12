//! Ephemeral dictionary-coverage keys; no key alone grants an admission.
use super::RelationalRegionResidency;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SymbolClosureLevel {
    Head,
    Owned,
    Full,
    Publication,
}
#[derive(Debug, Clone)]
// Weak opaque allocations keep keys unique after a cell is released without
// retaining any region payload or making dictionary snapshots own a region.
enum SymbolAdmissionSubject {
    Region(super::cell::RegionCellWeak),
    Publication(Weak<()>),
}
#[derive(Debug, Clone)]
pub(crate) struct RelationalRegionSymbolAdmissionIdentity {
    subject: SymbolAdmissionSubject,
    level: SymbolClosureLevel,
}
#[derive(Debug)]
pub(super) struct SymbolAdmissionAliases {
    pub(super) head: RelationalRegionSymbolAdmissionIdentity,
    pub(super) owned: RelationalRegionSymbolAdmissionIdentity,
    pub(super) full: RelationalRegionSymbolAdmissionIdentity,
}
impl RelationalRegionSymbolAdmissionIdentity {
    pub(super) fn publication(token: &Arc<()>) -> Self {
        Self {
            subject: SymbolAdmissionSubject::Publication(Arc::downgrade(token)),
            level: SymbolClosureLevel::Publication,
        }
    }
    fn region(region: &Arc<RelationalRegionResidency>, level: SymbolClosureLevel) -> Self {
        if let Some(aliases) = region.symbol_aliases.get() {
            return match level {
                SymbolClosureLevel::Head => aliases.head.clone(),
                SymbolClosureLevel::Owned => aliases.owned.clone(),
                SymbolClosureLevel::Full => aliases.full.clone(),
                SymbolClosureLevel::Publication => {
                    unreachable!("publication keys have their own owner")
                }
            };
        }
        Self {
            subject: SymbolAdmissionSubject::Region(RelationalRegionResidency::downgrade(region)),
            level,
        }
    }
    pub(super) fn head(region: &Arc<RelationalRegionResidency>) -> Self {
        Self::region(region, SymbolClosureLevel::Head)
    }
    pub(super) fn owned(region: &Arc<RelationalRegionResidency>) -> Self {
        Self::region(region, SymbolClosureLevel::Owned)
    }
    pub(super) fn full(region: &Arc<RelationalRegionResidency>) -> Self {
        Self::region(region, SymbolClosureLevel::Full)
    }
    pub(crate) fn covered_head(&self) -> Option<Self> {
        matches!(
            self.level,
            SymbolClosureLevel::Full | SymbolClosureLevel::Owned
        )
        .then(|| Self {
            subject: self.subject.clone(),
            level: SymbolClosureLevel::Head,
        })
    }
    pub(crate) fn covered_owned(&self) -> Option<Self> {
        matches!(self.level, SymbolClosureLevel::Full).then(|| Self {
            subject: self.subject.clone(),
            level: SymbolClosureLevel::Owned,
        })
    }
    fn key(&self) -> (u8, usize, SymbolClosureLevel) {
        match &self.subject {
            SymbolAdmissionSubject::Region(owner) => (0, owner.as_ptr() as usize, self.level),
            SymbolAdmissionSubject::Publication(owner) => (1, owner.as_ptr() as usize, self.level),
        }
    }
}
impl PartialEq for RelationalRegionSymbolAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.key() == other.key()
    }
}
impl Eq for RelationalRegionSymbolAdmissionIdentity {}
impl Hash for RelationalRegionSymbolAdmissionIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key().hash(state);
    }
}
impl PartialOrd for RelationalRegionSymbolAdmissionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RelationalRegionSymbolAdmissionIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key().cmp(&other.key())
    }
}
