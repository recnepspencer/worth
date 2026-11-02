//! Process-local provenance of one actual graph-provider reservation.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

static NEXT_RESERVATION: AtomicU64 = AtomicU64::new(1);

/// Identifies a reserved graph capacity owner; it grants no execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCapacityReservationIdentity(u64);
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryGraphCapacityReservationIdentityExhausted;

impl WorthQueryGraphCapacityReservationIdentity {
    pub(super) fn mint() -> Result<Self, WorthQueryGraphCapacityReservationIdentityExhausted> {
        mint_from(&NEXT_RESERVATION)
    }
}

fn mint_from(
    counter: &AtomicU64,
) -> Result<
    WorthQueryGraphCapacityReservationIdentity,
    WorthQueryGraphCapacityReservationIdentityExhausted,
> {
    let mut current = counter.load(Relaxed);
    loop {
        let next = current
            .checked_add(1)
            .ok_or(WorthQueryGraphCapacityReservationIdentityExhausted)?;
        match counter.compare_exchange_weak(current, next, Relaxed, Relaxed) {
            Ok(value) => return Ok(WorthQueryGraphCapacityReservationIdentity(value)),
            Err(observed) => current = observed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn graph_reservation_identity_exhaustion_is_permanent_and_never_wraps() {
        let counter = AtomicU64::new(u64::MAX - 1);
        assert_eq!(
            mint_from(&counter),
            Ok(WorthQueryGraphCapacityReservationIdentity(u64::MAX - 1))
        );
        for _ in 0..3 {
            assert_eq!(
                mint_from(&counter),
                Err(WorthQueryGraphCapacityReservationIdentityExhausted)
            );
            assert_eq!(counter.load(Relaxed), u64::MAX);
        }
    }
}
