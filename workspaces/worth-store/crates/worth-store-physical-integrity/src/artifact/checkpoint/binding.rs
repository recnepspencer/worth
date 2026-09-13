use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{decode_checkpoint_binding_record, durable_artifact_checksum};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCheckpointBinding, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::record_rejection::checkpoint_record_denial;

#[derive(Debug)]
pub enum CheckpointBindingIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointBinding<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_binding<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CheckpointBindingIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointBinding;
    if scope.artifact_family() != family {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let payload = match decode_checkpoint_binding_record(artifact.bytes()) {
        Ok(payload) => payload,
        Err(denial) => {
            return rejected(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    let payload_bytes = u32::try_from(payload.len())
        .expect("bounded checkpoint binding payload length fits the format u32");
    let validated = IntegrityValidatedCheckpointBinding::new(
        scope,
        payload_bytes,
        durable_artifact_checksum(artifact.bytes()),
        artifact,
    )
    .expect("validated binding record satisfies its sealed-view contract");
    (
        CheckpointBindingIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointBindingIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CheckpointBindingIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CheckpointBinding,
            byte_count,
            rejection,
        ),
    )
}
