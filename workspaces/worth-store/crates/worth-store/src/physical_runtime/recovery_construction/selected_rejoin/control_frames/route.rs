//! C.9-validated selected routing path for one release control record.

use worth_store_physical_backend::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, PersistedRecordIdentity, PhysicalRecordFormatDeclaration,
    PhysicalTreeIdentity, RecordArtifactFile, RootRoutingBlockScopeIdentity,
};
use worth_store_physical_integrity::{
    validate_root_routing_block, PhysicalArtifactScope, PhysicalByteRange,
    RootRoutingBlockIntegrityValidation, UntrustedPhysicalArtifact,
};

use super::super::SelectedMediaRejoinDenial as Denial;
use super::{super::root_checkpoint::ObservedRootCheckpoint, snapshot::SelectedArtifactSlice};

pub(super) fn selected_route(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selected: &ObservedRootCheckpoint,
    format: PhysicalRecordFormatDeclaration,
    record: PersistedRecordIdentity,
    slices: &mut Vec<SelectedArtifactSlice>,
) -> Result<CurrentPhysicalRecordPlacement, Denial> {
    let mut reference = selected.root().routing_root().ok_or(Denial::MissingRoute)?;
    let tree =
        PhysicalTreeIdentity::new(selected.root().tree_identity()).ok_or(Denial::RootBinding)?;
    if !reference.contains(record) {
        return Err(Denial::MissingRoute);
    }
    loop {
        let frame = discovery
            .read_root_routing_block(
                reference.generation(),
                reference.block(),
                u64::from(format.page_size().bytes()),
            )
            .map_err(Denial::Discovery)?;
        let bytes = frame.bytes().ok_or(Denial::MissingRoute)?;
        slices.push(
            SelectedArtifactSlice::observed(
                RecordArtifactFile::RootRoutingBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
                0,
                bytes,
                true,
            )
            .ok_or(Denial::BoundExceeded)?,
        );
        let range =
            PhysicalByteRange::new(0, bytes.len() as u64).map_err(|_| Denial::RoutingFrame)?;
        let scope = PhysicalArtifactScope::root_routing_block(
            discovery.store_identity(),
            format,
            RootRoutingBlockScopeIdentity::new(tree, reference),
            range,
        );
        let (validated, _) = validate_root_routing_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        );
        let RootRoutingBlockIntegrityValidation::Intact(block) = validated else {
            return Err(Denial::RoutingFrame);
        };
        if let Some(entries) = block.entries() {
            if entries.len() > usize::from(selected.root().node_capacity()) {
                return Err(Denial::RoutingFrame);
            }
            return entries
                .binary_search_by_key(&record, |entry| entry.record())
                .ok()
                .map(|index| entries[index])
                .ok_or(Denial::MissingRoute);
        }
        let children = block.children().ok_or(Denial::RoutingFrame)?;
        if children.len() > usize::from(selected.root().node_capacity()) {
            return Err(Denial::RoutingFrame);
        }
        let next = children
            .iter()
            .copied()
            .find(|child| child.contains(record))
            .ok_or(Denial::MissingRoute)?;
        if next.level().checked_add(1) != Some(reference.level())
            || next.generation() > reference.generation()
        {
            return Err(Denial::RoutingFrame);
        }
        reference = next;
    }
}
