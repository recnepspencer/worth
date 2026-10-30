use worth_store_physical_integrity::{
    CheckpointStreamHeaderScopeIdentity, IntegrityValidatedCheckpointDirtyBasis,
    IntegrityValidatedExtentChunkFrame, IntegrityValidatedExtentManifest,
    IntegrityValidatedPhysicalWorkObligation, IntegrityValidatedRootManifest,
    PhysicalArtifactScope, PhysicalByteRange,
};

fn retarget(validated: IntegrityValidatedRootManifest<'_>, other_scope: PhysicalArtifactScope) {
    let _ = validated.with_scope(other_scope);
}

fn retarget_physical_work(
    validated: IntegrityValidatedPhysicalWorkObligation<'_>,
    other_scope: PhysicalArtifactScope,
) {
    let _ = validated.with_scope(other_scope);
}

fn invent_prevalidation_lsn(scope: PhysicalArtifactScope) {
    let _ = scope.with_wal_lsn_range(3, 4);
}

fn retarget_extent_manifest(
    validated: IntegrityValidatedExtentManifest<'_>,
    other_scope: PhysicalArtifactScope,
) {
    let _ = validated.with_scope(other_scope);
}

fn retarget_extent_chunk(
    validated: IntegrityValidatedExtentChunkFrame<'_>,
    other_scope: PhysicalArtifactScope,
) {
    let _ = validated.with_scope(other_scope);
}

fn retarget_checkpoint(
    validated: IntegrityValidatedCheckpointDirtyBasis<'_>,
    other_scope: PhysicalArtifactScope,
) {
    let _ = validated.with_scope(other_scope);
}

fn stage_later_record(identity: CheckpointStreamHeaderScopeIdentity, range: PhysicalByteRange) {
    let _ = PhysicalArtifactScope::checkpoint_dirty_basis(identity, range);
}

fn main() {}
