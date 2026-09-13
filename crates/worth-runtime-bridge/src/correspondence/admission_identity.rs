use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug)]
struct BridgeCorrespondenceAdmissionToken {
    _private: u8,
}

/// Opaque identity issued by Bridge for one installed correspondence
/// admission.
///
/// The token is not a correspondence descriptor or a second mapping
/// authority. It contains no Bridge runtime or lease, and construction is
/// private to the Bridge owner crate.
#[derive(Clone)]
pub struct BridgeCorrespondenceAdmissionIdentity(Arc<BridgeCorrespondenceAdmissionToken>);

impl BridgeCorrespondenceAdmissionIdentity {
    pub(crate) fn issue() -> Self {
        Self(Arc::new(BridgeCorrespondenceAdmissionToken { _private: 0 }))
    }
}

impl fmt::Debug for BridgeCorrespondenceAdmissionIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BridgeCorrespondenceAdmissionIdentity(..)")
    }
}

impl PartialEq for BridgeCorrespondenceAdmissionIdentity {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for BridgeCorrespondenceAdmissionIdentity {}

impl Hash for BridgeCorrespondenceAdmissionIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.0) as usize).hash(state);
    }
}

impl PartialOrd for BridgeCorrespondenceAdmissionIdentity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BridgeCorrespondenceAdmissionIdentity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (Arc::as_ptr(&self.0) as usize).cmp(&(Arc::as_ptr(&other.0) as usize))
    }
}
