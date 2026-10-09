//! The two roots one step of the walk joins, as observed, and the step's
//! charge. The charge and the views physics replays are taken from this one
//! pair, so the charge is never counted from other roots than the step.

use std::num::NonZeroUsize;

use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
    RecordSegmentPageManifestEntry,
};
use worth_store_recovery_physics::ReleasedInventoryView;

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::{
    spend, ChargeTarget, EntryAdmission, ManifestEntryBudget, ViewEntryCap,
};
use crate::orchestration::planning::selected_source_inventory;
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

/// Charges a step `step` for what its member declares and then every tree
/// it rewrote whole, and rereads its result root. `None` where the result is
/// `selected`, which the walk already holds.
///
/// Each charge comes off `budget` before the reads it pays for. The step's
/// own, before any read, mints the token that pays for every page of the
/// result root. A rewritten tree's, as soon as its count is known, is
/// admitted before the trees under the headers are read: it pays for entries
/// the step wrote, not for a read, and may be nothing. The result's headers
/// count its records and its free entries. Nothing counts its segment pages
/// short of reading that tree, so those are charged once that tree is read
/// and before any other.
pub(super) fn charge_and_reread(
    discovery: &mut BoundedRecoveryFilesystemDiscovery,
    budget: &mut ManifestEntryBudget,
    trace: &mut RecoveryIntegrityIngressTrace,
    format: PhysicalRecordFormatDeclaration,
    step: NonZeroUsize,
    source: Observed<'_>,
    selected: Observed<'_>,
) -> Result<Option<Reread>, WalkFailure> {
    let generation = source.root.generation().checked_add(1).proven()?;
    let charge = budget.charge(step, ChargeTarget::root(generation))?;
    if generation == selected.root.generation() {
        let roots = StepRoots {
            source,
            result: selected,
        };
        budget.admit(roots.whole_tree_rewrite().proven()?)?;
        // The walk already holds the result: the charge pays for no read.
        spend(charge, generation);
        return Ok(None);
    }
    // The walk still rereads each intermediate root in full. Its leaves
    // charge nothing: they are admitted as one view, of no more entries than
    // recovery admits, and their bytes are observation bytes.
    let mut view = ViewEntryCap::of(budget);
    let root = source_root(discovery, generation, format, &charge)?;
    let headers =
        selected_source_inventory::observe_headers(discovery, &root, format, &charge, trace)?;
    let (before, after) = (
        source.headers(),
        TreeHeaders::of(&root, headers.free_space()),
    );
    budget.admit(whole_tree_rewrite(before, after).proven()?)?;
    let segments = headers.observe_segments(discovery, &root, format, &charge, &mut view, trace)?;
    // B3 owns this: the segment tree's rewrite is charged after its read,
    // until the previous step carries the segment pages it held.
    budget.admit(segment_tree_rewrite(before, after, segments.pages()))?;
    let inventory =
        segments.observe_free_entries(discovery, &root, format, &charge, &mut view, trace)?;
    let routes = selected_source_inventory::observe_routes_with_budget(
        discovery, &root, format, charge, &mut view, trace,
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
