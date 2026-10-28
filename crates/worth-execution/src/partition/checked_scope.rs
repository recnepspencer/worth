use std::{cell::Cell, mem::size_of};

use crate::{
    authority::{
        ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, ResourceReservation,
    },
    backend::{
        run_admitting_scope, KernelContext, KernelFailure, MemoryActivityGuard, ScopeKernelFailure,
        ScopeStop,
    },
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
    reservation: Option<RetainedReservation>,
    activity: Option<MemoryActivityGuard>,
}

enum RetainedReservation {
    Leased(ResourceReservation),
    Serial(ExecutionMemoryReservation),
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
            .ok_or_else(|| {
                PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
            })?;
        match &mut self.reservation {
            Some(RetainedReservation::Leased(reservation)) => lease
                .rebind_retained_memory(reservation, bytes)
                .map_err(PartitionUpdateDenial::Admission)?,
            Some(RetainedReservation::Serial(_)) => {
                return Err(PartitionUpdateDenial::Admission(LeaseDenial::refused(
                    LeaseDenial::UnrelatedNestedLease,
                )));
            }
            None => {
                self.reservation = Some(RetainedReservation::Leased(
                    lease
                        .reserve_retained_memory(bytes)
                        .map_err(PartitionUpdateDenial::Admission)?,
                ))
            }
        }
        self.activity = None;
        self.activity = crate::backend::enter_retained_memory(bytes);
        Ok(())
    }

    /// Use only the serial budget actually installed on the active request.
    pub(super) fn admit_serial(&mut self, bytes: u64) -> Result<(), PartitionUpdateDenial> {
        let bytes = bytes
            .checked_add(size_of::<Self>() as u64 + 64)
            .ok_or_else(|| {
                PartitionUpdateDenial::Admission(LeaseDenial::charged_bytes_overflow())
            })?;
        let serial = crate::backend::active_serial_memory().ok_or_else(|| {
            PartitionUpdateDenial::Admission(LeaseDenial::refused(
                LeaseDenial::NoActiveExecutionScope,
            ))
        })?;
        match &mut self.reservation {
            Some(RetainedReservation::Serial(memory)) => memory
                .take_over(None, Some(&serial), bytes)
                .map_err(PartitionUpdateDenial::Admission)?,
            Some(RetainedReservation::Leased(_)) => {
                return Err(PartitionUpdateDenial::Admission(LeaseDenial::refused(
                    LeaseDenial::UnrelatedNestedLease,
                )));
            }
            None => {
                self.reservation = Some(RetainedReservation::Serial(
                    ExecutionMemoryReservation::reserve_in_scope(None, bytes)
                        .map_err(PartitionUpdateDenial::Admission)?,
                ));
            }
        }
        self.activity = None;
        self.activity = crate::backend::enter_retained_memory(bytes);
        Ok(())
    }
}

/// The request entry already owns the active context and its original funding.
pub(super) fn request_edit(
    context: &mut KernelContext<'_, '_>,
    edit: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<PartitionWork, PartitionUpdateDenial>,
) -> Result<PartitionWork, PartitionUpdateDenial> {
    if !context.is_active() {
        return Err(PartitionUpdateDenial::Admission(
            context.refuse_request(LeaseDenial::UnrelatedNestedLease),
        ));
    }
    context.poll().map_err(PartitionUpdateDenial::Stop)?;
    let _structural_edit = StructuralEditGuard::enter();
    edit(context)
}

/// Bind a checked edit to both the invoking kernel and the supplied lease.
/// The child scope inherits parent work, memory, deadline, and cancellation.
pub(super) fn checked_edit(
    lease: &ExecutionResourceLease<'_>,
    parent: &mut KernelContext<'_, '_>,
    edit: impl FnOnce(&mut KernelContext<'_, '_>) -> Result<PartitionWork, PartitionUpdateDenial>,
) -> Result<PartitionWork, PartitionUpdateDenial> {
    if !parent.is_active() {
        let refused = Err(parent.refuse_request(PartitionUpdateDenial::Admission(
            LeaseDenial::UnrelatedNestedLease,
        )));
        // This checked edit has ended in admission failure on the context
        // it was asked through, before a child scope could be entered.
        parent.settle_result(&refused, |_| crate::backend::NestedStopOwner::Request);
        return refused;
    }
    let scope = run_admitting_scope(Some(lease), 0, 0, true, |child| {
        let _structural_edit = StructuralEditGuard::enter();
        edit(child).map_err(ScopeKernelFailure::from)
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

impl From<PartitionUpdateDenial> for ScopeKernelFailure<PartitionUpdateDenial> {
    fn from(denial: PartitionUpdateDenial) -> Self {
        match denial {
            PartitionUpdateDenial::Admission(_) => Self::Refused(denial),
            PartitionUpdateDenial::Stop(stop) => Self::Kernel(KernelFailure::Stop(stop)),
            PartitionUpdateDenial::KernelPanic => Self::Kernel(KernelFailure::Panic),
            PartitionUpdateDenial::ResultCapacityExceeded => {
                Self::Kernel(KernelFailure::ResultCapacityExceeded)
            }
            PartitionUpdateDenial::Components(_) | PartitionUpdateDenial::Bisection(_) => {
                Self::Kernel(KernelFailure::Domain(denial))
            }
        }
    }
}

#[cfg(test)]
mod tests;
