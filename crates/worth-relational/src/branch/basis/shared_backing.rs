//! Shares of the exact admitted basis and exclusive retirement of its state.
use super::AdmittedRelationalBranchBasisInner;
use std::sync::{Arc, Weak};

pub(crate) type SharedRelationalBranchBasis = Arc<dyn BasisBacking>;
pub(crate) type WeakRelationalBranchBasis = Weak<dyn BasisBacking>;

// Prove thread-safe sharing at the Native basis owner. Readers and registry
// bindings carry the original allocation without inheriting its storage shape.
pub(crate) trait BasisBacking: std::fmt::Debug + Send + Sync {
    fn state(&self) -> &AdmittedRelationalBranchBasisInner;
    fn release_share(self: Arc<Self>) -> Option<AdmittedRelationalBranchBasisInner>;
}

impl BasisBacking for AdmittedRelationalBranchBasisInner {
    fn state(&self) -> &Self {
        self
    }

    fn release_share(self: Arc<Self>) -> Option<Self> {
        Arc::into_inner(self)
    }
}

impl std::ops::Deref for dyn BasisBacking {
    type Target = AdmittedRelationalBranchBasisInner;

    fn deref(&self) -> &Self::Target {
        self.state()
    }
}
