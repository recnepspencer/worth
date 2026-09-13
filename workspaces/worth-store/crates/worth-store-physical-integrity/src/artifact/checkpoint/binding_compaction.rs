use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, CheckpointBindingCompactionHeader};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCheckpointBindingCompaction, PhysicalArtifactScope,
    PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use super::record_rejection::checkpoint_record_denial;

#[derive(Debug)]
pub enum CheckpointBindingCompactionIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointBindingCompaction<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_binding_compaction<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CheckpointBindingCompactionIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction;
    if scope.artifact_family() != family {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let header = match CheckpointBindingCompactionHeader::decode_record(artifact.bytes()) {
        Ok(header) => header,
        Err(denial) => {
            return rejected(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    let validated = IntegrityValidatedCheckpointBindingCompaction::new(
        scope,
        header,
        durable_artifact_checksum(artifact.bytes()),
        artifact,
    )
    .expect("validated binding-compaction record satisfies its sealed-view contract");
    (
        CheckpointBindingCompactionIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointBindingCompactionIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CheckpointBindingCompactionIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CheckpointBindingCompaction,
            byte_count,
            rejection,
        ),
    )
}
