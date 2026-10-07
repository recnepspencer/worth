//! Format-owned canonical encoders applied to owner-reserved frame backing.

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    PhysicalFreeSpaceMembershipBlock, PhysicalRecordFormatDeclaration, PhysicalRootRoutingBlock,
    PhysicalSegmentMembershipBlock, RootRoutingCoordinateKey,
};

use super::CandidateBuildDenial;
use crate::progression::planned::PlanningResidentAllowance;

pub(super) fn root_leaf(
    tree: u64,
    generation: u64,
    block: u64,
    entries: Vec<CurrentPhysicalRecordPlacement>,
    capacity: u16,
    allowance: &mut PlanningResidentAllowance,
) -> Result<PhysicalRootRoutingBlock, CandidateBuildDenial> {
    let mut scratch = allowance.reserve::<RootRoutingCoordinateKey>(entries.len())?;
    let result = PhysicalRootRoutingBlock::leaf_with_uniqueness_scratch(
        tree,
        generation,
        block,
        entries,
        capacity,
        &mut scratch,
    )
    .ok_or(CandidateBuildDenial::Invalid);
    let bytes = PlanningResidentAllowance::vector_bytes(&scratch)?;
    drop(scratch);
    allowance.release(bytes);
    result
}

pub(super) fn root_block(
    block: &PhysicalRootRoutingBlock,
    format: PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<u8>, CandidateBuildDenial> {
    let length = block
        .encoded_frame_bytes()
        .ok_or(CandidateBuildDenial::Invalid)?;
    let frame = allowance.reserve::<u8>(length)?;
    block
        .encode_in_reserved(format, frame)
        .ok_or(CandidateBuildDenial::Invalid)
}

pub(super) fn segment_block(
    block: &PhysicalSegmentMembershipBlock,
    format: PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<u8>, CandidateBuildDenial> {
    let length = block
        .encoded_frame_bytes()
        .ok_or(CandidateBuildDenial::Invalid)?;
    let frame = allowance.reserve::<u8>(length)?;
    block
        .encode_in_reserved(format, frame)
        .ok_or(CandidateBuildDenial::Invalid)
}

pub(super) fn free_block(
    block: &PhysicalFreeSpaceMembershipBlock,
    format: PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<u8>, CandidateBuildDenial> {
    let length = block
        .encoded_frame_bytes()
        .ok_or(CandidateBuildDenial::Invalid)?;
    let frame = allowance.reserve::<u8>(length)?;
    block
        .encode_in_reserved(format, frame)
        .ok_or(CandidateBuildDenial::Invalid)
}

pub(in crate::progression::planned::basis) fn root_manifest(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<u8>, CandidateBuildDenial> {
    let frame = allowance.reserve::<u8>(root.encoded_frame_bytes())?;
    root.encode_in_reserved(format, frame)
        .ok_or(CandidateBuildDenial::Invalid)
}

pub(in crate::progression::planned::basis) fn free_header(
    header: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<u8>, CandidateBuildDenial> {
    let frame = allowance.reserve::<u8>(header.encoded_frame_bytes())?;
    header
        .encode_in_reserved(format, frame)
        .ok_or(CandidateBuildDenial::Invalid)
}
