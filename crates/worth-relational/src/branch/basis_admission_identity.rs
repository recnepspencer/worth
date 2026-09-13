use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug)]
struct RelationalBranchBasisAdmissionToken {
    _private: u8,
}

/// Opaque identity issued by the Relational owner for one admitted basis.
///
/// The token is an admission identity, not a descriptor digest. It contains
/// no basis, owner runtime, or retention lease, so retaining this identity
/// does not keep the component owner or its history alive. Construction is
/// private to the owner crate.
#[derive(Clone)]
pub struct RelationalBranchBasisAdmissionIdentity(Arc<RelationalBranchBasisAdmissionToken>);

impl RelationalBranchBasisAdmissionIdentity {
    pub(crate) fn issue() -> Self {
        Self(Arc::new(RelationalBranchBasisAdmissionToken {
            _private: 0,
        }))
    }
}

impl fmt::Debug for RelationalBranchBasisAdmissionIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RelationalBranchBasisAdmissionIdentity(..)")
    }
}

impl PartialEq for RelationalBranchBasisAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for RelationalBranchBasisAdmissionIdentity {}

impl Hash for RelationalBranchBasisAdmissionIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl PartialOrd for RelationalBranchBasisAdmissionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RelationalBranchBasisAdmissionIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (Arc::as_ptr(&self.0) as usize).cmp(&(Arc::as_ptr(&other.0) as usize))
    }
}
