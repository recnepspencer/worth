//! Opt-in, load-free census of authoritative owner allocations, outside publication.
use super::sharing::{
    accounting::RelationalAuthoritativeAllocationAccounting,
    RelationalBranchSharingInspectionDenial,
};
use crate::branch::RelationalBranchRoot;
use crate::history::{data::CommitId, RelationalCommitArtifact};
use crate::runtime::RelationalRuntime;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod work;
pub use work::RelationalRetainedStorageWork;

/// Failure yields no partial or weakened byte total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRetainedStorageInspectionDenial {
    PublicationBusy,
    OwnerInventory(RelationalBranchSharingInspectionDenial),
}

/// Native owner-defined authoritative allocation categories, not total heap.
///
/// Root classes are disjoint by current, then held, then reclaimable precedence.
/// Their byte totals are incremental allocation unions in that order. Catalog
/// storage is separate and remains append-only regardless of reader release.
/// Nested optional authority records use their existing canonical encoded
/// footprint convention, rather than an allocator layout measurement.
/// Optional caches/index estimates, allocator-private headers, diagnostics and
/// inspection backing are excluded, as are unpublished candidate roots and
/// transaction working sets. A cold/unlinked root refuses the census.
///
/// Publication is quiescent, but concurrent reader Drop and lazy residency are
/// not globally locked: classification describes the actual owner-row visit,
/// before diagnostic clones. It grants no reclamation or currentness authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalRetainedStorageObservation {
    /// Work performed only by this explicit diagnostic walk.
    pub work: RelationalRetainedStorageWork,
    pub current_roots: usize,
    /// Noncurrent, nonreclaimable owner rows, including reservations or other custody.
    pub held_roots: usize,
    pub reclaimable_roots: usize,
    pub catalog_artifacts: usize,
    pub current_root_authoritative_bytes: u64,
    pub held_history_only_authoritative_bytes: u64,
    pub reclaimable_only_authoritative_bytes: u64,
    /// Resident catalog owner footprint, including canonical nested conventions.
    pub catalog_owner_footprint_bytes: u64,
    /// Root allocation union plus catalog footprint; not physical heap usage.
    pub authoritative_owner_footprint_bytes: u64,
}

impl RelationalRuntime {
    /// Inspect live roots, retained-owner rows and the immutable catalog without
    /// loading cold payloads. PublicationBusy defers an in-flight/unsettled
    /// cutover; OwnerInventory preserves cold or unlinked-owner refusal. No
    /// failure returns a partial observation. Run outside ordinary timing.
    pub fn observe_retained_storage(
        &self,
    ) -> Result<RelationalRetainedStorageObservation, RelationalRetainedStorageInspectionDenial>
    {
        self.history
            .canonical_checkpoint_gate()
            .with_settled_publication_snapshot(|| {
                inventory(self).map_err(RelationalRetainedStorageInspectionDenial::OwnerInventory)
            })
            .ok_or(RelationalRetainedStorageInspectionDenial::PublicationBusy)?
    }
}

fn inventory(
    runtime: &RelationalRuntime,
) -> Result<RelationalRetainedStorageObservation, RelationalBranchSharingInspectionDenial> {
    // Eligibility must precede ALL inspector-created root references.
    let retired = runtime.history.retired_root_storage_inventory();
    let current = runtime.history.current_root_storage_inventory();
    let catalog = runtime
        .history
        .commit_artifacts()
        .into_iter()
        .map(|artifact| (artifact.commit_id(), artifact))
        .collect::<BTreeMap<_, _>>();
    let mut accounting = RelationalAuthoritativeAllocationAccounting::profiled_census();
    let mut seen_roots = BTreeSet::new();
    for root in &current {
        observe_root(runtime, root, &catalog, &mut seen_roots, &mut accounting)?;
    }
    let current_roots = seen_roots.len();
    let current_bytes = accounting.unique_bytes();
    for entry in retired.iter().filter(|entry| !entry.reclaimable) {
        observe_root(
            runtime,
            &entry.root,
            &catalog,
            &mut seen_roots,
            &mut accounting,
        )?;
    }
    let held_roots = seen_roots.len() - current_roots;
    let held_bytes = accounting.unique_bytes() - current_bytes;
    for entry in retired.iter().filter(|entry| entry.reclaimable) {
        observe_root(
            runtime,
            &entry.root,
            &catalog,
            &mut seen_roots,
            &mut accounting,
        )?;
    }
    let reclaimable_roots = seen_roots.len() - current_roots - held_roots;
    let root_bytes = accounting.unique_bytes();
    for artifact in catalog.values() {
        accounting.observe_commit_allocation(runtime.runtime_instance_id(), artifact, 0);
    }
    let total = accounting.unique_bytes();
    Ok(RelationalRetainedStorageObservation {
        work: accounting.census_work().clone(),
        current_roots,
        held_roots,
        reclaimable_roots,
        catalog_artifacts: catalog.len(),
        current_root_authoritative_bytes: current_bytes,
        held_history_only_authoritative_bytes: held_bytes,
        reclaimable_only_authoritative_bytes: root_bytes - current_bytes - held_bytes,
        catalog_owner_footprint_bytes: total - root_bytes,
        authoritative_owner_footprint_bytes: total,
    })
}

fn observe_root(
    runtime: &RelationalRuntime,
    root: &Arc<RelationalBranchRoot>,
    catalog: &BTreeMap<CommitId, Arc<RelationalCommitArtifact>>,
    seen: &mut BTreeSet<u64>,
    accounting: &mut RelationalAuthoritativeAllocationAccounting,
) -> Result<(), RelationalBranchSharingInspectionDenial> {
    if !seen.insert(root.metadata_allocation_observation().owner_id) {
        return Ok(());
    }
    accounting.observe_census_root(runtime.runtime_instance_id(), root)?;
    if let Some(commit) = root.commit_id() {
        let artifact = catalog
            .get(&commit)
            .ok_or(RelationalBranchSharingInspectionDenial::RootUnavailable)?;
        // Semantic equality cannot stand for physical canonical-envelope custody.
        if !artifact.links_root(root) {
            return Err(RelationalBranchSharingInspectionDenial::RootUnavailable);
        }
    }
    Ok(())
}
