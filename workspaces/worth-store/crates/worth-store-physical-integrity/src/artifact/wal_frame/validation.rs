use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::wal_frame::decode_bounded_wal_frame_v1;

use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedWalFrame, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::rejection::{from_bounded_denial, input_length, scope_identity_mismatch, wrong_scope};

#[derive(Debug)]
pub enum WalFrameIntegrityValidation<'media> {
    Intact(IntegrityValidatedWalFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_wal_frame<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    WalFrameIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::WalFrame {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let decoded = match decode_bounded_wal_frame_v1(artifact.bytes()) {
        Ok(decoded) => decoded,
        Err(denial) => return rejected(from_bounded_denial(scope, denial), byte_count),
    };
    let expected_identity = scope
        .wal_segment_identity()
        .expect("WAL family scope carries WAL segment identity");
    if decoded.header().identity() != expected_identity {
        return rejected(
            scope_identity_mismatch(scope, decoded.header().identity()),
            byte_count,
        );
    }
    let validated = IntegrityValidatedWalFrame::new(scope, decoded, artifact)
        .expect("bounded WAL decode and exact scope satisfy the sealed-view contract");
    (
        WalFrameIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::WalFrame,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    WalFrameIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        WalFrameIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::WalFrame,
            byte_count,
            rejection,
        ),
    )
}
