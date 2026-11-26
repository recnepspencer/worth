//! Admission of one roster entry under the selected root's roster bound.

use std::collections::HashSet;

use worth_store_physical_format::PersistedRecordIdentity;

use super::{observe_manifest, SelectedBlobManifestPin, SelectedBlobManifestPinDenial};

/// The admitted roster bounds of one selected root.
#[derive(Debug, Clone, Copy)]
pub(super) struct RosterLimits {
    pub(super) store: [u8; 16],
    pub(super) selected_root_generation: u64,
    pub(super) maximum_pins: usize,
    pub(super) memory_budget: u64,
    pub(super) scratch_bytes: usize,
}

impl RosterLimits {
    pub(super) fn observe(
        self,
        bytes: &[u8],
        record: PersistedRecordIdentity,
        pins: &mut Vec<SelectedBlobManifestPin>,
        attempts: &mut HashSet<[u8; 16]>,
    ) -> Result<(), SelectedBlobManifestPinDenial> {
        observe_manifest(
            bytes,
            record,
            self.store,
            self.selected_root_generation,
            self.maximum_pins,
            self.memory_budget,
            self.scratch_bytes,
            pins,
            attempts,
        )
    }

    /// Re-admits a pin verified for the carried root under this root's bound.
    /// Its manifest bytes are unchanged, and a V2/V3 slot-generation check
    /// that held at an older root still holds at this one.
    pub(super) fn admit_carried(
        self,
        pin: SelectedBlobManifestPin,
        pins: &mut Vec<SelectedBlobManifestPin>,
        attempts: &mut HashSet<[u8; 16]>,
    ) -> Result<(), SelectedBlobManifestPinDenial> {
        charge_roster_growth(
            pins,
            attempts,
            0,
            self.maximum_pins,
            self.memory_budget,
            self.scratch_bytes,
        )?;
        if pin.store != self.store || !attempts.insert(pin.attempt) {
            return Err(SelectedBlobManifestPinDenial::ConflictingAttempt);
        }
        pins.push(pin);
        Ok(())
    }
}

/// Charges one more roster entry, plus `temporary` decoder bytes, against the
/// roster budget and reserves its slots before any decode allocates.
pub(super) fn charge_roster_growth(
    pins: &mut Vec<SelectedBlobManifestPin>,
    attempts: &mut HashSet<[u8; 16]>,
    temporary: usize,
    maximum_pins: usize,
    memory_budget: u64,
    scratch_bytes: usize,
) -> Result<(), SelectedBlobManifestPinDenial> {
    if pins.len() >= maximum_pins {
        return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
    }
    let next = pins.len() + 1;
    let pin_capacity = pins.capacity().max(next.saturating_mul(4));
    // HashSet's bucket array includes control bytes and may grow ahead of len.
    // Four 32-byte buckets per entry conservatively covers its growth and
    // avoids a per-insert tree allocation with an opaque node layout.
    let attempt_capacity = attempts.capacity().max(next.saturating_mul(4));
    let charged = scratch_bytes
        .checked_add(temporary)
        .and_then(|bytes| {
            bytes.checked_add(
                pin_capacity.checked_mul(std::mem::size_of::<SelectedBlobManifestPin>())?,
            )
        })
        .and_then(|bytes| bytes.checked_add(attempt_capacity.checked_mul(32)?))
        // Binding compaction copies one 32-byte material per pin. Include a
        // conservative second array for allocator growth at the handoff.
        .and_then(|bytes| bytes.checked_add(next.checked_mul(64)?))
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<HashSet<[u8; 16]>>()))
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    if charged as u64 > memory_budget {
        return Err(SelectedBlobManifestPinDenial::BudgetExceeded);
    }
    pins.try_reserve_exact(1)
        .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
    attempts
        .try_reserve(1)
        .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
    Ok(())
}
