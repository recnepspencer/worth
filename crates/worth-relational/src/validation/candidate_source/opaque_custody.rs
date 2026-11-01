//! SDK-funded opaque Arc custody, shared by context and historical metadata.
use crate::execution::RelationalExecutionDenialCause as Cause;
use std::{
    alloc::Layout,
    any::Any,
    sync::{Arc, Weak},
};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};

type Payload = dyn Any + Send + Sync;
type Funding = Arc<Option<ExecutionMemoryReservation>>;

pub(crate) struct FundedSourceMaterialShare<T: ?Sized = Payload> {
    value: Arc<T>,
    // Last: original opaque storage closes before its sealed funding share.
    funding: Funding,
}
impl<T: ?Sized> Clone for FundedSourceMaterialShare<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            funding: self.funding.clone(),
        }
    }
}
impl<T: Any + Send + Sync> FundedSourceMaterialShare<T> {
    pub(crate) fn new(execution: &ExecutionRequest<'_, '_>, value: T) -> Result<Self, (Cause, T)> {
        let bytes = arc_bytes(Layout::new::<T>()).and_then(|bytes| {
            bytes.checked_add(arc_bytes(
                Layout::new::<Option<ExecutionMemoryReservation>>(),
            )?)
        });
        let Some(bytes) = bytes else {
            return Err((Cause::ChargedBytesOverflow, value));
        };
        let memory = match execution.reserve_retained_memory(bytes) {
            Ok(memory) => memory,
            Err(cause) => return Err((cause.into(), value)),
        };
        Ok(Self {
            value: Arc::new(value),
            funding: Arc::new(memory),
        })
    }
    pub(crate) fn receipt(&self) -> FundedSourceMaterialReceipt {
        let erased: Arc<Payload> = self.value.clone();
        FundedSourceMaterialReceipt {
            value: Arc::downgrade(&erased),
            funding: self.funding.clone(),
        }
    }
    pub(crate) fn erase(self) -> FundedSourceMaterialShare {
        FundedSourceMaterialShare {
            value: self.value,
            funding: self.funding,
        }
    }
}
impl<T: ?Sized> FundedSourceMaterialShare<T> {
    pub(crate) fn value(&self) -> &T {
        &self.value
    }
}

#[derive(Clone)]
pub(crate) struct FundedSourceMaterialReceipt {
    value: Weak<Payload>,
    funding: Funding,
}
impl FundedSourceMaterialReceipt {
    /// Quote only the existing outer Arc allocation and this funding Arc.
    /// Inner heaps remain with their strong owners and are not weak backing.
    /// Borrowed strong custody must remain alive throughout this constructor.
    pub(crate) fn retain(
        execution: &ExecutionRequest<'_, '_>,
        value: &Arc<Payload>,
    ) -> Result<Self, Cause> {
        let bytes = arc_bytes(Layout::for_value(value.as_ref()))
            .and_then(|bytes| {
                bytes.checked_add(arc_bytes(
                    Layout::new::<Option<ExecutionMemoryReservation>>(),
                )?)
            })
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(Self {
            value: Arc::downgrade(value),
            funding: Arc::new(memory),
        })
    }
    /// This can acquire the last strong opaque owner: caller owns disposal.
    pub(crate) fn upgrade(&self) -> Option<FundedSourceMaterialShare> {
        Some(FundedSourceMaterialShare {
            value: self.value.upgrade()?,
            funding: self.funding.clone(),
        })
    }
}
fn arc_bytes(value: Layout) -> Option<u64> {
    Layout::new::<[usize; 2]>()
        .extend(value)
        .ok()
        .and_then(|(layout, _)| u64::try_from(layout.pad_to_align().size()).ok())
}
