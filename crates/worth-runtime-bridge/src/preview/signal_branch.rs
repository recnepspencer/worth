use std::fmt;
use std::sync::Arc;

use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalBranchIdentity};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BridgeSignalBranchIdentity(Arc<SignalBranchIdentity>);

impl BridgeSignalBranchIdentity {
    /// Carry the Signal owner's branch identity, which includes its graph.
    pub fn from_basis(basis: &AdmittedSignalBranchBasis) -> Self {
        Self(Arc::new(basis.observation().branch_id().clone()))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }

    #[cfg(test)]
    pub(crate) fn from_stable_name(label: impl Into<String>) -> Self {
        Self(Arc::new(
            SignalBranchIdentity::new(label).expect("preview fixture branch label is nonempty"),
        ))
    }
}

impl fmt::Debug for BridgeSignalBranchIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeSignalBranchIdentity")
            .field(&"<opaque>")
            .finish()
    }
}
