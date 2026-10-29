//! One compiled batch admission. The shell names payload sizes and then runs.
use crate::authority::{
    ExecutionMemoryReservation, ExecutionResourceLease, LeaseDenial, ResourceReservation,
};
use worth_foundational::{ExecutionFallbackCause, ExecutionPhysicalReport, ExecutionPosture};

use super::{
    meter::{MemoryActivityGuard, RunLimits},
    port::BackendKind,
    PreparedBatchResources,
};

/// Limits and reservations that stay live across the typed kernel.
///
/// Field order is drop order. Memory activity releases before limits, and both
/// release before the caller's custody, whose entry reservation drops after
/// limits on unwind.
pub(super) struct BatchRunPrefix {
    pub(super) memory_activity: Option<MemoryActivityGuard>,
    pub(super) limits: RunLimits,
    pub(super) requested_posture: ExecutionPosture,
    pub(super) initial_fallback: Option<ExecutionFallbackCause>,
    pub(super) accounted: bool,
}

pub(super) enum BatchRunPrefixStop {
    Denied {
        denial: LeaseDenial,
        accounted: bool,
        /// `None` until limits exist. The shell drops this after custody finish.
        limits: Option<RunLimits>,
    },
    NoWork {
        physical: ExecutionPhysicalReport,
        accounted: bool,
        limits: RunLimits,
    },
}

/// Parent checks, limits, posture, and reservations for one batch.
///
/// Charged bytes are the shell's fact: that sum depends on payload layout.
/// The callback is erased so this body is compiled once.
#[inline(never)]
pub(super) fn admit_batch_prefix(
    lease: Option<&ExecutionResourceLease<'_>>,
    batch_len: usize,
    retained: bool,
    backend: BackendKind,
    native: bool,
    prepared: Option<&PreparedBatchResources>,
    reservation: &mut Option<ResourceReservation>,
    serial_reservation: &mut Option<ExecutionMemoryReservation>,
    taken: &mut Option<ExecutionMemoryReservation>,
    memory_bytes: &mut dyn FnMut(&RunLimits, usize) -> Option<u64>,
) -> Result<BatchRunPrefix, BatchRunPrefixStop> {
    let accounted = !native || RunLimits::native_accounting(lease);
    if lease.is_none() && RunLimits::has_leased_parent() {
        return Err(denied(
            LeaseDenial::unrelated_nested_lease(),
            accounted,
            None,
        ));
    }
    if prepared.is_some_and(|resources| !resources.matches_current_parent()) {
        return Err(denied(
            LeaseDenial::unrelated_nested_lease(),
            accounted,
            None,
        ));
    }
    let limits = if let Some(resources) = prepared {
        resources.dispatch_limits(lease.expect("prepared map requires its lease"), backend)
    } else if native {
        RunLimits::for_native(lease, matches!(backend, BackendKind::Serial))
    } else {
        RunLimits::for_run(lease, matches!(backend, BackendKind::Serial))
    };
    let requested_posture = if lease
        .is_some_and(|value| value.resolved_posture() == ExecutionPosture::Automatic)
        && !limits.force_serial()
        && !matches!(backend, BackendKind::Serial)
    {
        ExecutionPosture::Automatic
    } else {
        ExecutionPosture::Serial
    };
    let initial_fallback = if requested_posture == ExecutionPosture::Serial {
        // A lease that resolves serial names why. An automatic lease runs
        // serial only under a serial oracle: this run's, or the enclosing
        // run's that its limits inherit.
        Some(match lease {
            None => ExecutionFallbackCause::NoLease,
            Some(value) => value
                .serial_cause()
                .unwrap_or(ExecutionFallbackCause::OracleSerial),
        })
    } else {
        None
    };
    if batch_len == 0 && !retained {
        let physical = if prepared.is_some() {
            ExecutionPhysicalReport::new(
                limits.worker_high_watermark(),
                limits.peak_memory(),
                Some(0),
                0,
                limits.discarded_work(),
            )
        } else {
            ExecutionPhysicalReport::default()
        };
        return Err(BatchRunPrefixStop::NoWork {
            physical,
            accounted,
            limits,
        });
    }
    // A lease-free run holds its framework at one worker: its own thread.
    let max_workers = lease.map_or(1, |value| value.policy().max_workers().get());
    let Some(memory_bytes) = memory_bytes(&limits, max_workers) else {
        return Err(denied(
            LeaseDenial::charged_bytes_overflow(),
            accounted,
            Some(limits),
        ));
    };
    if prepared.is_some_and(|resources| !resources.matches_dispatch(memory_bytes)) {
        return Err(denied(
            LeaseDenial::unrelated_nested_lease(),
            accounted,
            Some(limits),
        ));
    }
    *reservation = if let Some(lease) = lease {
        let admission_bytes = if prepared.is_some() || taken.is_some() {
            0
        } else {
            memory_bytes
        };
        match if limits.is_accounted() {
            lease.reserve_entry(admission_bytes)
        } else {
            lease.reserve_worker_entry()
        } {
            Ok(guard) => Some(guard),
            Err(denial) => {
                return Err(denied(denial, accounted, Some(limits)));
            }
        }
    } else {
        None
    };
    let take_over_denial = taken.as_mut().and_then(|held| {
        held.take_over(lease, limits.serial_memory(), memory_bytes)
            .err()
    });
    if let Some(denial) = take_over_denial {
        // The caller still covers every input. Close the zero-byte entry
        // before refused inputs can clean up otherwise empty ledger nodes.
        drop(reservation.take());
        return Err(denied(denial, accounted, Some(limits)));
    }
    let serial_result = match (lease, limits.serial_memory(), taken.as_ref()) {
        (None, Some(budget), None) => Some(budget.reserve(memory_bytes)),
        _ => None,
    };
    *serial_reservation = match serial_result {
        Some(Ok(guard)) => Some(guard),
        Some(Err(denial)) => {
            return Err(denied(
                LeaseDenial::MemoryExhausted(denial),
                accounted,
                Some(limits),
            ));
        }
        None => None,
    };
    let memory_activity = if !limits.is_accounted() || prepared.is_some() {
        None
    } else {
        (reservation.is_some() || serial_reservation.is_some() || taken.is_some())
            .then(|| limits.enter_memory(memory_bytes))
    };
    Ok(BatchRunPrefix {
        memory_activity,
        limits,
        requested_posture,
        initial_fallback,
        accounted,
    })
}

fn denied(denial: LeaseDenial, accounted: bool, limits: Option<RunLimits>) -> BatchRunPrefixStop {
    BatchRunPrefixStop::Denied {
        denial,
        accounted,
        limits,
    }
}
