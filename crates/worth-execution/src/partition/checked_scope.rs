use std::{cell::Cell, mem::size_of};

use crate::{
    authority::{ExecutionResourceLease, LeaseDenial, ResourceReservation},
    backend::{run_scope, KernelContext, KernelFailure, MemoryActivityGuard, ScopeStop},
    report::ChargedBytes,
};

use super::{PartitionUpdateDenial, PartitionWork};

thread_local! {
    static CHECKED_EDIT_DEPTH: Cell<u32> = const { Cell::new(0) };
}

struct StructuralEditGuard;

impl StructuralEditGuard {
    fn enter() -> Self {
        CHECKED_EDIT_DEPTH.with(|depth| depth.set(depth.get() + 1));
        Self
    }
}

impl Drop for StructuralEditGuard {
    fn drop(&mut self) {
        CHECKED_EDIT_DEPTH.with(|depth| depth.set(depth.get() - 1));
    }
}

pub(super) fn assert_offline_structural_edit(already_charged: bool) {
    let admitted = CHECKED_EDIT_DEPTH.with(|depth| depth.get() != 0);
    assert!(
        (!crate::backend::has_active_kernel() && !already_charged) || admitted,
        "pure partitioner edits are offline; use a checked edit in a kernel"
    );
}

/// Owns the live topology's charge across checked edits and request leases.
/// A rejected transfer leaves the prior reservation intact. Shrinking after a
/// successful edit releases excess charge; dropping the partitioner releases all.
#[derive(Default)]
pub(super) struct OwnedRetainedCharge {
    reservation: Option<ResourceReservation>,
    activity: Option<MemoryActivityGuard>,
}

impl OwnedRetainedCharge {
    pub(super) fn is_bound(&self) -> bool {
        self.reservation.is_some()
    }

    pub(super) fn admit(
        &mut self,
        lease: &ExecutionResourceLease<'_>,
        bytes: u64,
    ) -> Result<(), PartitionUpdateDenial> {
        // The owned charge is stored inline in the partitioner, but its
        // reservation retains a lineage Vec. Include both inline guard
        // metadata and two u64 slots per lineage node for Vec slack, plus a
        // small allocator header allowance, in every full-state admission.
        let bytes = u64::try_from(size_of::<Self>())
            .ok()
            .and_then(|base| base.checked_add(64))
            .and_then(|base| {
                u64::try_from(lease.lineage_depth())
                    .ok()
                    .and_then(|depth| depth.checked_mul(2 * size_of::<u64>() as u64))
                    .and_then(|lineage| base.checked_add(lineage))
            })
            .and_then(|overhead| overhead.checked_add(bytes))
            .ok_or(PartitionUpdateDenial::Admission(
                LeaseDenial::ChargedBytesOverflow,
            ))?;
        if let Some(reservation) = &mut self.reservation {
            lease
                .rebind_retained_memory(reservation, bytes)
                .map_err(PartitionUpdateDenial::Admission)?;
        } else {
            self.reservation = Some(
                lease
                    .reserve_retained_memory(bytes)
                    .map_err(PartitionUpdateDenial::Admission)?,
            );
        }
        self.activity = None;
        self.activity = crate::backend::enter_retained_memory(bytes);
        Ok(())
    }
}

/// Bind a checked edit to both the invoking kernel and the supplied lease.
/// The child scope inherits parent work, memory, deadline, and cancellation.
pub(super) fn checked_edit(
    lease: &ExecutionResourceLease<'_>,
    parent: &mut KernelContext<'_, '_>,
    edit: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<PartitionWork, PartitionUpdateDenial>,
) -> Result<PartitionWork, PartitionUpdateDenial> {
    if !parent.is_active() {
        return Err(PartitionUpdateDenial::Admission(
            LeaseDenial::UnrelatedNestedLease,
        ));
    }
    let scope = run_scope(Some(lease), 0, 0, |child| {
        let _structural_edit = StructuralEditGuard::enter();
        edit(child).map_err(KernelFailure::Domain)
    });
    match scope.result {
        Ok(work) => Ok(work),
        Err(ScopeStop::Admission(denial)) => Err(PartitionUpdateDenial::Admission(denial)),
        Err(ScopeStop::Failure(KernelFailure::Domain(denial))) => Err(denial),
        Err(ScopeStop::Failure(KernelFailure::Stop(stop))) => {
            Err(PartitionUpdateDenial::Stop(stop))
        }
        Err(ScopeStop::Failure(KernelFailure::Panic)) => Err(PartitionUpdateDenial::KernelPanic),
        Err(ScopeStop::Failure(KernelFailure::ResultCapacityExceeded)) => {
            Err(PartitionUpdateDenial::ResultCapacityExceeded)
        }
    }
}

impl ChargedBytes for PartitionWork {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

impl ChargedBytes for PartitionUpdateDenial {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}
