use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalFreeSpaceMembershipBlock,
    PhysicalRecordFormatDeclaration, PhysicalSegmentMembershipBlock, PhysicalTreeIdentity,
    RecordArtifactFile, RecordFreeSpaceManifestEntry, SegmentManifestBlockReference,
};

use super::manifest_entry_budget::ManifestEntryBudget;
use super::page_observation::{required_source, PageObservationFailure};
use crate::integrity_ingress::projection::MembershipProjectionFailure;
use crate::progression::{RecoverySelectedSegmentPage, RecoverySelectedSourceInventory};

type SelectedSegmentTopologyObservation = (
    BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
    BTreeSet<RecordArtifactFile>,
    BTreeMap<(u64, u64), PhysicalSegmentMembershipBlock>,
);

#[cfg(test)]
pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    maximum_manifest_entries: u64,
    byte_limit: u64,
) -> (
    Result<RecoverySelectedSourceInventory, PageObservationFailure>,
    crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) {
    let mut budget = ManifestEntryBudget::new(maximum_manifest_entries, 0);
    let mut integrity_trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::default();
    let inventory = observe_with_budget(
        discovery,
        root,
        format,
        &mut budget,
        byte_limit,
        &mut integrity_trace,
    );
    (inventory, integrity_trace)
}

pub(super) fn observe_with_budget(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<RecoverySelectedSourceInventory, PageObservationFailure> {
    budget.admit_pending_block_read()?;
    let free_space = read_free_space_header(discovery, root, format, byte_limit, integrity_trace)?;
    let (segment_pages, segment_artifacts, segment_topology) =
        read_segment_pages(discovery, root, format, budget, byte_limit, integrity_trace)?;
    let (free_entries, free_artifacts, free_topology) = read_free_entries(
        discovery,
        &free_space,
        format,
        budget,
        byte_limit,
        integrity_trace,
    )?;
    let mut source_artifacts = BTreeSet::from([RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    }]);
    source_artifacts.extend(segment_artifacts);
    source_artifacts.extend(free_artifacts);
    Ok(RecoverySelectedSourceInventory {
        free_space,
        segment_pages,
        segment_topology,
        free_entries: free_entries.into_boxed_slice(),
        free_topology,
        source_artifacts: source_artifacts
            .into_iter()
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    })
}

fn read_free_space_header(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    byte_limit: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<DurableFreeSpaceManifestHeader, PageObservationFailure> {
    let artifact = RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    };
    let source = required_source(
        discovery.read_free_space_manifest(root.generation(), byte_limit),
        None,
        artifact,
    )?;
    crate::integrity_ingress::projection::free_space_header(
        &source,
        discovery.store_identity(),
        format,
        root,
        integrity_trace,
    )
    .map_err(|rejection| PageObservationFailure::Integrity {
        artifact,
        denial: rejection.diagnostic(),
    })
}

fn read_segment_pages(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<SelectedSegmentTopologyObservation, PageObservationFailure> {
    let mut pending = root.segment_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    let mut pages = BTreeMap::new();
    let mut topology = BTreeMap::new();
    while let Some(reference) = pending.pop_front() {
        budget.admit_pending_block_read()?;
        let artifact = RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        artifacts.insert(artifact);
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact));
        }
        let source = required_source(
            discovery.read_segment_membership_block(
                reference.generation(),
                reference.block(),
                byte_limit,
            ),
            None,
            artifact,
        )?;
        let tree =
            PhysicalTreeIdentity::new(root.tree_identity()).ok_or_else(|| invalid(artifact))?;
        let block = crate::integrity_ingress::projection::segment_membership_block(
            &source,
            discovery.store_identity(),
            format,
            tree,
            reference,
            root.node_capacity(),
            budget.remaining(),
            integrity_trace,
        )
        .map_err(|failure| membership_failure(artifact, failure))?;
        topology.insert((reference.generation(), reference.block()), block.clone());
        if let Some(entries) = block.entries() {
            budget.consume(entries.len())?;
            for entry in entries {
                let key = (entry.page_cell().segment_id().get(), entry.page().get());
                let page = RecoverySelectedSegmentPage {
                    entry: *entry,
                    routing_identity: routing_identity(root, format, reference, *entry),
                    membership_artifact: artifact,
                };
                if pages.insert(key, page).is_some() {
                    return Err(invalid(artifact));
                }
            }
        } else if let Some(children) = block.children() {
            budget.consume(children.len())?;
            pending.extend(children.iter().copied());
        }
    }
    Ok((pages, artifacts, topology))
}

fn read_free_entries(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    header: &DurableFreeSpaceManifestHeader,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<
    (
        Vec<RecordFreeSpaceManifestEntry>,
        BTreeSet<RecordArtifactFile>,
        BTreeMap<(u64, u64), PhysicalFreeSpaceMembershipBlock>,
    ),
    PageObservationFailure,
> {
    let mut pending = header.root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    let mut entries = Vec::new();
    let mut topology = BTreeMap::new();
    while let Some(reference) = pending.pop_front() {
        budget.admit_pending_block_read()?;
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        artifacts.insert(artifact);
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact));
        }
        let source = required_source(
            discovery.read_free_space_membership_block(
                reference.generation(),
                reference.block(),
                byte_limit,
            ),
            None,
            artifact,
        )?;
        let tree =
            PhysicalTreeIdentity::new(header.tree_identity()).ok_or_else(|| invalid(artifact))?;
        let block = crate::integrity_ingress::projection::free_space_membership_block(
            &source,
            discovery.store_identity(),
            format,
            tree,
            reference,
            header.node_capacity(),
            budget.remaining(),
            integrity_trace,
        )
        .map_err(|failure| membership_failure(artifact, failure))?;
        topology.insert((reference.generation(), reference.block()), block.clone());
        if let Some(found) = block.entries() {
            budget.consume(found.len())?;
            entries.extend_from_slice(found);
        } else if let Some(children) = block.children() {
            budget.consume(children.len())?;
            pending.extend(children.iter().copied());
        }
    }
    entries.sort_unstable_by_key(|entry| (entry.class() as u8, entry.owner()));
    if entries
        .windows(2)
        .any(|pair| pair[0].class() == pair[1].class() && pair[0].owner() == pair[1].owner())
    {
        return Err(invalid(RecordArtifactFile::FreeSpaceManifest {
            generation: header.generation(),
        }));
    }
    Ok((entries, artifacts, topology))
}

fn routing_identity(
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    reference: SegmentManifestBlockReference,
    entry: worth_store_physical_format::RecordSegmentPageManifestEntry,
) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"worth.store.recovery.segment-page-routing.v1");
    digest.update(root.encode(format));
    digest.update(reference.generation().to_le_bytes());
    digest.update(reference.block().to_le_bytes());
    digest.update(reference.level().to_le_bytes());
    digest.update(reference.checksum().to_le_bytes());
    digest.update(reference.first().segment().get().to_le_bytes());
    digest.update(reference.first().page().get().to_le_bytes());
    digest.update(reference.last().segment().get().to_le_bytes());
    digest.update(reference.last().page().get().to_le_bytes());
    digest.update(entry.page_cell().segment_id().get().to_le_bytes());
    digest.update(entry.page().get().to_le_bytes());
    digest.update(entry.page_generation().to_le_bytes());
    digest.update(entry.data_generation().to_le_bytes());
    digest.update(entry.data_page_count().to_le_bytes());
    digest.update(entry.frame_index().to_le_bytes());
    digest.finalize().into()
}

const fn invalid(artifact: RecordArtifactFile) -> PageObservationFailure {
    PageObservationFailure::InvalidManifest {
        target: None,
        artifact,
    }
}

fn membership_failure(
    artifact: RecordArtifactFile,
    failure: MembershipProjectionFailure,
) -> PageObservationFailure {
    match failure {
        MembershipProjectionFailure::EntryLimit { .. } => {
            PageObservationFailure::ManifestEntryLimit
        }
        MembershipProjectionFailure::Integrity(rejection) => PageObservationFailure::Integrity {
            artifact,
            denial: rejection.diagnostic(),
        },
    }
}
