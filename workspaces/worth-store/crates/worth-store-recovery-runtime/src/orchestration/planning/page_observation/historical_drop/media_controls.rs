//! Bounded selected-media reads used by historical release classification.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    BlobRecordKind, CurrentPhysicalRecordPlacement, PersistedRecordIdentity,
    PhysicalRecordFormatDeclaration, SelectedRecordContentClass, BLOB_CONTROL_FRAME_MAX_BYTES,
};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, WitnessedSelectedControlFrame,
};

use crate::integrity_ingress::{
    admit_addressed_root, RecoveryArtifactNamespaceJoin, RecoveryIntegrityIngressTrace,
};
use crate::orchestration::planning::page_observation::ordered_history::WalkFailure;
use crate::orchestration::planning::{
    completion::blob_reclaim::record, manifest_entry_budget::ManifestEntryBudget,
    selected_source_inventory::ResidentAllowance,
};

pub(super) fn target_record(target: &PhysicalRedoTarget) -> Option<PersistedRecordIdentity> {
    match target.identity() {
        PhysicalRedoTargetIdentity::ExtentChunk { .. } => {
            let coordinate = target.extent_coordinate()?;
            PersistedRecordIdentity::new(coordinate.allocation_epoch(), coordinate.record_ordinal())
        }
        PhysicalRedoTargetIdentity::InlinePage { .. } => None,
    }
}

pub(in crate::orchestration::planning::page_observation) fn selected_control(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    routes: &[CurrentPhysicalRecordPlacement],
    record_id: PersistedRecordIdentity,
    kind: BlobRecordKind,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    resident: &mut ResidentAllowance,
) -> Option<WitnessedSelectedControlFrame> {
    let route = routes
        .iter()
        .copied()
        .find(|route| route.record() == record_id)?;
    if !matches!(route, CurrentPhysicalRecordPlacement::Extent(extent)
        if extent.content_class() == SelectedRecordContentClass::Blob(kind)
            && extent.payload_bytes() <= BLOB_CONTROL_FRAME_MAX_BYTES as u64)
    {
        return None;
    }
    let payload_bytes = route.payload_bytes();
    resident.bytes(payload_bytes).ok()?;
    // A retained payload can coexist with a replacement Box and a decoded
    // manifest's dropped-record backing. Preflight that window before media IO.
    resident.transient(payload_bytes.checked_mul(2)?).ok()?;
    let (bytes, witness) = record::read_with_witness(
        discovery,
        format,
        Some(route),
        record_id,
        BLOB_CONTROL_FRAME_MAX_BYTES as u64,
        budget,
        trace,
        &mut 0,
        resident,
    )
    .ok()?;
    WitnessedSelectedControlFrame::from_validated(bytes, witness).ok()
}

pub(in crate::orchestration::planning::page_observation) fn source_root(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    generation: u64,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
) -> Result<worth_store_physical_format::DurablePhysicalRootManifest, WalkFailure> {
    budget.consume(1)?;
    let source = discovery.read_root_manifest(generation, u64::from(format.page_size().bytes()))?;
    let admitted = admit_addressed_root(
        RecoveryArtifactNamespaceJoin::from_canonical(&source),
        discovery.store_identity(),
        format,
        generation,
    )
    .map_err(|_| WalkFailure::Unverified)?;
    let (root, observed_format) = admitted.project();
    (observed_format == format && root.generation() == generation)
        .then_some(root)
        .ok_or(WalkFailure::Unverified)
}
