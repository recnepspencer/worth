//! Bounded checkpoint-to-selected walk across interleaved ordinary and V3
//! root effects. This is preplanning lineage only; control custody is joined
//! separately before Store may seal an ordered release sequence.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest,
    PersistedPhysicalRecoveryBlobSemantic, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, OrderedRootHistoryBuilder, PhysicalSourceSelection,
    ReleasedInventoryView, VerifiedOrderedRootHistory, VerifiedOrdinaryRootStep,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory,
};
use crate::progression::RecoverySelectedSourceInventory;

use super::historical_drop::source_root;

#[path = "ordered_history/resident.rs"]
mod resident;
use resident::{
    dropped_bounded, inventory_resident_bytes, manifest_retained_bytes, segment_pair_bounded,
};
#[path = "ordered_history/controls.rs"]
mod controls;
#[path = "ordered_history/transcript.rs"]
mod transcript;
use transcript::inventory_transcript;
#[path = "ordered_history/head_replay.rs"]
mod head_replay;
#[path = "ordered_history/released_edge.rs"]
mod released_edge;
#[path = "ordered_history/released_observation.rs"]
mod released_observation;
pub(in crate::orchestration::planning) use released_observation::OrderedReleasedObservation;

#[allow(clippy::too_many_arguments)]
pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    selected_root: &DurablePhysicalRootManifest,
    selected_inventory: &RecoverySelectedSourceInventory,
    selected_routes: &[CurrentPhysicalRecordPlacement],
    redo: &AdmittedPhysicalRedoMembers,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Option<(
    VerifiedOrderedRootHistory,
    Vec<OrderedReleasedObservation>,
    u64,
)> {
    let checkpoint = selection.checkpoint()?;
    let checkpoint_generation = checkpoint.checkpoint().source().root().generation();
    let checkpoint_root = source_root(discovery, checkpoint_generation, format, budget)?;
    let checkpoint_inventory = selected_source_inventory::observe_with_budget(
        discovery,
        &checkpoint_root,
        format,
        budget,
        byte_limit,
        trace,
    )
    .ok()?;
    let checkpoint_routes = selected_source_inventory::observe_routes_with_budget(
        discovery,
        &checkpoint_root,
        format,
        budget,
        trace,
    )
    .ok()?;
    let initial = inventory_transcript(
        &checkpoint_root,
        &checkpoint_inventory,
        &checkpoint_routes,
        format,
        maximum_entries,
    )?;
    let mut history = OrderedRootHistoryBuilder::begin(
        &checkpoint_root,
        &checkpoint_inventory.free_space,
        checkpoint.source_root_frame_sha256(),
        checkpoint
            .checkpoint()
            .compaction_cutover()
            .wal_cutoff_lsn_exclusive(),
        initial,
        format,
        selected_root
            .generation()
            .checked_sub(checkpoint_generation)?,
        maximum_scratch_bytes,
    )
    .ok()?;
    let mut current_root = checkpoint_root;
    let mut current_inventory = checkpoint_inventory;
    let mut current_routes = checkpoint_routes;
    let mut releases = Vec::new();
    let mut retained = released_edge::ReleasedRetention::default();
    let selected_resident = inventory_resident_bytes(selected_inventory, selected_routes)?;
    while current_root.generation() < selected_root.generation() {
        let mut matching = redo.admitted_root_step_members().filter(|member| {
            member.materialization().source_root_generation() == current_root.generation()
        });
        let member = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        let next_generation = current_root.generation().checked_add(1)?;
        let next_root = if next_generation == selected_root.generation() {
            selected_root.clone()
        } else {
            source_root(discovery, next_generation, format, budget)?
        };
        let next_inventory = if next_generation == selected_root.generation() {
            None
        } else {
            Some(
                selected_source_inventory::observe_with_budget(
                    discovery, &next_root, format, budget, byte_limit, trace,
                )
                .ok()?,
            )
        };
        let next_routes = if next_generation == selected_root.generation() {
            None
        } else {
            Some(
                selected_source_inventory::observe_routes_with_budget(
                    discovery, &next_root, format, budget, trace,
                )
                .ok()?,
            )
        };
        let result_inventory = next_inventory.as_ref().unwrap_or(selected_inventory);
        let result_routes = next_routes.as_deref().unwrap_or(selected_routes);
        let current_resident = inventory_resident_bytes(&current_inventory, &current_routes)?;
        let result_resident = if next_inventory.is_some() {
            inventory_resident_bytes(result_inventory, result_routes)?
        } else {
            0
        };
        let live_inventory = selected_resident
            .checked_add(current_resident)?
            .checked_add(result_resident)?;
        let retained_before = retained.prior_bytes(releases.capacity())?;
        let input_limit = maximum_scratch_bytes
            .checked_sub(live_inventory)?
            .checked_sub(retained_before)?;
        let (source_segments, result_segments, input_scratch) = segment_pair_bounded(
            &current_inventory,
            result_inventory,
            maximum_entries,
            input_limit,
        )?;
        let source = ReleasedInventoryView::new(
            &current_root,
            &current_inventory.free_space,
            &current_routes,
            &source_segments,
            &current_inventory.free_entries,
        );
        let result = ReleasedInventoryView::new(
            &next_root,
            &result_inventory.free_space,
            result_routes,
            &result_segments,
            &result_inventory.free_entries,
        );
        match member.materialization().blob_semantic() {
            PersistedPhysicalRecoveryBlobSemantic::RecordsDropped(_) => {
                released_edge::admit(
                    discovery,
                    budget,
                    trace,
                    released_edge::ReleasedStep {
                        member,
                        redo,
                        source,
                        result,
                        source_inventory: &current_inventory,
                        result_inventory,
                        format,
                        maximum_entries,
                        maximum_scratch_bytes,
                        live_inventory,
                        input_scratch,
                        retained_before,
                    },
                    &mut history,
                    &mut releases,
                    &mut retained,
                )?;
            }
            _ => {
                let transition = VerifiedOrdinaryRootStep::admit_preplanning(
                    source,
                    result,
                    member,
                    format,
                    maximum_entries,
                    maximum_scratch_bytes.checked_sub(input_scratch)?,
                )
                .ok()?;
                let roster_bytes = (releases.capacity() as u64)
                    .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)?;
                let retained_bytes = retained
                    .manifest_bytes
                    .checked_add(retained.control_bytes)?
                    .checked_add(roster_bytes)?;
                let step_peak = input_scratch
                    .checked_add(transition.scratch_bytes())?
                    .checked_add(retained_bytes)?
                    .checked_add(live_inventory)?;
                if step_peak > maximum_scratch_bytes {
                    return None;
                }
                retained.peak_scratch = retained.peak_scratch.max(step_peak);
                history
                    .advance_ordinary(
                        member,
                        transition,
                        &next_root,
                        &result_inventory.free_space,
                        format,
                    )
                    .ok()?;
            }
        }
        if let (Some(inventory), Some(routes)) = (next_inventory, next_routes) {
            current_root = next_root;
            current_inventory = inventory;
            current_routes = routes;
        } else {
            break;
        }
    }
    let history = history
        .finish(selected_root, &selected_inventory.free_space, format)
        .ok()?;
    retained.peak_scratch = retained.peak_scratch.max(history.peak_scratch_bytes());
    Some((history, releases, retained.peak_scratch))
}
