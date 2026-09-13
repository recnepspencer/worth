use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, inspect_inline_page, InlinePageGeometry,
};

use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedPageFrame, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

use super::denial_localization::{page_identity_mismatch, page_integrity_denial};

#[derive(Debug)]
pub enum InlinePageIntegrityValidation<'media> {
    Intact(IntegrityValidatedPageFrame<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_inline_page<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    InlinePageIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::PageFrame {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let geometry = match inspect_inline_page(scope.record_format(), artifact.bytes()) {
        Ok(geometry) => geometry,
        Err(denial) => {
            return rejected(
                page_integrity_denial(scope, artifact.bytes(), denial),
                byte_count,
            )
        }
    };
    if Some(geometry.page_cell()) != scope.page_identity() {
        return rejected(page_identity_mismatch(scope, geometry), byte_count);
    }
    intact(artifact, scope, geometry, byte_count)
}

fn intact<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    geometry: InlinePageGeometry,
    byte_count: u64,
) -> (
    InlinePageIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let validated =
        IntegrityValidatedPageFrame::new(scope, geometry, byte_range_checksum, artifact)
            .expect("validated inline page satisfies the sealed-view contract");
    (
        InlinePageIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::PageFrame,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    InlinePageIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        InlinePageIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::PageFrame,
            byte_count,
            rejection,
        ),
    )
}
