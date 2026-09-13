mod free_space;
mod membership_budget;
mod root_routing;
mod segment_membership;

pub(crate) use free_space::{free_space_header, free_space_membership_block};
pub(crate) use membership_budget::MembershipProjectionFailure;
pub(crate) use root_routing::{root_routing_block, AdmittedRootRoutingProjection};
pub(crate) use segment_membership::segment_membership_block;

use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_integrity::{PhysicalByteRange, UntrustedPhysicalArtifact};

use super::RecoveryIntegrityIngressRejection;

fn source_input(
    source: &ObservedRecoveryArtifact,
) -> Result<UntrustedPhysicalArtifact<'_>, RecoveryIntegrityIngressRejection> {
    source
        .bytes()
        .map(UntrustedPhysicalArtifact::from_bounded_bytes)
        .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)
}

fn source_range(
    source: &ObservedRecoveryArtifact,
) -> Result<PhysicalByteRange, RecoveryIntegrityIngressRejection> {
    let length = source
        .bytes()
        .map(|bytes| bytes.len() as u64)
        .ok_or(RecoveryIntegrityIngressRejection::MissingBoundedArtifact)?;
    PhysicalByteRange::new(0, length)
        .map_err(|_| RecoveryIntegrityIngressRejection::SourceRangeOutsideObservation)
}

fn expected_range(
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> PhysicalByteRange {
    PhysicalByteRange::new(0, u64::from(format.page_size().bytes()))
        .expect("an admitted physical page size is nonzero")
}
