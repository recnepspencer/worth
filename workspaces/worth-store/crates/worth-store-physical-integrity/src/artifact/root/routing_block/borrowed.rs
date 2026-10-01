//! Borrowed route validation shares Format grammar and root-scope predicates.
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    durable_artifact_checksum, maximum_current_root_entries, BoundedRootRoutingBlockDecodeDenial,
    RootRoutingBlockDecodeLimits, RootRoutingBlockPreflight, RootRoutingCoordinateKey,
};

use super::super::routing_block_rejection::{
    format_mismatch, recursive_checksum_mismatch, routing_denial, scope_mismatch,
};
use crate::artifact::durable_frame_rejection::{input_length, wrong_scope};
use crate::observation::PhysicalIntegrityObservationCounters;
use crate::validation::{
    IntegrityValidatedRootRoutingBlockView, PhysicalArtifactScope, PhysicalIntegrityRejection,
    UntrustedPhysicalArtifact,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootRoutingCoordinateScratchDenial {
    pub required: usize,
    pub provided: usize,
}

#[derive(Debug)]
pub enum BorrowedRootRoutingBlockIntegrityValidation<'media> {
    Intact(IntegrityValidatedRootRoutingBlockView<'media>),
    Rejected(PhysicalIntegrityRejection),
}

pub fn validate_root_routing_block_borrowed<'media>(
    artifact: UntrustedPhysicalArtifact<'media>,
    scope: PhysicalArtifactScope,
    scratch: &mut Vec<RootRoutingCoordinateKey>,
) -> Result<
    (
        BorrowedRootRoutingBlockIntegrityValidation<'media>,
        PhysicalIntegrityObservationCounters,
    ),
    RootRoutingCoordinateScratchDenial,
> {
    let byte_count = artifact.byte_count();
    if scope.artifact_family() != PhysicalIntegrityArtifactFamily::RootRoutingBlock {
        return Ok(rejected(wrong_scope(scope), byte_count));
    }
    if let Some(rejection) = input_length(scope, byte_count) {
        return Ok(rejected(rejection, byte_count));
    }
    let capacity = maximum_current_root_entries(scope.record_format());
    let limits = RootRoutingBlockDecodeLimits {
        leaf_entries: u64::from(capacity),
        branch_children: u64::from(capacity),
    };
    let (preflight, record_format) =
        match RootRoutingBlockPreflight::inspect_frame(artifact.bytes(), capacity, limits) {
            Ok(value) => value,
            Err(denial) => {
                return Ok(rejected(
                    routing_denial(scope, artifact.bytes(), denial),
                    byte_count,
                ))
            }
        };
    let block = match preflight.validate(scratch) {
        Ok(block) => block,
        Err(BoundedRootRoutingBlockDecodeDenial::CoordinateScratchInsufficient {
            required,
            provided,
        }) => return Err(RootRoutingCoordinateScratchDenial { required, provided }),
        Err(denial) => {
            return Ok(rejected(
                routing_denial(scope, artifact.bytes(), denial),
                byte_count,
            ))
        }
    };
    if record_format != scope.record_format() {
        return Ok(rejected(format_mismatch(scope), byte_count));
    }
    if let Some(rejection) = scope_mismatch(scope, &block) {
        return Ok(rejected(rejection, byte_count));
    }
    let byte_range_checksum = durable_artifact_checksum(artifact.bytes());
    let expected = scope
        .root_routing_block_identity()
        .expect("root-routing scope carries identity");
    if byte_range_checksum != expected.reference().checksum() {
        return Ok(rejected(recursive_checksum_mismatch(scope), byte_count));
    }
    let validated = IntegrityValidatedRootRoutingBlockView::new(
        scope,
        block,
        record_format,
        byte_range_checksum,
        artifact,
    )
    .expect("validated borrowed root-routing block satisfies the sealed-view contract");
    Ok((
        BorrowedRootRoutingBlockIntegrityValidation::Intact(validated),
        PhysicalIntegrityObservationCounters::one_intact(
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            byte_count,
        ),
    ))
}

fn rejected<'media>(
    rejection: PhysicalIntegrityRejection,
    byte_count: u64,
) -> (
    BorrowedRootRoutingBlockIntegrityValidation<'media>,
    PhysicalIntegrityObservationCounters,
) {
    (
        BorrowedRootRoutingBlockIntegrityValidation::Rejected(rejection),
        PhysicalIntegrityObservationCounters::one_rejected(
            PhysicalIntegrityArtifactFamily::RootRoutingBlock,
            byte_count,
            rejection,
        ),
    )
}
