use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, CheckpointDirtyFrameBasis};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCheckpointDirtyBasis, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::record_rejection::checkpoint_record_denial;

#[derive(Debug)]
pub enum CheckpointDirtyBasisIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointDirtyBasis<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_dirty_basis<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CheckpointDirtyBasisIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis;
    if scope.artifact_family() != family {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let basis = match CheckpointDirtyFrameBasis::decode_record(artifact.bytes()) {
        Ok(basis) => basis,
        Err(denial) => {
            return rejected(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    let validated = IntegrityValidatedCheckpointDirtyBasis::new(
        scope,
        basis,
        durable_artifact_checksum(artifact.bytes()),
        artifact,
    )
    .expect("validated dirty-basis record satisfies its sealed-view contract");
    (
        CheckpointDirtyBasisIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointDirtyBasisIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CheckpointDirtyBasisIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CheckpointDirtyBasis,
            byte_count,
            rejection,
        ),
    )
}
