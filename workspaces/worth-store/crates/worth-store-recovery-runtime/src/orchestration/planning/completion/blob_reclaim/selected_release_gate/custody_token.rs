//! Mint selected release custody only from independently admitted C.9 extent
//! payloads. Candidate bytes, a selected route, or a checkpoint hash alone are
//! not authority to reopen after the source closure has been retired.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    decode_blob_record, BlobRecordKind, BlobRecordV1, CurrentPhysicalRecordPlacement,
    PersistedRecordIdentity, PhysicalRecordFormatDeclaration, SelectedRecordContentClass,
    BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_physical_integrity::IntegrityValidatedSelectedExtentPayload;
use worth_store_recovery_physics::{
    PhysicalSourceSelection, ReconciledOperationFates, VerifiedSelectedCheckpointCustody,
    WitnessedSelectedControlFrame,
};

use super::super::record;
use crate::{
    integrity_ingress::RecoveryIntegrityIngressTrace,
    orchestration::planning::manifest_entry_budget::ManifestEntryBudget,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn admit(
    selected: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    policy: [u8; 32],
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    descriptor_bytes: Vec<u8>,
    descriptor_witness: IntegrityValidatedSelectedExtentPayload,
) -> Option<VerifiedSelectedCheckpointCustody> {
    let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
        decode_blob_record(&descriptor_bytes).ok()?
    else {
        return None;
    };
    let manifest_id = descriptor.base().manifest_record();
    let descriptor =
        WitnessedSelectedControlFrame::from_validated(descriptor_bytes, descriptor_witness).ok()?;
    let reservation = read_control(
        selected,
        discovery,
        format,
        budget,
        trace,
        scratch,
        // The selected accumulator identifies the reservation; do not
        // derive a guessed route from the descriptor itself.
        selected
            .checkpoint()
            .and_then(|checkpoint| {
                super::certificates::selected_roster(checkpoint)
                    .ok()
                    .flatten()
            })?
            .accumulator()
            .tip()
            .reservation_record(),
        BlobRecordKind::OriginalDropReserved,
    )?;
    let manifest = read_control(
        selected,
        discovery,
        format,
        budget,
        trace,
        scratch,
        manifest_id,
        BlobRecordKind::DropSetManifestV3,
    )?;
    VerifiedSelectedCheckpointCustody::admit_selected_release(
        selected,
        &descriptor,
        &reservation,
        &manifest,
        fates,
        policy,
    )
    .ok()
}

#[allow(clippy::too_many_arguments)]
fn read_control(
    selected: &PhysicalSourceSelection,
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    scratch: &mut u64,
    id: PersistedRecordIdentity,
    kind: BlobRecordKind,
) -> Option<WitnessedSelectedControlFrame> {
    let route = selected
        .page_facts()
        .placements()
        .iter()
        .copied()
        .find(|route| {
            route.record() == id
                && matches!(route,
            CurrentPhysicalRecordPlacement::Extent(extent)
                if extent.content_class() == SelectedRecordContentClass::Blob(kind)
                    && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
        })?;
    let (bytes, witness) = record::read_with_witness(
        discovery,
        format,
        Some(route),
        id,
        BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        budget,
        trace,
        scratch,
    )
    .ok()?;
    WitnessedSelectedControlFrame::from_validated(bytes, witness).ok()
}
