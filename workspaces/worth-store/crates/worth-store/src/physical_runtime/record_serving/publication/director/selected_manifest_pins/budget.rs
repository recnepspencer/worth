use std::collections::HashSet;

use worth_store_physical_format::{
    PersistedRecordIdentity, BLOB_CONTROL_FRAME_MAX_BYTES, MAXIMUM_DROP_SET_RECORDS,
};

use super::{SelectedBlobManifestPin, SelectedBlobManifestPinDenial, MINIMUM_MANIFEST_FRAME_BYTES};

pub(super) struct PinScanBound {
    pub(super) scratch_bytes: usize,
    /// Scratch, roster and handoff bytes live through checkpoint publication.
    pub(super) roster_bytes: u64,
}

/// The pin scan's live bound under the admitted live-binding limit and
/// checkpoint memory limit. Its nested scan reads admit separately in
/// Maintenance scope, which may spend the pool's progress headroom.
pub(super) fn checkpoint_pin_scan_bound(
    maximum_pins: usize,
    memory_budget: u64,
    page_bytes: u64,
) -> Result<PinScanBound, SelectedBlobManifestPinDenial> {
    let scratch_bytes = usize::try_from(memory_budget / 4)
        .unwrap_or(usize::MAX)
        .max(MINIMUM_MANIFEST_FRAME_BYTES)
        .min(BLOB_CONTROL_FRAME_MAX_BYTES);
    // The scan owns one page plus one row, and its payload or deferred
    // reader owns another page at the same time.
    let nested_read_headroom = page_bytes
        .checked_mul(2)
        .and_then(|bytes| {
            bytes.checked_add(std::mem::size_of::<
                crate::physical_runtime::record_serving::ScannedPhysicalRecord,
            >() as u64)
        })
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    let roster_bytes = selected_roster_reservation(
        maximum_pins,
        memory_budget,
        scratch_bytes,
        nested_read_headroom,
    )?;
    Ok(PinScanBound {
        scratch_bytes,
        roster_bytes,
    })
}

/// The capture envelope's pin-scan term. A policy whose scan cannot run
/// denies before the scan allocates, so it needs no bytes.
pub(in crate::physical_runtime) fn checkpoint_pin_scan_bytes(
    durability: crate::physical_runtime::PhysicalDurabilityObservation,
    format: crate::physical_runtime::record_serving::AdmittedPhysicalRecordFormat,
) -> u64 {
    checkpoint_pin_scan_bound(
        durability
            .idempotency_policy()
            .live_binding_limit()
            .get()
            .get() as usize,
        durability.checkpoint_policy().memory_limit().get().get(),
        u64::from(format.declaration().page_size().bytes()),
    )
    .map_or(0, |bound| bound.roster_bytes)
}

fn selected_roster_reservation(
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
