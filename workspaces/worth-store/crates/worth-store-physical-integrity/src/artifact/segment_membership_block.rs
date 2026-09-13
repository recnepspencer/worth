use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_segment_manifest_pages, PhysicalSegmentMembershipBlock,
    SegmentMembershipBlockDecodeLimits,
};

use super::segment_membership_block_rejection::{
    format_mismatch, membership_denial, recursive_checksum_mismatch, scope_mismatch,
};
use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedSegmentMembershipBlock, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub enum SegmentMembershipBlockIntegrityValidation<'media> {
    Intact(IntegrityValidatedSegmentMembershipBlock<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_segment_membership_block<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    SegmentMembershipBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::SegmentMembership {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let capacity = u16::try_from(maximum_segment_manifest_pages(scope.record_format()))
        .expect("supported page sizes bound segment membership below u16::MAX");
    let limits = SegmentMembershipBlockDecodeLimits {
        leaf_entries: u64::from(capacity),
        branch_children: u64::from(capacity),
    };
    let (block, record_format) =
        match PhysicalSegmentMembershipBlock::decode_bounded(artifact.bytes(), capacity, limits) {
            Ok(decoded) => decoded,
            Err(denial) => {
                return rejected(
                    membership_denial(scope, artifact.bytes(), denial),
                    byte_count,
                )
            }
        };
    if record_format != scope.record_format() {
        return rejected(format_mismatch(scope), byte_count);
    }
    if let Some(rejection) = scope_mismatch(scope, &block) {
        return rejected(rejection, byte_count);
    }
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let expected = scope
        .segment_membership_block_identity()
        .expect("segment-membership scope carries identity");
    if byte_range_checksum != expected.reference().checksum() {
        return rejected(recursive_checksum_mismatch(scope), byte_count);
    }
    let validated = IntegrityValidatedSegmentMembershipBlock::new(
        scope,
        block,
        record_format,
        byte_range_checksum,
        artifact,
    )
    .expect("validated segment-membership block satisfies the sealed-view contract");
    (
        SegmentMembershipBlockIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::SegmentMembership,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    SegmentMembershipBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        SegmentMembershipBlockIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::SegmentMembership,
            byte_count,
            rejection,
        ),
    )
}
