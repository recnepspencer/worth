//! Opaque upper-layer synchronous output carrier for `worth-query-execution-attempt`.

use std::any::Any;

/// Opaque upper-layer `CapturedSynchronousOutputs`.
pub struct OpaqueSynchronousOutputs(pub Box<dyn Any + Send>);

impl OpaqueSynchronousOutputs {
    pub fn new<T: Any + Send>(value: T) -> Self {
        Self(Box::new(value))
    }

    pub fn downcast_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.0.downcast_mut::<T>()
    }

    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }

    pub fn downcast<T: Any>(self) -> Result<Box<T>, Self> {
        self.0.downcast::<T>().map_err(Self)
    }
}
