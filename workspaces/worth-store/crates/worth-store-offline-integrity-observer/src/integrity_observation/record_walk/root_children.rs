//! Root references that seed the independent bounded media walk.

use super::super::families::{
    durable_frame::{read_u16, read_u32, read_u64},
    root_manifest::OfflineRootManifestFacts,
    tree_reference::reference,
};
use super::{ChildExpectation, ChildScope};
use worth_foundational::PhysicalArtifactFamily as Family;

pub(crate) fn root_children(root: &OfflineRootManifestFacts) -> Vec<ChildExpectation> {
    let payload = &root.payload;
    let tree = read_u64(payload, 8);
    let capacity = read_u16(payload, 16);
    let mut children = Vec::new();
    if payload[40] == 1 {
        children.push(reference(
            &payload[48..120],
            Family::RootRoutingBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    if payload[160] == 1 {
        children.push(reference(
            &payload[168..224],
            Family::SegmentMembershipBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    children.push(ChildExpectation {
        path: format!(
            "families/records/free-space/free-space-{:016x}.manifest",
            root.generation
        ),
        family: Family::FreeSpaceHeader,
        generation: root.generation,
        format: root.format,
        offset: 0,
        length: Some(216),
        checksum: Some(read_u32(payload, 152)),
        scope: ChildScope::FreeSpace { tree, capacity },
    });
    if payload[232] == 1 {
        children.push(reference(
            &payload[240..312],
            Family::FreeSpaceMembershipBlock,
            tree,
            capacity,
            root.format,
        ));
    }
    children
}
