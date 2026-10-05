//! Bounded checkpoint-to-selected walk across interleaved ordinary and V3
//! root effects. This is preplanning lineage only; control custody is joined
//! separately before Store may seal an ordered release sequence.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest,
    PersistedPhysicalRecoveryOperation, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    decide_ordered_root_step_basis, AdmittedPhysicalRedoMembers, OrderedRootHistoryBuilder,
    OrderedRootStepBasis, PhysicalSourceSelection, ReleasedInventoryView, RetirementReleaseIntent,
    VerifiedOrderedRootHistory, VerifiedOrdinaryRootStep, VerifiedRetirementRootEdge,
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
#[path = "ordered_history/walk_failure.rs"]
mod walk_failure;
pub(super) use walk_failure::WalkFailure;
use walk_failure::WalkFailure::Unverified;
#[path = "ordered_history/released_observation.rs"]
mod released_observation;
pub(in crate::orchestration::planning) use released_observation::OrderedReleasedObservation;

/// The verified history, the V3 releases it replayed and its peak scratch.
pub(super) type Walked = (
    VerifiedOrderedRootHistory,
    Vec<OrderedReleasedObservation>,
    u64,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    selected_root: &DurablePhysicalRootManifest,
    selected_inventory: &RecoverySelectedSourceInventory,
    selected_routes: &[CurrentPhysicalRecordPlacement],
    redo: &AdmittedPhysicalRedoMembers,
    release_intents: &[RetirementReleaseIntent],
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<Walked, WalkFailure> {
    let checkpoint = selection.checkpoint().ok_or(Unverified)?;
    let checkpoint_generation = checkpoint.checkpoint().source().root().generation();
    let checkpoint_root = source_root(discovery, checkpoint_generation, format, budget)?;
    let checkpoint_inventory = selected_source_inventory::observe_with_budget(
        discovery,
        &checkpoint_root,
        format,
        budget,
        byte_limit,
        trace,
    )?;
    let checkpoint_routes = selected_source_inventory::observe_routes_with_budget(
        discovery,
        &checkpoint_root,
        format,
        budget,
        trace,
    )?;
    let initial = inventory_transcript(
        &checkpoint_root,
        &checkpoint_inventory,
        &checkpoint_routes,
        format,
        maximum_entries,
    )
    .ok_or(Unverified)?;
    let cutoff = checkpoint
        .checkpoint()
        .compaction_cutover()
        .wal_cutoff_lsn_exclusive();
    let mut history = OrderedRootHistoryBuilder::begin(
        &checkpoint_root,
        &checkpoint_inventory.free_space,
        checkpoint.source_root_frame_sha256(),
        cutoff,
        initial,
        format,
        selected_root
            .generation()
            .checked_sub(checkpoint_generation)
            .ok_or(Unverified)?,
        maximum_scratch_bytes,
    )
    .map_err(|_| Unverified)?;
    let mut current_root = checkpoint_root;
    let mut current_inventory = checkpoint_inventory;
    let mut current_routes = checkpoint_routes;
    let mut releases = Vec::new();
    let mut retained = released_edge::ReleasedRetention::default();
    let selected_resident =
        inventory_resident_bytes(selected_inventory, selected_routes).ok_or(Unverified)?;
    while current_root.generation() < selected_root.generation() {
        let mut matching = redo.admitted_root_step_members().filter(|member| {
            member.materialization().source_root_generation() == current_root.generation()
        });
        let member = matching.next();
        if matching.next().is_some() {
            return Err(Unverified);
        }
        let basis = decide_ordered_root_step_basis(
            current_root.generation(),
            history.retirement_prefix(),
            cutoff,
            member.is_some(),
            release_intents,
        )
        .map_err(|_| Unverified)?;
        let next_generation = current_root.generation().checked_add(1).ok_or(Unverified)?;
        let next_root = if next_generation == selected_root.generation() {
            selected_root.clone()
        } else {
            source_root(discovery, next_generation, format, budget)?
        };
        let next_inventory = if next_generation == selected_root.generation() {
            None
        } else {
            Some(selected_source_inventory::observe_with_budget(
                discovery, &next_root, format, budget, byte_limit, trace,
            )?)
        };
        let next_routes = if next_generation == selected_root.generation() {
            None
        } else {
            Some(selected_source_inventory::observe_routes_with_budget(
                discovery, &next_root, format, budget, trace,
            )?)
        };
        let result_inventory = next_inventory.as_ref().unwrap_or(selected_inventory);
        let result_routes = next_routes.as_deref().unwrap_or(selected_routes);
        let current_resident =
            inventory_resident_bytes(&current_inventory, &current_routes).ok_or(Unverified)?;
        let result_resident = if next_inventory.is_some() {
            inventory_resident_bytes(result_inventory, result_routes).ok_or(Unverified)?
        } else {
            0
        };
        let live_inventory = selected_resident
            .checked_add(current_resident)
            .and_then(|live| live.checked_add(result_resident))
            .ok_or(Unverified)?;
        let retained_before = retained
            .prior_bytes(releases.capacity())
            .ok_or(Unverified)?;
        let input_limit = maximum_scratch_bytes
            .checked_sub(live_inventory)
            .and_then(|limit| limit.checked_sub(retained_before))
            .ok_or(Unverified)?;
        let (source_segments, result_segments, input_scratch) = segment_pair_bounded(
            &current_inventory,
            result_inventory,
            maximum_entries,
            input_limit,
        )
        .ok_or(Unverified)?;
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
        let member = match (basis, member) {
            (OrderedRootStepBasis::RetirementIntent(basis), _) => {
                let edge = VerifiedRetirementRootEdge::admit(
                    source,
                    result,
                    basis,
                    format,
                    maximum_entries,
                )
                .map_err(|_| Unverified)?;
                history
                    .advance_retirement(edge, &next_root, &result_inventory.free_space, format)
                    .map_err(|_| Unverified)?;
                None
            }
            (OrderedRootStepBasis::WalMember, member) => Some(member.ok_or(Unverified)?),
        };
        match member.map(|member| (member, member.materialization().operation())) {
            None => {}
            Some((member, PersistedPhysicalRecoveryOperation::RecordsDropped { .. })) => {
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
                )
                .ok_or(Unverified)?;
            }
            Some((member, _)) => {
                let transition = VerifiedOrdinaryRootStep::admit_preplanning(
                    source,
                    result,
                    member,
                    format,
                    maximum_entries,
                    maximum_scratch_bytes
                        .checked_sub(input_scratch)
                        .ok_or(Unverified)?,
                )
                .map_err(|_| Unverified)?;
                let roster_bytes = (releases.capacity() as u64)
                    .checked_mul(std::mem::size_of::<OrderedReleasedObservation>() as u64)
                    .ok_or(Unverified)?;
                let retained_bytes = retained
                    .manifest_bytes
                    .checked_add(retained.control_bytes)
                    .and_then(|bytes| bytes.checked_add(roster_bytes))
                    .ok_or(Unverified)?;
                let step_peak = input_scratch
                    .checked_add(transition.scratch_bytes())
                    .and_then(|peak| peak.checked_add(retained_bytes))
                    .and_then(|peak| peak.checked_add(live_inventory))
                    .ok_or(Unverified)?;
                if step_peak > maximum_scratch_bytes {
                    return Err(Unverified);
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
                    .map_err(|_| Unverified)?;
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
        .map_err(|_| Unverified)?;
    retained.peak_scratch = retained.peak_scratch.max(history.peak_scratch_bytes());
    Ok((history, releases, retained.peak_scratch))
}
