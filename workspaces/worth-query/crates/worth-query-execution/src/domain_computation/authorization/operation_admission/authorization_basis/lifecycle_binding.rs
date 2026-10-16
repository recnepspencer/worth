//! Paid, move-only inline custody for one native lifecycle binding.
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial as Denial;
use std::sync::OnceLock;
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionArray, ExecutionArrayBuilder, ExecutionMemoryReservation,
    ExecutionRequest, WorkCeilingDenial,
};

pub(in crate::domain_computation) struct LifecycleBinding<T> {
    slot: ExecutionArray<OnceLock<T>>,
    // A serial request has explicit system physical backing, so its original
    // active serial budget owns the exact slot charge separately. Leased
    // arrays already carry their allocation owner's charge and use None here.
    _serial_backing: Option<ExecutionMemoryReservation>,
}
impl<T> LifecycleBinding<T> {
    pub(in super::super) fn retain(
        execution: &ExecutionRequest<'_, '_>,
        value: T,
        subject: &str,
    ) -> Result<Self, (Denial, T)> {
        // Every fallible step precedes transfer of the unique binding. A stop
        // after push/seal still owns only an EMPTY cell, never caller authority.
        let allocate = execution
            .in_scope(|lease| {
                let serial_backing = if lease.is_none() {
                    Some(
                        ExecutionMemoryReservation::reserve_in_scope(
                            None,
                            std::mem::size_of::<OnceLock<T>>() as u64,
                        )
                        .map_err(|cause| {
                            Denial::lifecycle_storage_request(
                                WorkCeilingDenial::Admission(cause),
                                subject,
                            )
                        })?,
                    )
                } else {
                    None
                };
                let policy = lease.map_or(
                    ExecutionAllocationPolicy::SystemAllocation,
                    ExecutionAllocationPolicy::Execution,
                );
                let allocation_denied =
                    |cause| Denial::lifecycle_storage_allocation(cause, subject);
                let mut slots =
                    ExecutionArrayBuilder::allocate(1, policy).map_err(allocation_denied)?;
                slots.push(OnceLock::new()).map_err(allocation_denied)?;
                let slot = slots.seal().map_err(allocation_denied)?;
                execution.check_live().map_err(|stop| {
                    Denial::lifecycle_storage_request(WorkCeilingDenial::Stopped(stop), subject)
                })?;
                Ok::<_, Denial>((slot, serial_backing))
            })
            .map_err(|cause| Denial::lifecycle_storage_request(cause, subject))
            .and_then(std::convert::identity);
        let (slot, serial_backing) = match allocate {
            Ok(storage) => storage,
            Err(denial) => return Err((denial, value)),
        };
        assert!(slot[0].set(value).is_ok(), "new lifecycle slot is empty");
        Ok(Self {
            slot,
            _serial_backing: serial_backing,
        })
    }
    pub(in super::super) fn get(&self) -> &T {
        self.slot[0]
            .get()
            .expect("retained lifecycle slot is initialized")
    }
    pub(in super::super) fn into_inner(self) -> T {
        // No heap slot survives this consuming move to the caller's stack.
        // Nested payloads and native receipt obligations retain their own owners.
        self.slot
            .into_iter()
            .next()
            .expect("one lifecycle slot")
            .into_inner()
            .expect("retained lifecycle slot is initialized")
    }
}

#[cfg(test)]
#[path = "lifecycle_binding/tests.rs"]
mod tests;
