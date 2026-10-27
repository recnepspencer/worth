use std::collections::BTreeSet;

use crate::branch::RelationalRootCorrectnessIndex;
use crate::history::data::CommitId;
use crate::runtime::RelationalRuntime;

use super::scope::BranchSharingScopeEntry;
use super::*;

pub(super) struct BranchSharingInventory {
    pub(super) accounting: RelationalAuthoritativeAllocationAccounting,
    pub(super) root_ids: BTreeSet<u64>,
    pub(super) commit_ids: BTreeSet<CommitId>,
    pub(super) coordination_cells: Vec<crate::branch::RelationalBranchCoordinationCellId>,
    pub(super) coordination_contacts: u64,
    pub(super) coordination_waits: u64,
    pub(super) correctness_index_posture: RelationalCorrectnessIndexPosture,
    pub(super) visibility_commitments: Vec<RelationalVisibilityCommitmentObservation>,
    pub(super) reconstructed_region_count: u64,
}

pub(super) fn inventory_sharing_scope(
    runtime: &RelationalRuntime,
    scope: &[BranchSharingScopeEntry],
) -> BranchSharingInventory {
    let mut inventory = empty_inventory(scope.len());
    for entry in scope {
        observe_entry(runtime, entry, &mut inventory);
    }
    inventory.coordination_cells.sort_unstable();
    inventory
}

fn empty_inventory(branch_count: usize) -> BranchSharingInventory {
    BranchSharingInventory {
        accounting: RelationalAuthoritativeAllocationAccounting::default(),
        root_ids: BTreeSet::new(),
        commit_ids: BTreeSet::new(),
        coordination_cells: Vec::with_capacity(branch_count),
        coordination_contacts: 0,
        coordination_waits: 0,
        correctness_index_posture: RelationalCorrectnessIndexPosture::AuthoritativeFallback,
        visibility_commitments: Vec::new(),
        reconstructed_region_count: 0,
    }
}

fn observe_entry(
    runtime: &RelationalRuntime,
    entry: &BranchSharingScopeEntry,
    inventory: &mut BranchSharingInventory,
) {
    let first_root_observation = inventory.root_ids.insert(entry.root.id());
    let commit_id = entry
        .root
        .commit_id()
        .expect(/* Proof: Inspection scope admission validates commit linkage and all root axes before constructing its private inventory entries. */ "scope validates commit linkage");
    inventory.commit_ids.insert(commit_id);
    if first_root_observation {
        observe_root(entry, inventory);
    }
    let derived_cache_bytes = runtime
        .indexes
        .derived_artifacts_for_commit(commit_id)
        .owned_allocation_capacity_bytes();
    inventory.accounting.observe_branch_root(
        runtime.runtime_instance_id(),
        &entry.root,
        entry.artifact.as_ref(),
        derived_cache_bytes,
    );
    inventory
        .coordination_cells
        .push(entry.coordination_cell.clone());
    inventory.coordination_contacts = inventory
        .coordination_contacts
        .saturating_add(entry.coordination_contacts);
    inventory.coordination_waits = inventory
        .coordination_waits
        .saturating_add(entry.coordination_waits);
}

fn observe_root(entry: &BranchSharingScopeEntry, inventory: &mut BranchSharingInventory) {
    let axes = entry.root.axes().expect(/* Proof: Inspection scope admission validates commit linkage and all root axes before constructing its private inventory entries. */ "scope validates complete axes");
    inventory.reconstructed_region_count = inventory
        .reconstructed_region_count
        .saturating_add(entry.root.region_count() as u64);
    inventory
        .visibility_commitments
        .push(RelationalVisibilityCommitmentObservation::new(
            entry.root.id(),
            axes.visibility.digest(),
        ));
    inventory.correctness_index_posture = match axes.correctness_index {
        RelationalRootCorrectnessIndex::AuthoritativeFallback => {
            RelationalCorrectnessIndexPosture::AuthoritativeFallback
        }
    };
}
