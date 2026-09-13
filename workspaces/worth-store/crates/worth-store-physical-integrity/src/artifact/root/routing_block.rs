use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_current_root_entries, PhysicalRootRoutingBlock,
    RootRoutingBlockDecodeLimits,
};

use super::routing_block_rejection::{
    format_mismatch, recursive_checksum_mismatch, routing_denial, scope_mismatch,
};
use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedRootRoutingBlock, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

#[derive(Debug)]
pub enum RootRoutingBlockIntegrityValidation<'media> {
    Intact(IntegrityValidatedRootRoutingBlock<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_root_routing_block<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
) -> (
    RootRoutingBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::RootRoutingBlock {
        return rejected(wrong_scope(scope), byte_count);
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return rejected(rejection, byte_count);
    }
    let capacity = maximum_current_root_entries(scope.record_format());
    let limits = RootRoutingBlockDecodeLimits {
        leaf_entries: u64::from(capacity),
        branch_children: u64::from(capacity),
    };
    let (block, record_format) =
        match PhysicalRootRoutingBlock::decode_bounded(artifact.bytes(), capacity, limits) {
            Ok(decoded) => decoded,
            Err(denial) => {
                return rejected(routing_denial(scope, artifact.bytes(), denial), byte_count)
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
        .root_routing_block_identity()
        .expect("root-routing scope carries identity");
    if byte_range_checksum != expected.reference().checksum() {
        return rejected(recursive_checksum_mismatch(scope), byte_count);
    }
    let validated = IntegrityValidatedRootRoutingBlock::new(
        scope,
        block,
        record_format,
        byte_range_checksum,
        artifact,
    )
    .expect("validated root-routing block satisfies the sealed-view contract");
    (
        RootRoutingBlockIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            byte_count,
        ),
    )
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    RootRoutingBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        RootRoutingBlockIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            byte_count,
            rejection,
        ),
    )
}
