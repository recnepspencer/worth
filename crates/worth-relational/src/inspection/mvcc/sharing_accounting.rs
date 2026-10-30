use std::collections::{BTreeMap, BTreeSet};

use super::{
    RelationalAuthoritativeAllocationKind, RelationalAuthoritativeAllocationLocator,
    RelationalAuthoritativeAllocationObservation,
};
use crate::branch::{RelationalBranchRoot, RelationalRootAuthoritativeAllocationKind};
use crate::history::{RelationalCommitArtifact, RelationalCommitAuthoritativeAllocationKind};
use crate::inspection::mvcc::retained_storage::RelationalRetainedStorageWork;
#[path = "sharing_accounting/census.rs"]
mod census;

#[derive(Debug, Default)]
pub(in crate::inspection::mvcc) struct RelationalAuthoritativeAllocationAccounting {
    allocations: BTreeMap<RelationalAuthoritativeAllocationLocator, u64>,
    pub(super) logical_partition_bytes: u64,
    logical_region_object_bytes: u64,
    pub(super) logical_root_metadata_bytes: u64,
    pub(super) logical_root_reachability_bytes: u64,
    pub(super) logical_canonical_commit_bytes: u64,
    excluded_allocations: BTreeMap<(u8, u64), RelationalExcludedAllocationBytes>,
    seen_payload: BTreeSet<u64>,
    seen_excluded: BTreeSet<u64>,
    census_work: Option<RelationalRetainedStorageWork>,
}

#[derive(Debug, Clone, Copy, Default)]
struct RelationalExcludedAllocationBytes {
    diagnostics: u64,
    retention_metadata: u64,
    allocator_bookkeeping: u64,
    optional_cache: u64,
}

impl RelationalAuthoritativeAllocationAccounting {
    pub(in crate::inspection::mvcc) fn profiled_census() -> Self {
        Self {
            census_work: Some(Default::default()),
            ..Self::default()
        }
    }

    pub(in crate::inspection::mvcc) fn census_work(&self) -> &RelationalRetainedStorageWork {
        self.census_work
            .as_ref()
            .expect("census accounting was selected")
    }

    pub(super) fn observe_branch_root(
        &mut self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
        artifact: &RelationalCommitArtifact,
        derived_cache_bytes: u64,
    ) -> Result<(), crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial> {
        self.observe_root(runtime_instance_id, root)?;
        self.observe_commit_allocation(runtime_instance_id, artifact, derived_cache_bytes);
        Ok(())
    }

    pub(in crate::inspection::mvcc) fn observe_root(
        &mut self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<(), crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial> {
        if let Some(work) = self.census_work.as_mut() {
            work.roots += 1;
        }
        self.observe_partition_allocations(runtime_instance_id, root)?;
        self.observe_root_allocations(runtime_instance_id, root);
        Ok(())
    }

    pub(super) fn allocations(&self) -> Vec<RelationalAuthoritativeAllocationObservation> {
        self.allocations
            .iter()
            .map(|(locator, bytes)| {
                RelationalAuthoritativeAllocationObservation::new(*locator, *bytes)
            })
            .collect()
    }

    pub(in crate::inspection::mvcc) fn unique_bytes(&self) -> u64 {
        self.allocations.values().copied().sum()
    }

    pub(super) fn unique_bytes_for(&self, kind: RelationalAuthoritativeAllocationKind) -> u64 {
        self.allocations
            .iter()
            .filter(|(locator, _)| locator.kind() == kind)
            .map(|(_, bytes)| *bytes)
            .sum()
    }

    pub(super) fn unique_canonical_commit_bytes(&self) -> u64 {
        [
            RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact,
            RelationalAuthoritativeAllocationKind::CanonicalCommitPayload,
            RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope,
            RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested,
        ]
        .into_iter()
        .map(|kind| self.unique_bytes_for(kind))
        .sum()
    }

    pub(super) fn unique_root_reachability_bytes(&self) -> u64 {
        [
            RelationalAuthoritativeAllocationKind::RootReachabilityStructure,
            RelationalAuthoritativeAllocationKind::RootReachabilitySetObject,
            RelationalAuthoritativeAllocationKind::RootReplacementStorage,
            RelationalAuthoritativeAllocationKind::RootRemovalStorage,
        ]
        .into_iter()
        .map(|kind| self.unique_bytes_for(kind))
        .sum()
    }

    pub(super) fn logical_bytes(&self) -> u64 {
        self.logical_partition_bytes
            .saturating_add(self.logical_region_object_bytes)
            .saturating_add(self.logical_root_metadata_bytes)
            .saturating_add(self.logical_root_reachability_bytes)
            .saturating_add(self.logical_canonical_commit_bytes)
    }

    pub(super) fn excluded_unique_bytes(&self) -> (u64, u64, u64, u64) {
        self.excluded_allocations.values().fold(
            (0_u64, 0_u64, 0_u64, 0_u64),
            |(diagnostics, retention, allocator, cache), allocation| {
                (
                    diagnostics.saturating_add(allocation.diagnostics),
                    retention.saturating_add(allocation.retention_metadata),
                    allocator.saturating_add(allocation.allocator_bookkeeping),
                    cache.saturating_add(allocation.optional_cache),
                )
            },
        )
    }

    fn observe_partition_allocations(
        &mut self,
        runtime_instance_id: u64,
        root: &RelationalBranchRoot,
    ) -> Result<(), crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial> {
        let start = self.census_work.as_ref().map(|_| std::time::Instant::now());
        for region in root.storage_regions() {
            if let Some(work) = self.census_work.as_mut() {
                work.region_entries += 1;
            }
            self.logical_partition_bytes = self
                .logical_partition_bytes
                .saturating_add(region.authoritative_bytes);
            self.logical_region_object_bytes = self
                .logical_region_object_bytes
                .saturating_add(region.root_region_bytes)
                .saturating_add(region.partition_state_bytes);
            let partition = root
                .partition_state(region.partition_id).map_err(crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial::StorageUnavailable)?
                .ok_or(crate::inspection::mvcc::sharing::RelationalBranchSharingInspectionDenial::RootUnavailable)?;
            for allocation in
                crate::inspection::mvcc::partition_allocations::authoritative_partition_allocations(
                    runtime_instance_id,
                    partition,
                    &mut self.seen_payload,
                    self.census_work.as_mut(),
                )
            {
                self.insert(allocation.locator(), allocation.authoritative_bytes());
            }
            self.insert(
                RelationalAuthoritativeAllocationLocator::new(
                    runtime_instance_id,
                    RelationalAuthoritativeAllocationKind::RootRegionObject,
                    region.region_id,
                    region.creation_root_id,
                    Some(region.partition_id),
                ),
                region.root_region_bytes,
            );
            self.insert(
                RelationalAuthoritativeAllocationLocator::new(
                    runtime_instance_id,
                    RelationalAuthoritativeAllocationKind::PartitionStateObject,
                    region.region_id,
                    region.creation_root_id,
                    Some(region.partition_id),
                ),
                region.partition_state_bytes,
            );
            let excluded_start = self.census_work.as_ref().map(|_| std::time::Instant::now());
            if let Some(work) = self.census_work.as_mut() {
                work.excluded_partition_walks += 1;
            }
            for (lane, allocation) in
                crate::inspection::mvcc::partition_allocations::excluded_partition_allocations(
                    partition,
                    root,
                    &mut self.seen_excluded,
                )
            {
                use crate::inspection::mvcc::allocation_ledger::RelationalExcludedAllocationLane;
                let mut bytes = RelationalExcludedAllocationBytes::default();
                match lane {
                    RelationalExcludedAllocationLane::Diagnostics => {
                        bytes.diagnostics = allocation.bytes
                    }
                    RelationalExcludedAllocationLane::RetentionMetadata => {
                        bytes.retention_metadata = allocation.bytes
                    }
                    RelationalExcludedAllocationLane::AllocatorBookkeeping => {
                        bytes.allocator_bookkeeping = allocation.bytes
                    }
                    RelationalExcludedAllocationLane::OptionalCache => {
                        bytes.optional_cache = allocation.bytes
                    }
                }
                self.excluded_allocations
                    .insert((2 + lane as u8, allocation.id), bytes);
            }
            if let Some(work) = self.census_work.as_mut() {
                work.excluded_walk_us += excluded_start.unwrap().elapsed().as_micros();
            }
        }
        if let Some(work) = self.census_work.as_mut() {
            work.partition_walk_us += start.unwrap().elapsed().as_micros();
        }
        Ok(())
    }

    fn observe_root_allocations(&mut self, runtime_instance_id: u64, root: &RelationalBranchRoot) {
        let start = self.census_work.as_ref().map(|_| std::time::Instant::now());
        if let Some(work) = self.census_work.as_mut() {
            work.schema_inventory_computations += 1;
        }
        for allocation in root.authoritative_allocation_observations() {
            if let Some(work) = self.census_work.as_mut() {
                work.root_allocation_rows += 1;
            }
            let kind = match allocation.kind {
                RelationalRootAuthoritativeAllocationKind::RootMetadata => {
                    self.logical_root_metadata_bytes = self
                        .logical_root_metadata_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootMetadata
                }
                RelationalRootAuthoritativeAllocationKind::SchemaAuthority => {
                    self.logical_root_metadata_bytes = self
                        .logical_root_metadata_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootSchemaAuthority
                }
                RelationalRootAuthoritativeAllocationKind::PersistentRegionMapNodeObject => {
                    self.logical_root_reachability_bytes = self
                        .logical_root_reachability_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootReachabilityStructure
                }
                RelationalRootAuthoritativeAllocationKind::PersistentRegionSetObject => {
                    self.logical_root_reachability_bytes = self
                        .logical_root_reachability_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootReachabilitySetObject
                }
                RelationalRootAuthoritativeAllocationKind::PersistentRegionReplacementStorage => {
                    self.logical_root_reachability_bytes = self
                        .logical_root_reachability_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootReplacementStorage
                }
                RelationalRootAuthoritativeAllocationKind::PersistentRegionRemovalStorage => {
                    self.logical_root_reachability_bytes = self
                        .logical_root_reachability_bytes
                        .saturating_add(allocation.authoritative_bytes);
                    RelationalAuthoritativeAllocationKind::RootRemovalStorage
                }
            };
            self.insert(
                RelationalAuthoritativeAllocationLocator::new(
                    runtime_instance_id,
                    kind,
                    allocation.owner_id,
                    allocation.owner_id,
                    None,
                ),
                allocation.authoritative_bytes,
            );
        }
        if let Some(work) = self.census_work.as_mut() {
            work.root_inventory_us += start.unwrap().elapsed().as_micros();
        }
    }

    pub(in crate::inspection::mvcc) fn observe_commit_allocation(
        &mut self,
        runtime_instance_id: u64,
        artifact: &RelationalCommitArtifact,
        derived_cache_bytes: u64,
    ) {
        let start = self.census_work.as_ref().map(|_| std::time::Instant::now());
        for allocation in artifact.authoritative_allocation_observations() {
            let kind = match allocation.kind {
                RelationalCommitAuthoritativeAllocationKind::ArtifactObject => {
                    RelationalAuthoritativeAllocationKind::CanonicalCommitArtifact
                }
                RelationalCommitAuthoritativeAllocationKind::CanonicalPayload => {
                    RelationalAuthoritativeAllocationKind::CanonicalCommitPayload
                }
                RelationalCommitAuthoritativeAllocationKind::EnvelopeObject => {
                    RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelope
                }
                RelationalCommitAuthoritativeAllocationKind::EnvelopeNestedOwnerStorage => {
                    RelationalAuthoritativeAllocationKind::CanonicalCommitEnvelopeNested
                }
            };
            self.logical_canonical_commit_bytes = self
                .logical_canonical_commit_bytes
                .saturating_add(allocation.authoritative_bytes);
            self.insert(
                RelationalAuthoritativeAllocationLocator::new(
                    runtime_instance_id,
                    kind,
                    artifact.commit_id().0,
                    artifact.commit_id().0,
                    None,
                ),
                allocation.authoritative_bytes,
            );
        }
        if self.census_work.is_none() {
            let excluded = artifact.excluded_allocation_inventory();
            self.excluded_allocations.insert(
                (1, artifact.commit_id().0),
                RelationalExcludedAllocationBytes {
                    diagnostics: excluded.diagnostic_bytes,
                    optional_cache: excluded
                        .optional_cache_bytes
                        .saturating_add(derived_cache_bytes),
                    ..RelationalExcludedAllocationBytes::default()
                },
            );
        }
        if let Some(work) = self.census_work.as_mut() {
            work.catalog_artifacts += 1;
            work.catalog_inventory_us += start.unwrap().elapsed().as_micros();
        }
    }

    fn insert(&mut self, locator: RelationalAuthoritativeAllocationLocator, bytes: u64) {
        if let Some(previous) = self.allocations.insert(locator, bytes) {
            debug_assert_eq!(
                /* Proof: The compared byte observations use one retained immutable StorageAllocation identity and the same owner-defined layout during this inventory; no allocation can be mutated or recycled while that root and its leases are retained. */
                previous,
                bytes
            );
        }
    }
}
