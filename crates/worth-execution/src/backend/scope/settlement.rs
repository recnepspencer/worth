use crate::authority::ExecutionMemoryReservation;
use crate::LeaseDenial;

/// Moves a held reservation to `kept` bytes.
///
/// `kept` is the success charge, or the overflow of computing it. A failure
/// passes zero. This does not release a refused success: the caller drops
/// that value first, then [`release_held_reservation`], so disposal still
/// sees the charge the failed resize left in place.
pub(super) fn apply_held_charge(
    reservation: &mut ExecutionMemoryReservation,
    kept: Result<u64, LeaseDenial>,
) -> Result<(), LeaseDenial> {
    let bytes = kept?;
    reservation
        .resize(bytes)
        .map_err(LeaseDenial::MemoryExhausted)
}

/// Releases a held reservation after a refused success has been dropped.
pub(super) fn release_held_reservation(reservation: &mut ExecutionMemoryReservation) {
    let _ = reservation.resize(0);
}
