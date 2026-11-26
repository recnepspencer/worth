//! Selected C.11 manifest custody for checkpoint binding retention.

use std::collections::HashSet;

use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, PersistedRecordIdentity, RootPublicationCell,
    BLOB_RECORD_HEADER_BYTES, MAXIMUM_DROP_SET_RECORDS,
};

use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::PhysicalRecordReader;

#[path = "selected_manifest_pins/budget.rs"]
mod budget;
#[path = "selected_manifest_pins/carried.rs"]
mod carried;
#[cfg(feature = "certification-test-authority")]
#[path = "selected_manifest_pins/cutover_audit.rs"]
mod cutover_audit;
#[path = "selected_manifest_pins/read_deferred.rs"]
mod deferred_read;
#[path = "selected_manifest_pins/delta_scan.rs"]
mod delta_scan;
#[path = "selected_manifest_pins/full_scan.rs"]
mod full_scan;

pub(in crate::physical_runtime) use budget::checkpoint_pin_scan_bytes;
use budget::{checkpoint_pin_scan_bound, PinScanBound};
pub(in crate::physical_runtime::record_serving::publication::director) use carried::CarriedManifestRoster;
use delta_scan::delta_scan;
use full_scan::full_scan;
#[path = "selected_manifest_pins/roster_admission.rs"]
mod roster_admission;
use roster_admission::{charge_roster_growth, RosterLimits};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) struct SelectedBlobManifestPin {
    record: PersistedRecordIdentity,
    store: [u8; 16],
    attempt: [u8; 16],
    basis_digest: [u8; 32],
}

pub(in crate::physical_runtime) struct SelectedBlobManifestPins {
    root: RootPublicationCell,
    pins: Vec<SelectedBlobManifestPin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum SelectedBlobManifestPinDenial {
    RootUnavailable,
    ScanUnavailable,
    ReadUnavailable,
    MalformedManifest,
    ConflictingAttempt,
    BudgetExceeded,
}

impl SelectedBlobManifestPin {
    pub(in crate::physical_runtime) const fn store(self) -> [u8; 16] {
        self.store
    }

    pub(in crate::physical_runtime) const fn attempt(self) -> [u8; 16] {
        self.attempt
    }
}

impl SelectedBlobManifestPins {
    pub(in crate::physical_runtime) const fn root(&self) -> RootPublicationCell {
        self.root
    }

    pub(in crate::physical_runtime) fn pins(&self) -> &[SelectedBlobManifestPin] {
        &self.pins
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn empty_for_test() -> Self {
        let root = worth_store_physical_format::DurablePhysicalRootManifest::builder(1, 1, 2, 1)
            .admit()
            .expect("test root")
            .root_cell();
        Self {
            root,
            pins: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn one_for_test(store: [u8; 16], attempt: [u8; 16]) -> Self {
        let mut selected = Self::empty_for_test();
        selected.pins.push(SelectedBlobManifestPin {
            record: PersistedRecordIdentity::new([9; 16], 1).expect("test record"),
            store,
            attempt,
            basis_digest: [7; 32],
        });
        selected
    }
}

struct SelectedRosterInputs {
    root: RootPublicationCell,
    tree_identity: u64,
    limits: RosterLimits,
    reader: PhysicalRecordReader,
    scratch: Vec<u8>,
}

impl RecordPublicationDirector {
    /// Captures a protected selected root and derives its canonical C.5
    /// manifest roster. A roster carried from this director's previous
    /// selection is advanced by the routing delta alone (`O(touched)`); a
    /// cold director, another routing tree or an older root rebuilds by full
    /// scan. Failure leaves checkpoint publication unavailable; it cannot
    /// erase pins.
    pub(in crate::physical_runtime) fn selected_blob_manifest_pins(
        &self,
        maximum_pins: usize,
        memory_budget: u64,
    ) -> Result<SelectedBlobManifestPins, SelectedBlobManifestPinDenial> {
        let mut selected = self.selected_roster_inputs(maximum_pins, memory_budget)?;
        let carried = self.carried_roster_for(selected.root, selected.tree_identity);
        let (pins, _) = match &carried {
            Some(carried) => delta_scan(
                selected.reader,
                carried,
                selected.limits,
                &mut selected.scratch,
            )?,
            None => full_scan(selected.reader, selected.limits, &mut selected.scratch)?,
        };
        self.carry_roster(CarriedManifestRoster {
            root: selected.root,
            tree_identity: selected.tree_identity,
            pins: pins.clone(),
        });
        Ok(SelectedBlobManifestPins {
            root: selected.root,
            pins,
        })
    }

    /// One protected capture of the selected root, with its roster bound,
    /// reader and scan scratch.
    fn selected_roster_inputs(
        &self,
        maximum_pins: usize,
        memory_budget: u64,
    ) -> Result<SelectedRosterInputs, SelectedBlobManifestPinDenial> {
        let (root, protection) = self
            .capture_read_root()
            .map_err(|_| SelectedBlobManifestPinDenial::RootUnavailable)?;
        let runtime = self
            .runtime
            .upgrade()
            .ok_or(SelectedBlobManifestPinDenial::RootUnavailable)?;
        let root_cell = root.root_cell();
        let tree_identity = root.tree_identity();
        // The roster is held by the standing capture reservation, whose
        // envelope includes this exact bound; the scan takes no pool bytes.
        let PinScanBound {
            scratch_bytes,
            roster_bytes,
        } = checkpoint_pin_scan_bound(
            maximum_pins,
            memory_budget,
            u64::from(self.format.declaration().page_size().bytes()),
        )?;
        let limits = RosterLimits {
            store: self.durability.store_identity().bytes(),
            selected_root_generation: root_cell.generation().get(),
            maximum_pins,
            memory_budget: roster_bytes,
            scratch_bytes,
        };
        let reader = PhysicalRecordReader {
            execution: crate::physical_runtime::instance::PhysicalStoreWorkRuntime::execution(
                &runtime,
                self.generation,
            ),
            protection,
            store: self.durability.store_identity(),
            format: self.format,
            access: self.access,
            current_root: root,
            generation: self.generation,
            runtime: std::sync::Arc::downgrade(&runtime),
            lifecycle: self.reader_factory.reader(),
            residency: self.residency.clone().for_rebuild(),
        };
        let mut scratch = Vec::new();
        scratch
            .try_reserve_exact(scratch_bytes)
            .map_err(|_| SelectedBlobManifestPinDenial::BudgetExceeded)?;
        scratch.resize(scratch_bytes, 0);
        Ok(SelectedRosterInputs {
            root: root_cell,
            tree_identity,
            limits,
            reader,
            scratch,
        })
    }

    fn carried_roster_for(
        &self,
        current: RootPublicationCell,
        tree_identity: u64,
    ) -> Option<CarriedManifestRoster> {
        let carry = self
            .selected_pin_carry
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        carry
            .as_ref()
            .filter(|carried| carried.carries_to(current, tree_identity))
            .cloned()
    }

    fn carry_roster(&self, roster: CarriedManifestRoster) {
        *self
            .selected_pin_carry
            .lock()
            .unwrap_or_else(|error| error.into_inner()) = Some(roster);
    }
}

fn observe_manifest(
    bytes: &[u8],
    record: PersistedRecordIdentity,
    store: [u8; 16],
    selected_root_generation: u64,
    maximum_pins: usize,
    memory_budget: u64,
    scratch_bytes: usize,
    pins: &mut Vec<SelectedBlobManifestPin>,
    attempts: &mut HashSet<[u8; 16]>,
) -> Result<(), SelectedBlobManifestPinDenial> {
    if !bytes.starts_with(b"WRC11BLB") || !is_manifest_kind(bytes.get(8).copied()) {
        return Ok(());
    }
    // The count is a bounded wire field. Charge both temporary identity arrays
    // used by the decoder, plus worst-case roster/hash capacities, before it
    // allocates either array or an attempt-index bucket.
    let count_offset = manifest_count_offset(bytes)?;
    let count = bytes
        .get(count_offset..count_offset + 2)
        .map(|raw| u16::from_le_bytes([raw[0], raw[1]]) as usize)
        .filter(|count| (1..=MAXIMUM_DROP_SET_RECORDS).contains(count))
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)?;
    let temporary = count
        .checked_mul(std::mem::size_of::<PersistedRecordIdentity>())
        .and_then(|bytes| bytes.checked_mul(2))
        .ok_or(SelectedBlobManifestPinDenial::BudgetExceeded)?;
    charge_roster_growth(
        pins,
        attempts,
        temporary,
        maximum_pins,
        memory_budget,
        scratch_bytes,
    )?;
    let decoded =
        decode_blob_record(bytes).map_err(|_| SelectedBlobManifestPinDenial::MalformedManifest)?;
    let (manifest_store, attempt, basis_digest) = match decoded {
        BlobRecordV1::DropSetManifest(manifest) => (
            manifest.store(),
            manifest.reclaim_attempt(),
            manifest.source_basis_digest(),
        ),
        BlobRecordV1::DropSetManifestV2(manifest)
            if manifest.never_reserved_slot_generation() <= selected_root_generation =>
        {
            (
                manifest.store(),
                manifest.reclaim_attempt(),
                manifest.source_basis_digest(),
            )
        }
        BlobRecordV1::DropSetManifestV3(manifest)
            if manifest.never_reserved_slot_generation() <= selected_root_generation =>
        {
            (
                manifest.store(),
                manifest.reclaim_attempt(),
                manifest.source_basis_digest(),
            )
        }
        _ => return Err(SelectedBlobManifestPinDenial::MalformedManifest),
    };
    if manifest_store != store || !attempts.insert(attempt) {
        return Err(SelectedBlobManifestPinDenial::ConflictingAttempt);
    }
    pins.push(SelectedBlobManifestPin {
        record,
        store,
        attempt,
        basis_digest,
    });
    Ok(())
}

fn is_manifest_kind(kind: Option<u8>) -> bool {
    matches!(kind, Some(value) if value == BlobRecordKind::DropSetManifest as u8
        || value == BlobRecordKind::DropSetManifestV2 as u8
        || value == BlobRecordKind::DropSetManifestV3 as u8)
}

const MANIFEST_COUNT_OFFSET: usize = BLOB_RECORD_HEADER_BYTES + 160;
const MINIMUM_MANIFEST_FRAME_BYTES: usize = BLOB_RECORD_HEADER_BYTES + 194 + 24;

fn manifest_count_offset(bytes: &[u8]) -> Result<usize, SelectedBlobManifestPinDenial> {
    if bytes.get(8).copied() != Some(BlobRecordKind::DropSetManifestV3 as u8) {
        return Ok(MANIFEST_COUNT_OFFSET);
    }
    // V3 carries a length-tagged source basis; unlike V1/V2 its count has no
    // fixed offset. Inspect only the fixed prefix before charging decoder
    // storage for the untrusted count.
    let source_length_offset = BLOB_RECORD_HEADER_BYTES + 33;
    let source_length = bytes
        .get(source_length_offset..source_length_offset + 2)
        .map(|raw| u16::from_le_bytes([raw[0], raw[1]]) as usize)
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)?;
    (BLOB_RECORD_HEADER_BYTES + 35)
        .checked_add(source_length)
        .ok_or(SelectedBlobManifestPinDenial::MalformedManifest)
}

#[cfg(test)]
#[path = "selected_manifest_pins/carried_tests.rs"]
mod carried_tests;
#[cfg(test)]
#[path = "selected_manifest_pins/tests.rs"]
mod tests;
