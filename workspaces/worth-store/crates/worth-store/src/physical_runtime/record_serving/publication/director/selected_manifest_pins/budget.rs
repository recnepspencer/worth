use std::collections::HashSet;

use worth_store_physical_format::{PersistedRecordIdentity, MAXIMUM_DROP_SET_RECORDS};

use super::{SelectedBlobManifestPin, SelectedBlobManifestPinDenial};

pub(super) fn selected_roster_reservation(
    maximum_pins: usize,
    memory_budget: u64,
    scratch_bytes: usize,
    nested_read_headroom: u64,
) -> Result<u64, SelectedBlobManifestPinDenial> {
    let available = memory_budget
        .checked_sub(nested_read_headroom)
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    // The decoder holds two maximum-sized identity arrays. Vec and HashSet
    // may each have old and growing storage live at once; four slots per pin
    // bounds both. Binding compaction also builds its 32-byte material list.
    let maximum_charge = (|| {
        let pins = maximum_pins.checked_mul(4)?;
        scratch_bytes
            .checked_add(std::mem::size_of::<HashSet<[u8; 16]>>())?
            .checked_add(if maximum_pins == 0 {
                0
            } else {
                MAXIMUM_DROP_SET_RECORDS
                    .checked_mul(std::mem::size_of::<PersistedRecordIdentity>())?
                    .checked_mul(2)?
            })?
            .checked_add(pins.checked_mul(std::mem::size_of::<SelectedBlobManifestPin>())?)?
            .checked_add(pins.checked_mul(32)?)?
            .checked_add(maximum_pins.checked_mul(64)?)
    })()
    .and_then(|bytes| u64::try_from(bytes).ok())
    .unwrap_or(u64::MAX);
    let reservation = maximum_charge.min(available);
    if reservation < scratch_bytes as u64 {
        return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
    }
    Ok(reservation)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roster_charge_preserves_nested_scan_and_denies_below_scratch() {
        let scratch = 64 * 1024;
        let headroom = 2 * 16 * 1024 + 32;
        let budget = 16 * 1024 * 1024;
        let charge = selected_roster_reservation(4_096, budget, scratch, headroom).unwrap();
        assert!(charge > scratch as u64);
        assert!(charge + headroom < budget);
        assert_eq!(
            selected_roster_reservation(4_096, (scratch as u64) + headroom - 1, scratch, headroom),
            Err(SelectedBlobManifestPinDenial::BudgetExceeded)
        );
    }
}
