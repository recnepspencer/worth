//! Type-erased strong and weak handles to `AdmittedRelationalBranchBasisInner`.
//!
//! Uses unsized coercion on the existing `Arc<AdmittedRelationalBranchBasisInner>`
//! / `Weak<AdmittedRelationalBranchBasisInner>` allocation so rustdoc auto-trait
//! synthesis does not traverse deep branch roots, canonical envelopes, and
//! persistent derived-index maps through observation and basis handles.

use std::any::Any;
use std::fmt;
use std::ops::Deref;
use std::sync::{Arc, Weak};

use super::AdmittedRelationalBranchBasisInner;

trait AdmittedRelationalBranchBasisBacking: Send + Sync {
    fn as_inner(&self) -> &AdmittedRelationalBranchBasisInner;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl AdmittedRelationalBranchBasisBacking for AdmittedRelationalBranchBasisInner {
    fn as_inner(&self) -> &AdmittedRelationalBranchBasisInner {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Clone)]
pub struct SharedAdmittedRelationalBranchBasisInner(Arc<dyn AdmittedRelationalBranchBasisBacking>);

#[derive(Clone)]
pub struct WeakAdmittedRelationalBranchBasisInner(Weak<dyn AdmittedRelationalBranchBasisBacking>);

impl SharedAdmittedRelationalBranchBasisInner {
    pub fn new(inner: AdmittedRelationalBranchBasisInner) -> Self {
        Self(Arc::new(inner))
    }

    pub fn from_arc(inner: Arc<AdmittedRelationalBranchBasisInner>) -> Self {
        Self(inner)
    }

    pub fn into_arc(self) -> Arc<AdmittedRelationalBranchBasisInner> {
        self.0
            .into_any()
            .downcast::<AdmittedRelationalBranchBasisInner>()
            .expect("SharedAdmittedRelationalBranchBasisInner always wraps AdmittedRelationalBranchBasisInner")
    }

    pub fn downgrade(&self) -> WeakAdmittedRelationalBranchBasisInner {
        WeakAdmittedRelationalBranchBasisInner(Arc::downgrade(&self.0))
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(&**self, &**other)
    }
}

impl From<Arc<AdmittedRelationalBranchBasisInner>> for SharedAdmittedRelationalBranchBasisInner {
    fn from(inner: Arc<AdmittedRelationalBranchBasisInner>) -> Self {
        Self::from_arc(inner)
    }
}

impl Deref for SharedAdmittedRelationalBranchBasisInner {
    type Target = AdmittedRelationalBranchBasisInner;

    fn deref(&self) -> &Self::Target {
        AdmittedRelationalBranchBasisBacking::as_inner(&*self.0)
    }
}

impl fmt::Debug for SharedAdmittedRelationalBranchBasisInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}

impl WeakAdmittedRelationalBranchBasisInner {
    pub fn upgrade(&self) -> Option<SharedAdmittedRelationalBranchBasisInner> {
        self.0
            .upgrade()
            .map(SharedAdmittedRelationalBranchBasisInner)
    }

    pub fn strong_count(&self) -> usize {
        self.0.strong_count()
    }
}

impl fmt::Debug for WeakAdmittedRelationalBranchBasisInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakAdmittedRelationalBranchBasisInner")
            .finish_non_exhaustive()
    }
}
