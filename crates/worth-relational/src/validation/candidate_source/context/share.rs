//! Bundled opaque allocation custody survives both strong and weak metadata shares.
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::{
    alloc::Layout,
    any::Any,
    sync::{Arc, Weak},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

type Payload = dyn Any + Send + Sync;
type Funding = Arc<Option<ExecutionMemoryReservation>>;

/// A strong metadata share with its original allocation funding. No raw Arc,
/// Weak, detachable funding token, READ or Source authority is exposed.
pub struct ManagedIndexSourceContextShare<T: ?Sized = Payload> {
    value: Arc<T>,
    // Last: the payload allocation closes before its original funding share.
    funding: Funding,
}
impl<T: ?Sized> Clone for ManagedIndexSourceContextShare<T> {
    fn clone(&self) -> Self {
        Self {
            value: Arc::clone(&self.value),
            funding: Arc::clone(&self.funding),
        }
    }
}
impl<T: Any + Send + Sync> ManagedIndexSourceContextShare<T> {
    /// Admit both exact Arc allocations on the actual original execution.
    /// Heaps inside T retain their own original owners. Refusal returns T so
    /// callers can retire opaque inputs outside their runtime/registry guards.
    pub fn new(execution: &ExecutionRequest<'_, '_>, value: T) -> Result<Self, (Cause, T)> {
        let bytes = arc_bytes::<T>()
            .and_then(|a| a.checked_add(arc_bytes::<Option<ExecutionMemoryReservation>>()?));
        let Some(bytes) = bytes else {
            return Err((Cause::ChargedBytesOverflow, value));
        };
        let memory = match execution.reserve_retained_memory(bytes) {
            Ok(memory) => memory,
            Err(cause) => return Err((cause.into(), value)),
        };
        let funding = Arc::new(memory);
        Ok(Self {
            value: Arc::new(value),
            funding,
        })
    }
    pub fn receipt(&self) -> ManagedIndexSourceContextReceipt {
        let erased: Arc<Payload> = self.value.clone();
        ManagedIndexSourceContextReceipt {
            value: Arc::downgrade(&erased),
            funding: self.funding.clone(),
        }
    }
    pub fn erase(self) -> ManagedIndexSourceContextShare {
        ManagedIndexSourceContextShare {
            value: self.value,
            funding: self.funding,
        }
    }
}
impl<T: ?Sized> ManagedIndexSourceContextShare<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
}
impl ManagedIndexSourceContextShare {
    pub fn downcast_ref<T: Any>(&self) -> Option<&T> {
        self.value.downcast_ref()
    }
}

/// Native retains only this weak, originally funded receipt. Lingering weak
/// receipts keep the entire inline Arc allocation charged after T has closed.
#[derive(Clone)]
pub struct ManagedIndexSourceContextReceipt {
    value: Weak<Payload>,
    // Only a sealed SDK reservation can drop under a Native guard.
    funding: Funding,
}
impl ManagedIndexSourceContextReceipt {
    pub fn upgrade(&self) -> Option<ManagedIndexSourceContextShare> {
        Some(ManagedIndexSourceContextShare {
            value: self.value.upgrade()?,
            funding: self.funding.clone(),
        })
    }
}
impl std::fmt::Debug for ManagedIndexSourceContextReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ManagedIndexSourceContextReceipt")
            .finish_non_exhaustive()
    }
}
fn arc_bytes<T>() -> Option<u64> {
    Layout::new::<[usize; 2]>()
        .extend(Layout::new::<T>())
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
}
