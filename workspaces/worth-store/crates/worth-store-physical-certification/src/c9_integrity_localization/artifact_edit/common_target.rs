use super::{ArtifactInventory,ArtifactOperator as Operator,common_render,enclosures};
use std::path::Path;
use worth_store::physical_runtime::{PhysicalIntegrityScrubTarget};
use worth_store_physical_format::*;
use worth_store_physical_integrity::PhysicalArtifactScope;

pub(in super::super) fn common_inspection_target(baseline:&Path,inventory:&ArtifactInventory,index:usize,operator:Operator)->PhysicalIntegrityScrubTarget {
    let target=&inventory.granules[index];
    if !matches!(operator,Operator::Checksum|Operator::Length|Operator::ScopeSubstitution|Operator::Pointer|Operator::EnvelopeVersion|Operator::RecordVersion) {
        return target.scrub_target();
    }
    // This checksum is derived before the editor from the declared clean-world
    // mutation. Expected identity/generation/range never come from damaged media.
    let checksum=enclosures::complete_checksum(&common_render(baseline,inventory,index,operator));
    let scope=target.scope;
    let range=scope.byte_range();
    let revised=if let Some(identity)=scope.root_routing_block_identity() {
        let r=identity.reference();
        PhysicalArtifactScope::root_routing_block(inventory.store,inventory.format,
            RootRoutingBlockScopeIdentity::new(identity.tree(),ManifestBlockReference::new(r.generation(),r.block(),r.level(),checksum,r.first(),r.last()).unwrap()),range)
    } else if let Some(identity)=scope.segment_membership_block_identity() {
        let r=identity.reference();
        PhysicalArtifactScope::segment_membership_block(inventory.store,inventory.format,
            SegmentMembershipBlockScopeIdentity::new(identity.tree(),SegmentManifestBlockReference::new(r.generation(),r.block(),r.level(),checksum,r.first(),r.last()).unwrap()),range)
    } else if let Some(identity)=scope.free_space_membership_block_identity() {
        let r=identity.reference();
        PhysicalArtifactScope::free_space_membership_block(inventory.store,inventory.format,
            FreeSpaceMembershipBlockScopeIdentity::new(identity.tree(),FreeSpaceBlockReference::new(r.generation(),r.block(),r.level(),checksum,r.first(),r.last()).unwrap()),range)
    } else if let Some(identity)=scope.free_space_header_identity() {
        PhysicalArtifactScope::free_space_header(inventory.store,inventory.format,
            FreeSpaceHeaderScopeIdentity::new(identity.generation(),identity.tree(),identity.root(),DurableArtifactCrc32c::new(checksum)),range)
    } else {scope};
    PhysicalIntegrityScrubTarget::new(target.target,revised).unwrap()
}
