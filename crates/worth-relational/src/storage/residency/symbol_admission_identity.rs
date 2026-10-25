//! An immutable region view's identity, without access to its contents.
use super::ResidentRegion;
use std::cmp::Ordering;
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub(crate) struct RelationalRegionSymbolAdmissionIdentity(Weak<ResidentRegion>);

impl RelationalRegionSymbolAdmissionIdentity {
    pub(super) fn new(region: &Arc<ResidentRegion>) -> Self {
        Self(Arc::downgrade(region))
    }
    // Keeping the Weak prevents address reuse for the cache entry's lifetime.
    fn address(&self) -> usize {
        self.0.as_ptr() as usize
    }
}
impl PartialEq for RelationalRegionSymbolAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for RelationalRegionSymbolAdmissionIdentity {}
impl PartialOrd for RelationalRegionSymbolAdmissionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RelationalRegionSymbolAdmissionIdentity {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address().cmp(&other.address())
    }
}
