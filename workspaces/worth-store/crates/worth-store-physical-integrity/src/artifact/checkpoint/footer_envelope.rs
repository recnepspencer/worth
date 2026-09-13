use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::CheckpointStreamFooter;

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedCheckpointFooterEnvelope, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::footer::{identity_mismatch, rejected_footer};
use super::record_rejection::checkpoint_record_denial;

#[derive(Debug)]
pub enum CheckpointFooterEnvelopeIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointFooterEnvelope<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_footer_envelope<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CheckpointFooterEnvelopeIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointFooter;
    if scope.artifact_family() != family {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let footer = match CheckpointStreamFooter::decode_record(artifact.bytes()) {
        Ok(footer) => footer,
        Err(denial) => {
            return rejected(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    if let Some(rejection) = identity_mismatch(scope, footer) {
        return rejected(rejection, byte_count);
    }
    let validated = IntegrityValidatedCheckpointFooterEnvelope::new(scope, footer, artifact)
        .expect("validated footer envelope satisfies its sealed-view contract");
    (
        CheckpointFooterEnvelopeIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointFooterEnvelopeIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let (_, counters) = rejected_footer(rejection, byte_count);
    (
        CheckpointFooterEnvelopeIntegrityValidation::Rejected(rejection),
        counters,
    )
}
