//! Preplanning proof that an old V3 candidate is an ancestor of the selected
//! root through exact, semantics-admitted ordinary C.9 publication edges.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DropSetManifestV3, DurablePhysicalRootManifest,
    PhysicalRecordFormatDeclaration, RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, AdmittedRootStepMemberView, HistoricalReleaseRootChainBuilder,
    HistoricalReleaseRootPrefixBuilder, PhysicalSourceSelection, ReleasedInventoryView,
    VerifiedHistoricalReleaseRootChain, VerifiedOrdinaryRootStep,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory,
};
use crate::progression::{verified_historical_release_transition, RecoverySelectedSourceInventory};

use super::historical_drop::source_root as read_source_root;

#[allow(clippy::too_many_arguments)]
pub(super) fn admit(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    selection: &PhysicalSourceSelection,
    selected_root: &DurablePhysicalRootManifest,
    selected_inventory: &RecoverySelectedSourceInventory,
    selected_routes: &[CurrentPhysicalRecordPlacement],
    first_member: AdmittedRootStepMemberView<'_>,
    manifest: &DropSetManifestV3,
    redo: &AdmittedPhysicalRedoMembers,
    format: PhysicalRecordFormatDeclaration,
    budget: &mut ManifestEntryBudget,
    byte_limit: u64,
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
    trace: &mut RecoveryIntegrityIngressTrace,
) -> Option<(VerifiedHistoricalReleaseRootChain, u64)> {
    let first_projection = first_member.materialization();
    let source_generation = first_projection.source_root_generation();
    let source_root = read_source_root(discovery, source_generation, format, budget)?;
    let source_inventory = selected_source_inventory::observe_with_budget(
        discovery,
        &source_root,
        format,
        budget,
        byte_limit,
        trace,
    )
    .ok()?;
    let source_routes = selected_source_inventory::observe_routes_with_budget(
        discovery,
        &source_root,
        format,
        budget,
        trace,
    )
    .ok()?;
    let first_generation = source_generation.checked_add(1)?;
    let first_root = read_source_root(discovery, first_generation, format, budget)?;
    let first_inventory = selected_source_inventory::observe_with_budget(
        discovery,
        &first_root,
        format,
        budget,
        byte_limit,
        trace,
    )
    .ok()?;
    let first_routes = selected_source_inventory::observe_routes_with_budget(
        discovery,
        &first_root,
        format,
        budget,
        trace,
    )
    .ok()?;
    let mut dropped = manifest.dropped().to_vec();
    if let worth_store_physical_format::PersistedPhysicalRecoveryOperation::DerivedDirectory {
        retirement: Some(retirement),
        ..
    } = first_projection.operation()
    {
        dropped.extend_from_slice(retirement.dropped_records());
    }
    dropped.sort_unstable();
    if dropped.windows(2).any(|pair| pair[0] == pair[1]) {
        return None;
    }
    let (first, input_scratch) = verified_historical_release_transition(
        &source_root,
        &source_inventory,
        &source_routes,
        &first_root,
        &first_inventory,
        &first_routes,
        &dropped,
        first_projection.placements(),
        None,
        format,
        maximum_entries,
        maximum_scratch_bytes,
    )?;
    let first_peak = input_scratch.checked_add(first.scratch_bytes())?;
    let mut chain = HistoricalReleaseRootChainBuilder::begin(
        first_member,
        first,
        &source_root,
        &source_inventory.free_space,
        &first_root,
        &first_inventory.free_space,
        format,
        selected_root.generation().checked_sub(first_generation)?,
        maximum_scratch_bytes,
    )
    .ok()?;
    let mut peak = first_peak;
    let mut current_root = first_root;
    let mut current_inventory = first_inventory;
    let mut current_routes = first_routes;
    while current_root.generation() < selected_root.generation() {
        let next_generation = current_root.generation().checked_add(1)?;
        let next_root = if next_generation == selected_root.generation() {
            selected_root.clone()
        } else {
            read_source_root(discovery, next_generation, format, budget)?
        };
        let mut matches = redo.admitted_root_step_members().filter(|member| {
            member.materialization().source_root_generation() == current_root.generation()
        });
        let member = matches.next()?;
        if matches.next().is_some() {
            return None;
        }
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
        let (transition, input_scratch) = ordinary_transition(
            &current_root,
            &current_inventory,
            &current_routes,
            &next_root,
            result_inventory,
            result_routes,
            member,
            format,
            maximum_entries,
            maximum_scratch_bytes,
        )?;
        peak = peak.max(input_scratch.checked_add(transition.scratch_bytes())?);
        chain
            .advance(
                member,
                transition,
                &next_root,
                &result_inventory.free_space,
                format,
            )
            .ok()?;
        if let (Some(inventory), Some(routes)) = (next_inventory, next_routes) {
            current_root = next_root;
            current_inventory = inventory;
            current_routes = routes;
        } else {
            break;
        }
    }
    let chain = chain
        .finish(selected_root, &selected_inventory.free_space, format)
        .ok()?;
    let selected_checkpoint = selection.checkpoint()?;
    let checkpoint_generation = selected_checkpoint
        .checkpoint()
        .source()
        .root()
        .generation();
    let checkpoint_sha = selected_checkpoint.source_root_frame_sha256();
    let chain = if checkpoint_generation == source_generation {
        chain
            .bind_direct_checkpoint(
                &source_root,
                &source_inventory.free_space,
                checkpoint_sha,
                format,
            )
            .ok()?
    } else {
        if checkpoint_generation > source_generation {
            return None;
        }
        let checkpoint_root = read_source_root(discovery, checkpoint_generation, format, budget)?;
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
        let mut prefix = HistoricalReleaseRootPrefixBuilder::begin(
            &checkpoint_root,
            &checkpoint_inventory.free_space,
            checkpoint_sha,
            format,
            source_generation.checked_sub(checkpoint_generation)?,
            maximum_scratch_bytes,
        )
        .ok()?;
        let mut current_root = checkpoint_root;
        let mut current_inventory = checkpoint_inventory;
        let mut current_routes = checkpoint_routes;
        while current_root.generation() < source_generation {
            let next_generation = current_root.generation().checked_add(1)?;
            let next_root = if next_generation == source_generation {
                source_root.clone()
            } else {
                read_source_root(discovery, next_generation, format, budget)?
            };
            let mut members = redo.admitted_root_step_members().filter(|member| {
                member.materialization().source_root_generation() == current_root.generation()
            });
            let member = members.next()?;
            if members.next().is_some() {
                return None;
            }
            let next_inventory = if next_generation == source_generation {
                None
            } else {
                Some(
                    selected_source_inventory::observe_with_budget(
                        discovery, &next_root, format, budget, byte_limit, trace,
                    )
                    .ok()?,
                )
            };
            let next_routes = if next_generation == source_generation {
                None
            } else {
                Some(
                    selected_source_inventory::observe_routes_with_budget(
                        discovery, &next_root, format, budget, trace,
                    )
                    .ok()?,
                )
            };
            let result_inventory = next_inventory.as_ref().unwrap_or(&source_inventory);
            let result_routes = next_routes.as_deref().unwrap_or(&source_routes);
            let (transition, input_scratch) = ordinary_transition(
                &current_root,
                &current_inventory,
                &current_routes,
                &next_root,
                result_inventory,
                result_routes,
                member,
                format,
                maximum_entries,
                maximum_scratch_bytes,
            )?;
            peak = peak.max(input_scratch.checked_add(transition.scratch_bytes())?);
            prefix
                .advance(
                    member,
                    transition,
                    &next_root,
                    &result_inventory.free_space,
                    format,
                )
                .ok()?;
            if let (Some(inventory), Some(routes)) = (next_inventory, next_routes) {
                current_root = next_root;
                current_inventory = inventory;
                current_routes = routes;
            } else {
                break;
            }
        }
        let prefix = prefix
            .finish(&source_root, &source_inventory.free_space, format)
            .ok()?;
        chain
            .bind_checkpoint_prefix(prefix, checkpoint_sha, maximum_scratch_bytes)
            .ok()?
    };
    peak = peak.max(chain.scratch_bytes());
    Some((chain, peak))
}

#[allow(clippy::too_many_arguments)]
fn ordinary_transition(
    source_root: &DurablePhysicalRootManifest,
    source: &RecoverySelectedSourceInventory,
    source_routes: &[CurrentPhysicalRecordPlacement],
    result_root: &DurablePhysicalRootManifest,
    result: &RecoverySelectedSourceInventory,
    result_routes: &[CurrentPhysicalRecordPlacement],
    member: AdmittedRootStepMemberView<'_>,
    format: PhysicalRecordFormatDeclaration,
    maximum_entries: u64,
    maximum_scratch_bytes: u64,
) -> Option<(VerifiedOrdinaryRootStep, u64)> {
    let source_segments = source
        .segment_pages
        .values()
        .map(|page| page.entry)
        .collect::<Vec<_>>();
    let result_segments = result
        .segment_pages
        .values()
        .map(|page| page.entry)
        .collect::<Vec<_>>();
    let input_scratch = (source_segments.capacity() as u64)
        .checked_add(result_segments.capacity() as u64)?
        .checked_mul(std::mem::size_of::<RecordSegmentPageManifestEntry>() as u64)?;
    let transition = VerifiedOrdinaryRootStep::admit_preplanning(
        ReleasedInventoryView::new(
            source_root,
            &source.free_space,
            source_routes,
            &source_segments,
            &source.free_entries,
        ),
        ReleasedInventoryView::new(
            result_root,
            &result.free_space,
            result_routes,
            &result_segments,
            &result.free_entries,
        ),
        member,
        format,
        maximum_entries,
        maximum_scratch_bytes.checked_sub(input_scratch)?,
    )
    .ok()?;
    Some((transition, input_scratch))
}
