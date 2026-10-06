use std::collections::{BTreeMap, BTreeSet, VecDeque};

use worth_store::physical_runtime::{
    ArtifactCeiling, BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact, PageAddress,
    ReadGrant, RecoveryDiscoveryFailure, UnchargedRead,
};
use worth_store_physical_format::{
    DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest, PhysicalFreeSpaceMembershipBlock,
    PhysicalRecordFormatDeclaration, PhysicalSegmentMembershipBlock, PhysicalTreeIdentity,
    RecordArtifactFile, RecordFreeSpaceManifestEntry,
};

use super::manifest_entry_budget::{ManifestEntryBudget, RootUnit};
use super::page_observation::{required_source, PageObservationFailure};
use crate::integrity_ingress::projection::MembershipProjectionFailure;
use crate::progression::{RecoverySelectedSegmentPage, RecoverySelectedSourceInventory};

type SelectedSegmentTopologyObservation = (
    BTreeMap<(u64, u64), RecoverySelectedSegmentPage>,
    BTreeSet<RecordArtifactFile>,
    BTreeMap<(u64, u64), PhysicalSegmentMembershipBlock>,
);

#[path = "selected_source_inventory/routes.rs"]
mod routes;
pub(super) use routes::{observe_routes_held, observe_routes_with_budget, RoutesFailure};
#[path = "selected_source_inventory/canonical.rs"]
mod canonical;
#[path = "selected_source_inventory/resident.rs"]
mod resident;
use canonical::{canonical_free_entries, routing_identity};
pub(in crate::orchestration::planning) use resident::{ResidentAllowance, ResidentTraceDenial};

#[cfg(test)]
pub(super) fn observe(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    maximum_manifest_entries: u64,
) -> (
    Result<RecoverySelectedSourceInventory, PageObservationFailure>,
    crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) {
    let mut budget = ManifestEntryBudget::new(maximum_manifest_entries, 0);
    let mut integrity_trace = crate::integrity_ingress::RecoveryIntegrityIngressTrace::default();
    let inventory = budget
        .charge_root()
        .map_err(PageObservationFailure::from)
        .and_then(|root_unit| {
            observe_with_budget(
                discovery,
                root,
                format,
                &root_unit,
                &mut budget,
                &mut integrity_trace,
            )
        });
    (inventory, integrity_trace)
}

/// The segment pages and free entries under `root`, each leaf entry charged.
/// The root's own entry is charged where the root is read.
pub(super) fn observe_with_budget(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    root_unit: &RootUnit,
    budget: &mut ManifestEntryBudget,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<RecoverySelectedSourceInventory, PageObservationFailure> {
    observe_headers(discovery, root, format, root_unit, integrity_trace)?
        .observe_segments(discovery, root, format, budget, integrity_trace)?
        .observe_free_entries(discovery, root, format, budget, integrity_trace)
}

/// The first read of an inventory: the free-space header `root` names.
pub(super) fn observe_headers(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    _root_unit: &RootUnit,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<InventoryHeaders, PageObservationFailure> {
    read_free_space_header(discovery, root, format, integrity_trace)
        .map(|free_space| InventoryHeaders { free_space })
}

/// An inventory read as far as its headers, with no tree under them yet.
pub(super) struct InventoryHeaders {
    free_space: DurableFreeSpaceManifestHeader,
}

impl InventoryHeaders {
    pub(super) const fn free_space(&self) -> &DurableFreeSpaceManifestHeader {
        &self.free_space
    }

    /// Reads the segment tree, each page entry charged.
    pub(super) fn observe_segments(
        self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Result<InventorySegments, PageObservationFailure> {
        Ok(InventorySegments {
            segments: read_segment_pages(discovery, root, format, budget, integrity_trace)?,
            free_space: self.free_space,
        })
    }
}

/// An inventory read as far as its segment tree.
pub(super) struct InventorySegments {
    free_space: DurableFreeSpaceManifestHeader,
    segments: SelectedSegmentTopologyObservation,
}

impl InventorySegments {
    /// The segment pages read. No header counts them.
    pub(super) fn pages(&self) -> usize {
        self.segments.0.len()
    }

    /// Reads the free-space tree, each entry charged.
    pub(super) fn observe_free_entries(
        self,
        discovery: &mut BoundedRecoveryFilesystemDiscovery,
        root: &DurablePhysicalRootManifest,
        format: PhysicalRecordFormatDeclaration,
        budget: &mut ManifestEntryBudget,
        integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
    ) -> Result<RecoverySelectedSourceInventory, PageObservationFailure> {
        let (segment_pages, segment_artifacts, segment_topology) = self.segments;
        let (free_entries, free_artifacts, free_topology) =
            read_free_entries(discovery, &self.free_space, format, budget, integrity_trace)?;
        let mut source_artifacts = BTreeSet::from([RecordArtifactFile::FreeSpaceManifest {
            generation: root.generation(),
        }]);
        source_artifacts.extend(segment_artifacts);
        source_artifacts.extend(free_artifacts);
        Ok(RecoverySelectedSourceInventory {
            free_space: self.free_space,
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
}

/// One page of `format` at `address`, the read every page here makes. A
/// free-space manifest and a membership block are each at most one page, so
/// a larger one is damage: no budget of the caller's stands in for the
/// artifact's own ceiling.
fn read_page(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    format: PhysicalRecordFormatDeclaration,
    address: PageAddress,
) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
    discovery
        .read(
            ArtifactCeiling::page(format, address),
            ReadGrant::ceiling_only(),
        )
        .observed()
}

fn read_free_space_header(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    root: &DurablePhysicalRootManifest,
    format: PhysicalRecordFormatDeclaration,
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<DurableFreeSpaceManifestHeader, PageObservationFailure> {
    let artifact = RecordArtifactFile::FreeSpaceManifest {
        generation: root.generation(),
    };
    let source = required_source(
        read_page(
            discovery,
            format,
            PageAddress::FreeSpaceManifest {
                generation: root.generation(),
            },
        ),
        None,
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
    integrity_trace: &mut crate::integrity_ingress::RecoveryIntegrityIngressTrace,
) -> Result<SelectedSegmentTopologyObservation, PageObservationFailure> {
    let mut pending = root.segment_root().into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut artifacts = BTreeSet::new();
    let mut pages = BTreeMap::new();
    let mut topology = BTreeMap::new();
    while let Some(reference) = pending.pop_front() {
        let artifact = RecordArtifactFile::SegmentMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        artifacts.insert(artifact);
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact));
        }
        let source = required_source(
            read_page(
                discovery,
                format,
                PageAddress::SegmentMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
            ),
            None,
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
        .map_err(|failure| membership_failure(artifact, failure, budget))?;
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
        let artifact = RecordArtifactFile::FreeSpaceMembershipBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        artifacts.insert(artifact);
        if !visited.insert((reference.generation(), reference.block())) {
            return Err(invalid(artifact));
        }
        let source = required_source(
            read_page(
                discovery,
                format,
                PageAddress::FreeSpaceMembershipBlock {
                    generation: reference.generation(),
                    block: reference.block(),
                },
            ),
            None,
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
        .map_err(|failure| membership_failure(artifact, failure, budget))?;
        topology.insert((reference.generation(), reference.block()), block.clone());
        if let Some(found) = block.entries() {
            budget.consume(found.len())?;
            entries.extend_from_slice(found);
        } else if let Some(children) = block.children() {
            pending.extend(children.iter().copied());
        }
    }
    if !canonical_free_entries(&mut entries) {
        return Err(invalid(RecordArtifactFile::FreeSpaceManifest {
            generation: header.generation(),
        }));
    }
    Ok((entries, artifacts, topology))
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
    budget: &mut ManifestEntryBudget,
) -> PageObservationFailure {
    match failure {
        MembershipProjectionFailure::EntryLimit { observed } => {
            budget.refuse_decoded(observed).into()
        }
        MembershipProjectionFailure::Integrity(rejection) => PageObservationFailure::Integrity {
            artifact,
            denial: rejection.diagnostic(),
        },
    }
}

#[cfg(test)]
#[path = "selected_source_inventory/canonical_free_entry_tests.rs"]
mod canonical_free_entry_tests;
#[cfg(test)]
#[path = "selected_source_inventory/membership_limit_tests.rs"]
mod membership_limit_tests;
