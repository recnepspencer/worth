use crate::branch::RelationalBranchIdentity;
use crate::runtime::RelationalRuntime;

use super::inventory::inventory_sharing_scope;
use super::scope::resolve_sharing_scope;
use super::*;

impl RelationalRuntime {
    pub fn observe_branch_sharing(
        &self,
        branches: &[RelationalBranchIdentity],
    ) -> Result<RelationalBranchSharingObservation, RelationalBranchSharingInspectionDenial> {
        let scope = resolve_sharing_scope(self, branches)?;
        let inventory = inventory_sharing_scope(self, &scope);
        let branch_ids = scope
            .iter()
            .map(|entry| entry.branch_id.clone())
            .collect::<Vec<_>>();
        let sharing_costs = self.branch_sharing_cost_counters_for_branches(&branch_ids);
        let authoritative_allocations = inventory.accounting.allocations();
        let region_locators = partition_region_locators(&authoritative_allocations);
        let excluded = inventory.accounting.excluded_unique_bytes();
        Ok(assemble_observation(SharingObservationAssembly {
            branch_count: branches.len(),
            inventory,
            costs: sharing_costs,
            authoritative_allocations,
            region_locators,
            excluded,
        }))
    }
}

struct SharingObservationAssembly {
    branch_count: usize,
    inventory: super::inventory::BranchSharingInventory,
    costs: crate::runtime::RelationalBranchSharingCostCounters,
    authoritative_allocations: Vec<RelationalAuthoritativeAllocationObservation>,
    region_locators: Vec<RelationalStorageRegionLocator>,
    excluded: (u64, u64, u64, u64),
}

fn assemble_observation(
    assembly: SharingObservationAssembly,
) -> RelationalBranchSharingObservation {
    let inventory = assembly.inventory;
    let accounting = inventory.accounting;
    let costs = assembly.costs;
    let excluded = assembly.excluded;
    RelationalBranchSharingObservation {
        inspection_version: RELATIONAL_SHARING_INSPECTION_VERSION,
        byte_metric_scope: RelationalSharingByteMetricScope::CompleteAuthoritativeOwnerAllocations,
        branch_count: assembly.branch_count as u64,
        unique_root_count: inventory.root_ids.len() as u64,
        root_ids: inventory.root_ids.into_iter().collect(),
        unique_canonical_commit_artifacts: inventory.commit_ids.len() as u64,
        logical_branch_partition_payload_bytes: accounting.logical_partition_bytes,
        unique_physical_partition_payload_bytes: accounting
            .unique_bytes_for(RelationalAuthoritativeAllocationKind::PartitionPayload),
        logical_branch_root_metadata_bytes: accounting.logical_root_metadata_bytes,
        unique_physical_root_metadata_bytes: accounting
            .unique_bytes_for(RelationalAuthoritativeAllocationKind::RootMetadata),
        logical_branch_root_reachability_bytes: accounting.logical_root_reachability_bytes,
        unique_physical_root_reachability_bytes: accounting.unique_root_reachability_bytes(),
        logical_branch_canonical_commit_bytes: accounting.logical_canonical_commit_bytes,
        unique_physical_canonical_commit_bytes: accounting.unique_canonical_commit_bytes(),
        logical_branch_authoritative_bytes: accounting.logical_bytes(),
        unique_physical_authoritative_bytes: accounting.unique_bytes(),
        unique_diagnostic_bytes: excluded.0,
        unique_retention_metadata_bytes: excluded.1,
        unique_allocator_bookkeeping_bytes: excluded.2,
        unique_optional_cache_bytes: excluded.3,
        branch_metadata_bytes: (assembly.branch_count
            * std::mem::size_of::<crate::branch::RelationalBranchReferenceState>())
            as u64,
        copied_truth_bytes: costs.copied_truth_bytes,
        copied_commit_envelopes: costs.copied_commit_envelopes,
        fork_materialized_entity_count: costs.fork_materialized_entity_count,
        fork_materialized_relation_count: costs.fork_materialized_relation_count,
        fork_materialized_authoritative_bytes: costs.fork_materialized_authoritative_bytes,
        shared_root_acquisitions: costs.shared_root_acquisitions,
        publication_touched_region_count: costs.publication_touched_region_count,
        publication_reused_region_count: costs.publication_reused_region_count,
        publication_new_authoritative_bytes: costs.publication_new_authoritative_bytes,
        reclaimable_unique_bytes: costs.reclaimable_unique_bytes,
        coordination_contacts: inventory.coordination_contacts,
        coordination_waits: inventory.coordination_waits,
        correctness_index_posture: inventory.correctness_index_posture,
        coordination_cells: inventory.coordination_cells,
        region_locators: assembly.region_locators,
        authoritative_allocations: assembly.authoritative_allocations,
        visibility_commitments: inventory.visibility_commitments,
        inspection_reconstructed_region_count: inventory.reconstructed_region_count,
    }
}

fn partition_region_locators(
    allocations: &[RelationalAuthoritativeAllocationObservation],
) -> Vec<RelationalStorageRegionLocator> {
    allocations
        .iter()
        .filter_map(|allocation| {
            let locator = allocation.locator();
            (locator.kind() == RelationalAuthoritativeAllocationKind::RootRegionObject).then(|| {
                RelationalStorageRegionLocator::new(
                    locator.runtime_instance_id(),
                    locator.creation_owner_id(),
                    locator.owner_id(),
                    locator
                        .partition_id()
                        .expect("partition allocation locator"),
                )
            })
        })
        .collect()
}
