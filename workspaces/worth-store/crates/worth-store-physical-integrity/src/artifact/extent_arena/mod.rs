use crate::{
    validate_extent_chunk_membership, validate_extent_manifest, ExtentChunkIntegrityValidation,
    ExtentManifestIntegrityValidation, IntegrityValidatedExtentChunkFrame,
    IntegrityValidatedExtentManifest, IntegrityValidatedExtentMembership, PhysicalArtifactScope,
    PhysicalIntegrityObservationCounters, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;

/// The root supplies manifest placement; only a validated manifest supplies chunk membership.
#[derive(Debug, Clone, Copy)]
pub enum ExtentArenaFrameExpectation {
    Manifest(PhysicalArtifactScope),
    Chunk {
        scope: PhysicalArtifactScope,
        membership: IntegrityValidatedExtentMembership,
    },
}

#[derive(Debug)]
pub enum ExtentArenaFrameIntegrityValidation<'media> {
    Manifest(IntegrityValidatedExtentManifest<'media>),
    Chunk(IntegrityValidatedExtentChunkFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

/// Validates one encoded arena frame at its absolute media offset. Padding belongs
/// to the allocated range, never to a frame's checksummed length or decoded payload.
pub fn validate_extent_arena_frame<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    expectation: ExtentArenaFrameExpectation,
) -> (
    ExtentArenaFrameIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let length = artifact.byte_count();
    let result = match expectation {
        ExtentArenaFrameExpectation::Manifest(scope) => {
            match validate_extent_manifest(artifact, scope).0 {
                ExtentManifestIntegrityValidation::Intact(value) => {
                    ExtentArenaFrameIntegrityValidation::Manifest(value)
                }
                ExtentManifestIntegrityValidation::Rejected(error) => {
                    ExtentArenaFrameIntegrityValidation::Rejected(error)
                }
            }
        }
        ExtentArenaFrameExpectation::Chunk { scope, membership } => {
            match validate_extent_chunk_membership(artifact, scope, membership).0 {
                ExtentChunkIntegrityValidation::Intact(value) => {
                    ExtentArenaFrameIntegrityValidation::Chunk(value)
                }
                ExtentChunkIntegrityValidation::Rejected(error) => {
                    ExtentArenaFrameIntegrityValidation::Rejected(error)
                }
            }
        }
    };
    let counters = match &result {
        ExtentArenaFrameIntegrityValidation::Rejected(error) => {
            PhysicalIntegrityObservationCounters::one_rejected(
                PhysicalIntegrityArtifactFamily::ExtentArenaFrame,
                length,
                *error,
            )
        }
        _ => PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::ExtentArenaFrame,
            length,
        ),
    };
    (result, counters)
}

#[cfg(test)]
mod tests;
