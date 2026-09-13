use std::collections::BTreeMap;

use sha2::{Digest, Sha256};
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableExtentManifest, DurableExtentRecordPlacement, ExtentChunkCoordinate,
    PhysicalRecordFormatDeclaration, RecordArtifactFile, RecordFrameCoordinate,
};
use worth_store_physical_integrity::{
    IntegrityValidatedExtentMembership, PhysicalArtifactScope, PhysicalByteRange,
};
use worth_store_recovery_physics::{
    PhysicalRedoTarget, PhysicalRedoTargetIdentity, RecoveryPageObservation,
};

use super::{super::PageObservationFailure, observed::required_observed};

pub(crate) struct RecoveryExtentManifest {
    manifest: DurableExtentManifest,
    membership: IntegrityValidatedExtentMembership,
}

struct ExtentChunkObservationPlan {
    artifact: RecordArtifactFile,
    coordinate: ExtentChunkCoordinate,
    offset: u64,
    length: u32,
}

pub(crate) fn observe_extent(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    placement: DurableExtentRecordPlacement,
    target: &PhysicalRedoTarget,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    manifests: &mut BTreeMap<(u64, u64), RecoveryExtentManifest>,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<RecoveryPageObservation, PageObservationFailure> {
    let key = (placement.extent().get(), placement.extent_generation());
    admit_manifest_once(
        discovery, placement, target, format, byte_limit, key, manifests, integrity,
    )?;
    let admitted = manifests
        .get(&key)
        .expect("successful admission installs the exact extent manifest");
    admit_chunk(
        discovery, placement, target, format, byte_limit, admitted, integrity,
    )
}

#[allow(clippy::too_many_arguments)]
fn admit_manifest_once(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    placement: DurableExtentRecordPlacement,
    target: &PhysicalRedoTarget,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    key: (u64, u64),
    manifests: &mut BTreeMap<(u64, u64), RecoveryExtentManifest>,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<(), PageObservationFailure> {
    let std::collections::btree_map::Entry::Vacant(entry) = manifests.entry(key) else {
        return Ok(());
    };
    let artifact = RecordArtifactFile::ExtentManifest {
        extent: key.0,
        generation: key.1,
    };
    let observed = required_observed(
        discovery.read_extent_manifest(key.0, key.1, byte_limit),
        Some(target.identity()),
        artifact,
    )?;
    let bytes = observed
        .bytes()
        .ok_or(PageObservationFailure::MissingArtifact {
            target: Some(target.identity()),
            artifact,
        })?;
    let range = PhysicalByteRange::new(0, bytes.len() as u64)
        .map_err(|_| invalid_manifest(target, artifact))?;
    let scope = PhysicalArtifactScope::extent_manifest(
        discovery.store_identity(),
        format,
        placement,
        range,
    );
    let admitted =
        crate::integrity_ingress::admit_extent_manifest_projection(&observed, scope, integrity)
            .map_err(|_| invalid_manifest(target, artifact))?;
    let projection = admitted.projection;
    let manifest = DurableExtentManifest::new(
        projection.record_format,
        projection.record,
        projection.extent_cell,
        projection.logical_bytes,
        projection.maximum_frame_bytes,
        projection.chunk_count,
    )
    .filter(|manifest| {
        manifest.extent_cell() == placement.extent_cell() && manifest.record() == placement.record()
    })
    .ok_or_else(|| invalid_manifest(target, artifact))?;
    entry.insert(RecoveryExtentManifest {
        manifest,
        membership: admitted.membership,
    });
    Ok(())
}

fn admit_chunk(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    placement: DurableExtentRecordPlacement,
    target: &PhysicalRedoTarget,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    admitted: &RecoveryExtentManifest,
    integrity: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<RecoveryPageObservation, PageObservationFailure> {
    let manifest = admitted.manifest;
    let plan = plan_chunk_observation(placement, target, manifest)?;
    let frame = required_observed(
        discovery.read_extent_range(
            placement.extent().get(),
            placement.extent_generation(),
            plan.offset,
            plan.length,
            byte_limit,
        ),
        Some(target.identity()),
        plan.artifact,
    )?;
    let range = PhysicalByteRange::new(plan.offset, u64::from(plan.length))
        .map_err(|_| PageObservationFailure::InvalidPage(target.identity()))?;
    let scope = PhysicalArtifactScope::extent_chunk(
        discovery.store_identity(),
        format,
        plan.coordinate,
        range,
    );
    let projection = crate::integrity_ingress::admit_extent_chunk_projection(
        &frame,
        scope,
        admitted.membership,
        integrity,
    )
    .map_err(|_| PageObservationFailure::InvalidPage(target.identity()))?;
    let source = RecordFrameCoordinate::new(plan.artifact, plan.offset, plan.length)
        .ok_or(PageObservationFailure::InvalidPage(target.identity()))?;
    Ok(RecoveryPageObservation::materialized(
        target.identity(),
        projection.page_lsn.get(),
        projection.encoded_digest,
        source,
        routing_identity(manifest, placement, format),
    ))
}

fn plan_chunk_observation(
    placement: DurableExtentRecordPlacement,
    target: &PhysicalRedoTarget,
    manifest: DurableExtentManifest,
) -> Result<ExtentChunkObservationPlan, PageObservationFailure> {
    let chunk = target_chunk(target, manifest)?;
    let logical_offset = u64::from(chunk - 1) * u64::from(manifest.chunk_payload_capacity());
    let payload = (manifest.logical_bytes() - logical_offset)
        .min(u64::from(manifest.chunk_payload_capacity()));
    let length = u32::try_from(
        worth_store_physical_format::DURABLE_EXTENT_FRAME_HEADER_BYTES as u64
            + worth_store_physical_format::EXTENT_CHUNK_METADATA_BYTES as u64
            + payload,
    )
    .map_err(|_| PageObservationFailure::InvalidTarget(target.identity()))?;
    let offset = u64::from(chunk - 1) * u64::from(manifest.maximum_frame_bytes());
    require_chunk_coordinate(target, manifest, logical_offset, offset, length)?;
    let coordinate = ExtentChunkCoordinate::new(
        manifest.record(),
        manifest.extent_cell(),
        manifest.logical_bytes(),
        logical_offset,
        chunk,
    )
    .ok_or(PageObservationFailure::InvalidPage(target.identity()))?;
    Ok(ExtentChunkObservationPlan {
        artifact: RecordArtifactFile::Extent {
            extent: placement.extent().get(),
            generation: placement.extent_generation(),
        },
        coordinate,
        offset,
        length,
    })
}

fn target_chunk(
    target: &PhysicalRedoTarget,
    manifest: DurableExtentManifest,
) -> Result<u32, PageObservationFailure> {
    let PhysicalRedoTargetIdentity::ExtentChunk { chunk, .. } = target.identity() else {
        return Err(PageObservationFailure::InvalidPage(target.identity()));
    };
    if chunk == 0 || chunk > manifest.chunk_count() {
        return Err(PageObservationFailure::InvalidTarget(target.identity()));
    }
    Ok(chunk)
}

fn require_chunk_coordinate(
    target: &PhysicalRedoTarget,
    manifest: DurableExtentManifest,
    logical_offset: u64,
    offset: u64,
    length: u32,
) -> Result<(), PageObservationFailure> {
    let coordinate = target
        .extent_coordinate()
        .ok_or(PageObservationFailure::InvalidTarget(target.identity()))?;
    if coordinate.allocation_epoch() == manifest.record().allocation_epoch()
        && coordinate.record_ordinal() == manifest.record().ordinal()
        && coordinate.logical_bytes() == manifest.logical_bytes()
        && coordinate.logical_offset() == logical_offset
        && target.artifact_offset() == offset
        && target.artifact_length() == length
    {
        Ok(())
    } else {
        Err(PageObservationFailure::InvalidTarget(target.identity()))
    }
}

fn routing_identity(
    manifest: DurableExtentManifest,
    placement: DurableExtentRecordPlacement,
    format: PhysicalRecordFormatDeclaration,
) -> [u8; 32] {
    let mut routing = Sha256::new();
    routing.update(b"worth.store.recovery.extent-routing.v1");
    routing.update(manifest.encode(format));
    routing.update(placement.record().allocation_epoch());
    routing.update(placement.record().ordinal().to_le_bytes());
    routing.update(placement.extent().get().to_le_bytes());
    routing.update(placement.extent_generation().to_le_bytes());
    routing.finalize().into()
}

fn invalid_manifest(
    target: &PhysicalRedoTarget,
    artifact: RecordArtifactFile,
) -> PageObservationFailure {
    PageObservationFailure::InvalidManifest {
        target: Some(target.identity()),
        artifact,
    }
}
