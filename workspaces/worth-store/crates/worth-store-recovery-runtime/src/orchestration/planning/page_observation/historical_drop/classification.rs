//! Classify the WAL targets the selected root does not route, in three typed
//! phases: the targets historical V3 drops removed, the ordered walk a retired
//! target needs when no drop walked the history, and the targets that history
//! retired.

use crate::orchestration::recovery_budget::RecoveryAllowance;
use std::sync::Arc;
use worth_store::physical_runtime::BoundedRecoveryFilesystemDiscovery;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, PhysicalRecordFormatDeclaration,
};
use worth_store_recovery_physics::{
    AdmittedPhysicalRedoMembers, PhysicalRedoTarget, PhysicalSourceSelection,
    RecoveryPageObservation, RetirementReleaseIntent, VerifiedOrderedRootHistory,
};

use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::manifest_entry_budget::ManifestEntryBudget;
use crate::progression::RecoverySelectedSourceInventory;

use super::super::ordered_history::{self, OrderedReleasedObservation, WalkFailure, Walked};
use super::super::{AbsentTarget, PageObservationFailure, SelectedFrontier};
use super::ordered_retirements::OrderedRetirements;
use super::released_drops::{self, ReleasedDrops};
use super::{retired_targets, HistoricalDropEvidence};

/// Everything one bounded checkpoint-to-selected walk reads and charges.
pub(in crate::orchestration::planning::page_observation) struct HistoryWalk<'a> {
    pub(in crate::orchestration::planning::page_observation) discovery:
        &'a mut BoundedRecoveryFilesystemDiscovery,
    pub(in crate::orchestration::planning::page_observation) selection: &'a PhysicalSourceSelection,
    pub(in crate::orchestration::planning::page_observation) selected_root:
        &'a DurablePhysicalRootManifest,
    pub(in crate::orchestration::planning::page_observation) selected_inventory:
        &'a RecoverySelectedSourceInventory,
    pub(in crate::orchestration::planning::page_observation) routes:
        &'a [CurrentPhysicalRecordPlacement],
    pub(in crate::orchestration::planning::page_observation) redo: &'a AdmittedPhysicalRedoMembers,
    pub(in crate::orchestration::planning::page_observation) release_intents:
        &'a [RetirementReleaseIntent],
    pub(in crate::orchestration::planning::page_observation) format:
        PhysicalRecordFormatDeclaration,
    pub(in crate::orchestration::planning::page_observation) budget: &'a mut ManifestEntryBudget,
    pub(in crate::orchestration::planning::page_observation) maximum_entries: u64,
    pub(in crate::orchestration::planning::page_observation) staging: RecoveryAllowance,
    pub(in crate::orchestration::planning::page_observation) trace:
        &'a mut RecoveryIntegrityIngressTrace,
}

impl HistoryWalk<'_> {
    pub(super) fn admit(&mut self) -> Result<Walked, WalkFailure> {
        ordered_history::admit(
            self.discovery,
            self.selection,
            self.selected_root,
            self.selected_inventory,
            self.routes,
            self.redo,
            self.release_intents,
            self.format,
            self.budget,
            self.maximum_entries,
            self.staging,
            self.trace,
        )
    }

    fn frontier(&self) -> SelectedFrontier {
        SelectedFrontier::of(&self.selected_inventory.free_space)
    }
}

/// The historical observations, and the targets left to allocation truth.
pub(in crate::orchestration::planning::page_observation) struct ClassifiedTargets<'target> {
    pub(in crate::orchestration::planning::page_observation) observations:
        Vec<RecoveryPageObservation>,
    pub(in crate::orchestration::planning::page_observation) absent:
        Vec<&'target PhysicalRedoTarget>,
    pub(in crate::orchestration::planning::page_observation) drops: Vec<HistoricalDropEvidence>,
    pub(in crate::orchestration::planning::page_observation) history_scratch: u64,
    pub(in crate::orchestration::planning::page_observation) ordered_releases:
        Option<Vec<OrderedReleasedObservation>>,
}

pub(in crate::orchestration::planning::page_observation) fn classify<'target>(
    mut walk: HistoryWalk<'_>,
    targets: &[AbsentTarget<'target>],
) -> Result<ClassifiedTargets<'target>, PageObservationFailure> {
    let released = released_drops::classify(&mut walk, targets)?;
    let ordered = OrderedTargets::walk_on_demand(released, &mut walk)?;
    Ok(ordered.classify_retired(&walk))
}

/// The targets no V3 drop removed, and the verified history, if any, that
/// can prove them retired.
struct OrderedTargets<'target> {
    remaining: Vec<AbsentTarget<'target>>,
    observations: Vec<RecoveryPageObservation>,
    drops: Vec<HistoricalDropEvidence>,
    history: Option<Arc<VerifiedOrderedRootHistory>>,
    ordered_releases: Option<Vec<OrderedReleasedObservation>>,
    history_scratch: u64,
}

impl<'target> OrderedTargets<'target> {
    /// When no V3 drop walked the history, yet the selected root already
    /// allocated an absent target, the history is walked for that target:
    /// every publication retires the derived nodes it replaced, and only the
    /// ordered walk can prove that. A walk that ran out of an admitted limit
    /// is reported as that limit. A walk that does not verify proves nothing
    /// and leaves the target to allocation truth.
    fn walk_on_demand(
        released: ReleasedDrops<'target>,
        walk: &mut HistoryWalk<'_>,
    ) -> Result<Self, PageObservationFailure> {
        let ReleasedDrops {
            remaining,
            observations,
            evidence,
            walked,
        } = released;
        let frontier = walk.frontier();
        let (history, ordered_releases, history_scratch) = match walked {
            Some(walked) => (Some(walked.history), Some(walked.releases), walked.scratch),
            None if remaining
                .iter()
                .any(|target| target.allocated_under(frontier)) =>
            {
                match walk.admit() {
                    Ok((history, _, scratch)) => (Some(Arc::new(history)), None, scratch),
                    Err(failure) => match failure.stopped() {
                        Some(limit) => return Err(limit),
                        None => (None, None, 0),
                    },
                }
            }
            None => (None, None, 0),
        };
        Ok(Self {
            remaining,
            observations,
            drops: evidence,
            history,
            ordered_releases,
            history_scratch,
        })
    }

    /// A target published by an ordered edge and removed by a later ordinary
    /// retirement edge (for example a V3 drop's replacement directory frame
    /// superseded by a derived-directory retirement, or an inline page of
    /// derived nodes a later publication replaced) is classified from the
    /// already-verified history; no further media is read.
    fn classify_retired(self, walk: &HistoryWalk<'_>) -> ClassifiedTargets<'target> {
        let Self {
            mut remaining,
            mut observations,
            drops,
            history,
            ordered_releases,
            history_scratch,
        } = self;
        if let Some(history) = history.as_deref() {
            let store = OrderedRetirements::of(walk.redo, history, walk.routes);
            let physics = walk.redo.historical_retirements(walk.selection, history);
            let (retired, absent) =
                retired_targets::classify(remaining, walk.frontier(), &store, |target| {
                    physics.admit(target).and_then(|witness| {
                        RecoveryPageObservation::historical_retired_target(target, witness)
                    })
                });
            observations.extend(retired);
            remaining = absent;
        }
        ClassifiedTargets {
            observations,
            absent: remaining.into_iter().map(AbsentTarget::first).collect(),
            drops,
            history_scratch,
            ordered_releases,
        }
    }
}
