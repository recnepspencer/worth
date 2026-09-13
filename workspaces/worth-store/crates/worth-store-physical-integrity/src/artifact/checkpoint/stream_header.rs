use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{durable_artifact_checksum, PhysicalCheckpointSource};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::localization::{PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    CheckpointStreamHeaderScopeIdentity, IntegrityValidatedCheckpointStreamHeader,
    PhysicalArtifactScope, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use super::record_rejection::{checkpoint_record_denial, field_damage, CheckpointRecordFieldRange};

const STORE_IDENTITY_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(16, 16);
const SEQUENCE_IDENTITY_FIELD: CheckpointRecordFieldRange = CheckpointRecordFieldRange::new(32, 8);

#[derive(Debug)]
pub enum CheckpointStreamHeaderIntegrityValidation<'media> {
    Intact(IntegrityValidatedCheckpointStreamHeader<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_checkpoint_stream_header<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    CheckpointStreamHeaderIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    let family = PhysicalIntegrityArtifactFamily::CheckpointStreamHeader;
    if scope.artifact_family() != family {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let source = match PhysicalCheckpointSource::decode_stream_header_record(artifact.bytes()) {
        Ok(source) => source,
        Err(denial) => {
            return rejected(
                checkpoint_record_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    if let Some(rejection) = identity_mismatch(scope, source) {
        return rejected(rejection, byte_count);
    }
    let validated = IntegrityValidatedCheckpointStreamHeader::new(
        scope,
        source,
        durable_artifact_checksum(artifact.bytes()),
        artifact,
    )
    .expect("validated checkpoint header satisfies its sealed-view contract");
    (
        CheckpointStreamHeaderIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(family, byte_count),
    )
}

fn identity_mismatch(
    scope: PhysicalArtifactScope,
    source: PhysicalCheckpointSource,
) -> Option<PhysicalIntegrityRejection> {
    let expected = scope
        .checkpoint_stream_header_identity()
        .expect("checkpoint-header scope carries its staged or known identity");
    if source.identity().store_identity() != expected.store_identity() {
        return Some(field_damage(
            scope,
            PhysicalDamageCause::StoreIdentityMismatch,
            STORE_IDENTITY_FIELD,
            PhysicalFormatField::StoreIdentity,
            PhysicalBlastRadius::CompleteArtifact,
        ));
    }
    match expected {
        CheckpointStreamHeaderScopeIdentity::Known(identity) if source.identity() != identity => {
            Some(field_damage(
                scope,
                PhysicalDamageCause::SequenceMismatch,
                SEQUENCE_IDENTITY_FIELD,
                PhysicalFormatField::CheckpointIdentity,
                PhysicalBlastRadius::CompleteArtifact,
            ))
        }
        _ => None,
    }
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    CheckpointStreamHeaderIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        CheckpointStreamHeaderIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::CheckpointStreamHeader,
            byte_count,
            rejection,
        ),
    )
}
