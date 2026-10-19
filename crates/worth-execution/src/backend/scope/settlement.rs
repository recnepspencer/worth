use super::{ScopeHold, ScopeStop};
use crate::{LeaseDenial, MapKernelFailure as KernelFailure};

/// Leaves `hold` holding what a completed result keeps, and nothing after a
/// failure. Holding less never refuses.
pub(super) fn settle_hold<R, E>(
    hold: ScopeHold<'_, R>,
    result: Result<R, KernelFailure<E>>,
) -> Result<R, ScopeStop<E>> {
    let kept = match &result {
        Ok(value) => (hold.kept)(value).ok_or_else(LeaseDenial::charged_bytes_overflow),
        Err(_) => Ok(0),
    };
    let held = kept.and_then(|bytes| {
        hold.reservation
            .resize(bytes)
            .map_err(LeaseDenial::MemoryExhausted)
    });
    match (result, held) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(denial)) => {
            let _ = hold.reservation.resize(0);
            Err(ScopeStop::Admission(denial))
        }
        (Err(failure), _) => Err(ScopeStop::Failure(failure)),
    }
}
