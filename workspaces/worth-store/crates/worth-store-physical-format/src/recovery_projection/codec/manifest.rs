use super::*;

pub(super) fn write_manifest(target: &mut Vec<u8>, manifest: &PersistedPhysicalRecoveryManifest) {
    let RecordArtifactFile::ExtentArena { arena } = manifest.artifact() else {
        unreachable!("admitted extent manifest has an arena coordinate")
    };
    target.extend_from_slice(&arena.to_le_bytes());
    target.extend_from_slice(&manifest.coordinate().offset().to_le_bytes());
    target.extend_from_slice(&manifest.coordinate().length().to_le_bytes());
    field(target, manifest.bytes());
}

pub(super) fn read_manifest(
    bytes: &[u8],
) -> Result<PersistedPhysicalRecoveryManifest, PhysicalRecoveryProjectionDenial> {
    let mut cursor = Cursor::new(bytes);
    let artifact = RecordArtifactFile::ExtentArena {
        arena: cursor.u64()?,
    };
    let coordinate = RecordFrameCoordinate::new(artifact, cursor.u64()?, cursor.u32()?)
        .ok_or(PhysicalRecoveryProjectionDenial::InvalidManifest)?;
    let payload = cursor.field()?;
    cursor.end()?;
    PersistedPhysicalRecoveryManifest::new(coordinate, payload)
        .ok_or(PhysicalRecoveryProjectionDenial::InvalidManifest)
}
