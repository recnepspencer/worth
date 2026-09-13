use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug)]
struct SignalBranchBasisAdmissionToken {
    _private: u8,
}

/// Opaque identity issued by the Signal owner for one admitted basis.
///
/// The token is descriptive admission identity, not a serializable
/// descriptor or currentness proof. It contains no Signal owner, basis, or
/// retention lease, and construction is private to the owner crate.
#[derive(Clone)]
pub struct SignalBranchBasisAdmissionIdentity(Arc<SignalBranchBasisAdmissionToken>);

impl SignalBranchBasisAdmissionIdentity {
    pub(crate) fn issue() -> Self {
        Self(Arc::new(SignalBranchBasisAdmissionToken { _private: 0 }))
    }
}

impl fmt::Debug for SignalBranchBasisAdmissionIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SignalBranchBasisAdmissionIdentity(..)")
    }
}

impl PartialEq for SignalBranchBasisAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for SignalBranchBasisAdmissionIdentity {}

impl Hash for SignalBranchBasisAdmissionIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl PartialOrd for SignalBranchBasisAdmissionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SignalBranchBasisAdmissionIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (Arc::as_ptr(&self.0) as usize).cmp(&(Arc::as_ptr(&other.0) as usize))
    }
}
