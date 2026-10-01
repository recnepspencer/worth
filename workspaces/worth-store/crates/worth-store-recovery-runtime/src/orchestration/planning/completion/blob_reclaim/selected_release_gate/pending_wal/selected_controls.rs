//! Independent selected-media manifest and exhaustive reservation witnesses.

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimDescriptorV3, BlobRecordKind, BlobRecordV1,
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{PhysicalSourceSelection, WitnessedSelectedControlFrame};

use super::super::super::record;
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::orchestration::planning::selected_source_inventory::ResidentAllowance;

pub(super) struct SelectedPendingControls {
    manifest: Option<WitnessedSelectedControlFrame>,
    reservations: Vec<WitnessedSelectedControlFrame>,
    invalid_reservation: bool,
}

impl SelectedPendingControls {
    pub(super) fn reservation_count(&self) -> usize {
        self.reservations.len()
    }

    pub(super) fn into_witnesses(
        self,
    ) -> Option<(WitnessedSelectedControlFrame, WitnessedSelectedControlFrame)> {
        let manifest = self.manifest?;
        let [reservation] = self.reservations.try_into().ok()?;
        if self.invalid_reservation {
            return None;
        }
        Some((manifest, reservation))
    }
}

pub(super) fn observe(
    selected: &PhysicalSourceSelection,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    descriptor: BlobReclaimDescriptorV3,
    resident: &mut ResidentAllowance,
) -> Option<SelectedPendingControls> {
    let request = descriptor.custody().request();
    let manifest = read_control(
        selected,
        discovery,
        format,
        budget,
        trace,
        scratch,
        descriptor.base().manifest_record(),
        BlobRecordKind::DropSetManifestV3,
        resident,
    );
    // Re-read every reservation. A damaged sibling cannot disappear through
    // filter_map and leave one valid-looking reservation behind.
    resident
        .entries(2, std::mem::size_of::<WitnessedSelectedControlFrame>())
        .ok()?;
    let mut reservations = Vec::new();
    reservations.try_reserve_exact(2).ok()?;
    let mut invalid_reservation = false;
    for route in selected
        .page_facts()
        .placements()
        .iter()
        .copied()
        .filter(|route| {
            route.content_class()
                == SelectedRecordContentClass::Blob(BlobRecordKind::OriginalDropReserved)
        })
    {
        let Some(frame) = read_control(
            selected,
            discovery,
            format,
            budget,
            trace,
            scratch,
            route.record(),
            BlobRecordKind::OriginalDropReserved,
            resident,
        ) else {
            invalid_reservation = true;
            break;
        };
        let Ok(BlobRecordV1::OriginalDropReserved(value)) = decode_blob_record(frame.bytes())
        else {
            invalid_reservation = true;
            break;
        };
        if value.manifest_record() == descriptor.base().manifest_record()
            && value.request() == request
        {
            if reservations.len() == 2 {
                invalid_reservation = true;
                break;
            }
            reservations.push(frame);
        } else {
            let retained = u64::try_from(frame.bytes().len()).ok()?;
            drop(frame);
            resident.release(retained).ok()?;
        }
    }
    Some(SelectedPendingControls {
        manifest,
        reservations,
        invalid_reservation,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn read_control(
    selected: &PhysicalSourceSelection,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    record: PersistedRecordIdentity,
    kind: BlobRecordKind,
    resident: &mut ResidentAllowance,
) -> Option<WitnessedSelectedControlFrame> {
    let route = selected
        .page_facts()
        .placements()
        .iter()
        .copied()
        .find(|route| {
            route.record() == record
                && matches!(route,
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.content_class() == SelectedRecordContentClass::Blob(kind)
                    && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
        })?;
    resident.bytes(route.payload_bytes()).ok()?;
    resident
        .transient(
            route
                .payload_bytes()
                .checked_mul(3)?
                .checked_add(u64::from(format.page_size().bytes()).checked_mul(3)?)?
                .checked_add(1024)?,
        )
        .ok()?;
    let (bytes, witness) = record::read_with_witness(
        discovery,
        format,
        Some(route),
        record,
        BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        budget,
        trace,
        scratch,
        resident,
    )
    .ok()?;
    if <[u8; 32]>::from(Sha256::digest(&bytes)) != witness.payload_sha256() {
        return None;
    }
    resident.transient(u64::try_from(bytes.len()).ok()?).ok()?;
    WitnessedSelectedControlFrame::from_validated(bytes, witness).ok()
}
