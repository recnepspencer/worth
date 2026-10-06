//! The two roots one step of the walk joins, as observed, and the step's
//! charge. The charge and the views physics replays are taken from this one
//! pair, so the charge is never counted from other roots than the step.

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::ReleasedInventoryView;

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::{
    manifest_entry_budget::ManifestEntryBudget, selected_source_inventory,
};
use crate::progression::RecoverySelectedSourceInventory;

use super::charge::{segment_tree_rewrite, whole_tree_rewrite, TreeHeaders};
use super::{source_root, Verdict, WalkFailure};

/// One root with the leaves the walk observed under it.
#[derive(Clone, Copy)]
pub(super) struct Observed<'a> {
    pub(super) root: &'a DurablePhysicalRootManifest,
    pub(super) inventory: &'a RecoverySelectedSourceInventory,
    pub(super) routes: &'a [CurrentPhysicalRecordPlacement],
}

impl<'a> Observed<'a> {
    pub(super) fn view(
        self,
        segments: &'a [RecordSegmentPageManifestEntry],
    ) -> ReleasedInventoryView<'a> {
        ReleasedInventoryView::new(
            self.root,
            &self.inventory.free_space,
            self.routes,
            segments,
            &self.inventory.free_entries,
        )
    }

    fn headers(self) -> TreeHeaders {
        TreeHeaders::of(self.root, &self.inventory.free_space)
    }
}

#[derive(Clone, Copy)]
pub(super) struct StepRoots<'a> {
    pub(super) source: Observed<'a>,
    pub(super) result: Observed<'a>,
}

impl StepRoots<'_> {
    /// What a step between two roots the walk already holds is charged for
    /// the trees it rewrote whole: counted from the result, where the
    /// source's node capacities differ.
    fn whole_tree_rewrite(&self) -> Option<usize> {
        let (source, result) = (self.source.headers(), self.result.headers());
        whole_tree_rewrite(source, result)?.checked_add(segment_tree_rewrite(
            source,
            result,
            self.result.inventory.segment_pages.len(),
        ))
    }
}

/// An intermediate root the walk reread, with the leaves under it.
pub(super) struct Reread {
    pub(super) root: DurablePhysicalRootManifest,
    pub(super) inventory: RecoverySelectedSourceInventory,
    pub(super) routes: Vec<CurrentPhysicalRecordPlacement>,
}

impl Reread {
    pub(super) fn observed(&self) -> Observed<'_> {
        Observed {
            root: &self.root,
            inventory: &self.inventory,
            routes: &self.routes,
        }
    }
}

/// Charges one step, `step` for what its member declares and then every tree
/// it rewrote whole, and rereads its result root. `None` where the result is
/// `selected`, which the walk already holds.
///
/// Each charge comes off `budget` before the reads it pays for: the step's
/// own before any read, a rewritten tree's as soon as its count is known. The
/// result's headers count its records and its free entries. Nothing counts
/// its segment pages short of reading that tree, so those are charged once
/// that tree is read and before any other.
#[allow(clippy::too_many_arguments)]
pub(super) fn charge_and_reread(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    format: PhysicalRecordFormatDeclaration,
    step: usize,
    source: Observed<'_>,
    selected: Observed<'_>,
) -> Result<Option<Reread>, WalkFailure> {
    budget.consume(step)?;
    let generation = source.root.generation().checked_add(1).proven()?;
    if generation == selected.root.generation() {
        let roots = StepRoots {
            source,
            result: selected,
        };
        budget.consume(roots.whole_tree_rewrite().proven()?)?;
        return Ok(None);
    }
    // The walk still rereads each intermediate root in full. The reread
    // charges nothing: it is admitted as one view, of no more entries than
    // recovery admits, and its bytes are observation bytes.
    let mut view = budget.view();
    let (root, unit) = source_root(discovery, generation, format, &mut view)?;
    let headers =
        selected_source_inventory::observe_headers(discovery, &root, format, &unit, trace)?;
    let (before, after) = (
        source.headers(),
        TreeHeaders::of(&root, headers.free_space()),
    );
    budget.consume(whole_tree_rewrite(before, after).proven()?)?;
    let segments = headers.observe_segments(discovery, &root, format, &mut view, trace)?;
    budget.consume(segment_tree_rewrite(before, after, segments.pages()))?;
    let inventory = segments.observe_free_entries(discovery, &root, format, &mut view, trace)?;
    let routes = selected_source_inventory::observe_routes_with_budget(
        discovery, &root, format, &unit, &mut view, trace,
    )?;
    Ok(Some(Reread {
        root,
        inventory,
        routes,
    }))
}

#[cfg(test)]
#[path = "step_roots_tests.rs"]
mod tests;
