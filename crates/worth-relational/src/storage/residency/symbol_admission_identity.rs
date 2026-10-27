//! A logical immutable cell version and its requested dictionary closure.
use super::RelationalRegionResidency;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Weak};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SymbolClosureLevel {
    Head,
    Full,
}

#[derive(Debug)]
pub(crate) struct RelationalRegionSymbolAdmissionIdentity {
    region: Weak<RelationalRegionResidency>,
    level: SymbolClosureLevel,
}

impl RelationalRegionSymbolAdmissionIdentity {
    pub(super) fn head(region: &Arc<RelationalRegionResidency>) -> Self {
        Self {
            region: Arc::downgrade(region),
            level: SymbolClosureLevel::Head,
        }
    }
    pub(super) fn full(region: &Arc<RelationalRegionResidency>) -> Self {
        Self {
            region: Arc::downgrade(region),
            level: SymbolClosureLevel::Full,
        }
    }
    pub(crate) fn covered_head(&self) -> Option<Self> {
        matches!(self.level, SymbolClosureLevel::Full).then(|| Self {
            region: self.region.clone(),
            level: SymbolClosureLevel::Head,
        })
    }
}
impl PartialEq for RelationalRegionSymbolAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.level == other.level && Weak::ptr_eq(&self.region, &other.region)
    }
}
impl Eq for RelationalRegionSymbolAdmissionIdentity {}
impl Hash for RelationalRegionSymbolAdmissionIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // The Weak keeps this allocation's address from being reused.
        (self.region.as_ptr() as usize).hash(state);
        self.level.hash(state);
    }
}
