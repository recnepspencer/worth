//! Type-erased shared handle to an immutable `DerivedIndexGeneration`.
//!
//! Uses unsized coercion on the existing `Arc<DerivedIndexGeneration>`
//! allocation so rustdoc auto-trait synthesis does not walk deeply nested
//! persistent index maps (`im::OrdMap` + `im::Vector`) through page boundary
//! and pinned-read handles.

use std::any::Any;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use super::DerivedIndexGeneration;

trait DerivedIndexGenerationBacking: Send + Sync {
    fn as_generation(&self) -> &DerivedIndexGeneration;
    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync>;
}

impl DerivedIndexGenerationBacking for DerivedIndexGeneration {
    fn as_generation(&self) -> &DerivedIndexGeneration {
        self
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        self
    }
}

#[derive(Clone)]
pub struct SharedDerivedIndexGeneration(Arc<dyn DerivedIndexGenerationBacking>);

impl SharedDerivedIndexGeneration {
    pub fn from_arc(generation: Arc<DerivedIndexGeneration>) -> Self {
        Self(generation)
    }

    pub fn to_arc(&self) -> Arc<DerivedIndexGeneration> {
        Arc::clone(&self.0)
            .into_any()
            .downcast::<DerivedIndexGeneration>()
            .expect("SharedDerivedIndexGeneration always wraps DerivedIndexGeneration")
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        std::ptr::eq(&**self, &**other)
    }
}

impl From<Arc<DerivedIndexGeneration>> for SharedDerivedIndexGeneration {
    fn from(generation: Arc<DerivedIndexGeneration>) -> Self {
        Self::from_arc(generation)
    }
}

impl Deref for SharedDerivedIndexGeneration {
    type Target = DerivedIndexGeneration;

    fn deref(&self) -> &Self::Target {
        DerivedIndexGenerationBacking::as_generation(&*self.0)
    }
}

impl fmt::Debug for SharedDerivedIndexGeneration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(f)
    }
}
