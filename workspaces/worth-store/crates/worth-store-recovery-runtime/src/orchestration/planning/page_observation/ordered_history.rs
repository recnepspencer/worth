//! Bounded checkpoint-to-selected walk across interleaved ordinary and V3
//! root effects. This is preplanning lineage only; control custody is joined
//! separately before Store may seal an ordered release sequence.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest,
    PersistedPhysicalRecoveryOperation, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    decide_ordered_root_step_basis, AdmittedPhysicalRedoMembers, OrderedRootHistoryBuilder,
    OrderedRootStepBasis, PhysicalSourceSelection, RetirementReleaseIntent,
    VerifiedOrderedRootHistory, VerifiedOrdinaryRootStep, VerifiedRetirementRootEdge,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory,
};
use crate::progression::RecoverySelectedSourceInventory;

use super::historical_drop::source_root;

#[path = "ordered_history/charge.rs"]
mod charge;
#[path = "ordered_history/step_roots.rs"]
mod step_roots;
use step_roots::{Observed, Reread, StepRoots};
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
#[cfg(test)]
#[path = "ordered_history/test_inventory.rs"]
mod test_inventory;
#[path = "ordered_history/walk_failure.rs"]
mod walk_failure;
use walk_failure::WalkFailure::Unverified;
pub(super) use walk_failure::{Verdict, WalkFailure};
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
    maximum_entries: u64,
    staging: RecoveryAllowance,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Result<Walked, WalkFailure> {
    let refused = |exceeded, budget: &mut ManifestEntryBudget| {
        WalkFailure::refused(exceeded, budget, staging)
    };
    let checkpoint = selection.checkpoint().ok_or(Unverified)?;
    let checkpoint_generation = checkpoint.checkpoint().source().root().generation();
    let (checkpoint_root, checkpoint_unit) =
        source_root(discovery, checkpoint_generation, format, budget)?;
    let checkpoint_inventory = selected_source_inventory::observe_with_budget(
        discovery,
        &checkpoint_root,
        format,
        &checkpoint_unit,
        budget,
        trace,
    )?;
    let checkpoint_routes = selected_source_inventory::observe_routes_with_budget(
        discovery,
        &checkpoint_root,
        format,
        &checkpoint_unit,
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
        staging.admitted(),
    )
    .proven()?;
    let mut current_root = checkpoint_root;
    let mut current_inventory = checkpoint_inventory;
    let mut current_routes = checkpoint_routes;
    let mut releases = Vec::new();
    let mut retained = released_edge::ReleasedRetention::default();
    let selected_resident =
        inventory_resident_bytes(selected_inventory, selected_routes).proven()?;
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
        let selected = Observed {
            root: selected_root,
            inventory: selected_inventory,
            routes: selected_routes,
        };
        let source = Observed {
            root: &current_root,
            inventory: &current_inventory,
            routes: &current_routes,
        };
        let observed_next = step_roots::charge_and_reread(
            discovery,
            budget,
            trace,
            format,
            charge::step(member.as_ref()).proven()?,
            source,
            selected,
        )?;
        let roots = StepRoots {
            source,
            result: observed_next.as_ref().map_or(selected, Reread::observed),
        };
        let current_resident =
            inventory_resident_bytes(&current_inventory, &current_routes).ok_or(Unverified)?;
        let result_resident = if observed_next.is_some() {
            inventory_resident_bytes(roots.result.inventory, roots.result.routes)
                .ok_or(Unverified)?
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
        let held = live_inventory
            .checked_add(retained_before)
            .ok_or(WalkFailure::CountOverflow)?;
        let input_limit = WalkFailure::left(staging, held)?;
        let (source_segments, result_segments, input_scratch) = segment_pair_bounded(
            roots.source.inventory,
            roots.result.inventory,
            budget,
            input_limit,
            staging,
        )?;
        let source = roots.source.view(&source_segments);
        let result = roots.result.view(&result_segments);
        let (next_root, result_inventory) = (roots.result.root, roots.result.inventory);
        let member = match (basis, member) {
            (OrderedRootStepBasis::RetirementIntent(basis), _) => {
                budget.consume(
                    charge::net_free_difference(
                        &roots.source.inventory.free_entries,
                        &result_inventory.free_entries,
                    )
                    .proven()?,
                )?;
                let edge = VerifiedRetirementRootEdge::admit(
                    source,
                    result,
                    basis,
                    format,
                    maximum_entries,
                )
                .map_err(|denial| refused(denial.exceeded_bound(), budget))?;
                history
                    .advance_retirement(edge, next_root, &result_inventory.free_space, format)
                    .map_err(|denial| refused(denial.exceeded_bound(), budget))?;
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
                        source_inventory: roots.source.inventory,
                        result_inventory,
                        format,
                        maximum_entries,
                        staging,
                        live_inventory,
                        input_scratch,
                        retained_before,
                    },
                    &mut history,
                    &mut releases,
                    &mut retained,
                )?;
            }
            Some((member, _)) => {
                // The segment pair was held to `input_limit`, so the step may
                // take what that left. Physics refuses a step past it.
                let step_limit = input_limit.saturating_sub(input_scratch);
                let transition = VerifiedOrdinaryRootStep::admit_preplanning(
                    source,
                    result,
                    member,
                    format,
                    maximum_entries,
                    step_limit,
                )
                .map_err(|denial| refused(denial.exceeded_bound(), budget))?;
                let step_peak = (staging.admitted() - step_limit) + transition.scratch_bytes();
                retained.peak_scratch = retained.peak_scratch.max(step_peak);
                history
                    .advance_ordinary(
                        member,
                        transition,
                        next_root,
                        &result_inventory.free_space,
                        format,
                    )
                    .map_err(|denial| refused(denial.exceeded_bound(), budget))?;
            }
        }
        let Some(reread) = observed_next else {
            break;
        };
        (current_root, current_inventory, current_routes) =
            (reread.root, reread.inventory, reread.routes);
    }
    let history = history
        .finish(selected_root, &selected_inventory.free_space, format)
        .proven()?;
    retained.peak_scratch = retained.peak_scratch.max(history.peak_scratch_bytes());
    Ok((history, releases, retained.peak_scratch))
}
