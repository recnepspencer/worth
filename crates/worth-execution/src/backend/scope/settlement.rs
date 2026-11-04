//! What happens to a payload after its kernel returned: measured, refused or
//! kept. Its measure and destructor are caller code, so each runs contained.
use super::body::{HeldPayload, KernelVerdict, ScopeBody, ScopeVerdict};
use super::ScopeStop;
use crate::authority::ExecutionMemoryReservation;
use crate::backend::meter::KernelFailure;
use crate::backend::panic_boundary::contain;
use crate::LeaseDenial;
use std::panic::AssertUnwindSafe;

/// A panicking destructor cannot replace the outcome that disposed of it.
pub(super) fn discard_payload(body: &mut dyn ScopeBody) {
    let _ = contain(AssertUnwindSafe(|| body.discard()));
}

fn holds_payload(invoked: &KernelVerdict) -> bool {
    matches!(
        invoked,
        Ok(HeldPayload) | Err(KernelFailure::Domain(HeldPayload))
    )
}

/// What the held payload charges, refusing it past `capacity`. A panicking
/// measure leaves no size to trust, so it fails the scope like a panicking kernel.
pub(super) fn weigh_payload(
    invoked: KernelVerdict,
    capacity: Option<u64>,
    body: &mut dyn ScopeBody,
) -> (KernelVerdict, u64) {
    if !holds_payload(&invoked) {
        return (invoked, 0);
    }
    match contain(AssertUnwindSafe(|| body.payload_bytes())) {
        Ok(bytes) if capacity.is_some_and(|capacity| bytes > capacity) => {
            discard_payload(body);
            (Err(KernelFailure::ResultCapacityExceeded), 0)
        }
        Ok(bytes) => (invoked, bytes),
        Err(()) => {
            discard_payload(body);
            (Err(KernelFailure::Panic), 0)
        }
    }
}

/// Grows the original reservation over the payload's actual heap.
pub(super) fn admit_measured_payload(
    reservation: &mut ExecutionMemoryReservation,
    memory_bytes: u64,
    payload_bytes: u64,
    invoked: KernelVerdict,
    body: &mut dyn ScopeBody,
) -> ScopeVerdict {
    let admitted = memory_bytes
        .checked_add(payload_bytes)
        .ok_or_else(LeaseDenial::charged_bytes_overflow)
        .and_then(|total| {
            reservation
                .resize(total)
                .map_err(LeaseDenial::MemoryExhausted)
        });
    match admitted {
        Ok(()) => invoked.map_err(ScopeStop::Failure),
        Err(denial) => {
            // Keep the original reservation until the unretained materialized payload is gone.
            discard_payload(body);
            Err(ScopeStop::Admission(denial))
        }
    }
}

/// Keeps a measured value or error charged until its caller releases it.
pub(super) fn settle_measured_hold(
    reservation: &mut ExecutionMemoryReservation,
    payload_bytes: u64,
    verdict: ScopeVerdict,
) -> ScopeVerdict {
    let kept = match &verdict {
        Ok(HeldPayload) | Err(ScopeStop::Failure(KernelFailure::Domain(HeldPayload))) => {
            payload_bytes
        }
        Err(_) => 0,
    };
    reservation
        .resize(kept)
        .expect("measured settlement only releases bytes");
    verdict
}

/// Leaves the hold holding what a completed result keeps, and nothing after a
/// failure. Holding less never refuses.
pub(super) fn settle_hold(
    reservation: &mut ExecutionMemoryReservation,
    verdict: ScopeVerdict,
    body: &mut dyn ScopeBody,
) -> ScopeVerdict {
    let refusal = if verdict.is_ok() {
        match contain(AssertUnwindSafe(|| body.kept_bytes())) {
            Ok(Some(kept)) => match reservation.resize(kept) {
                Ok(()) => return verdict,
                Err(denial) => ScopeStop::Admission(LeaseDenial::MemoryExhausted(denial)),
            },
            Ok(None) => ScopeStop::Admission(LeaseDenial::charged_bytes_overflow()),
            Err(()) => ScopeStop::Failure(KernelFailure::Panic),
        }
    } else {
        let _ = reservation.resize(0);
        return verdict;
    };
    // The reservation outlives the result it no longer admits.
    discard_payload(body);
    let _ = reservation.resize(0);
    Err(refusal)
}

#[cfg(test)]
mod tests;
