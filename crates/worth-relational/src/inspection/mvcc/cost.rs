use crate::branch::RelationalBranchIdentity;
use crate::runtime::RelationalRuntime;

use super::sharing::{RelationalBranchSharingInspectionDenial, RelationalBranchSharingObservation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMvccCostScope {
    branches: Vec<RelationalBranchIdentity>,
    baseline: crate::runtime::RelationalBranchSharingCostCounters,
    baseline_retention: crate::history::retention::RelationalRetentionCostCounters,
    baseline_interruption: crate::runtime::RelationalInterruptionCostCounters,
    baseline_maintenance: crate::history::retention::RelationalRetentionCostCounters,
    baseline_coordination_contacts: u64,
    baseline_global_population_scans: u64,
    baseline_settlement_route_visits: u64,
}

impl RelationalMvccCostScope {
    pub fn capture(runtime: &RelationalRuntime, branches: Vec<RelationalBranchIdentity>) -> Self {
        let branch_ids = branches
            .iter()
            .map(|identity| identity.branch_id().clone())
            .collect::<Vec<_>>();
        let baseline_retention = selected_retention_counters(runtime, &branches);
        let baseline_interruption = selected_interruption_counters(runtime, &branches);
        let baseline_maintenance = runtime.retention_cost_counters().maintenance_only();
        Self {
            branches,
            baseline: runtime.branch_sharing_cost_counters_for_branches(&branch_ids),
            baseline_retention,
            baseline_interruption,
            baseline_maintenance,
            baseline_global_population_scans: runtime.branch_population_scan_count(),
            baseline_settlement_route_visits: runtime.history.settlement_route_visit_count(),
            baseline_coordination_contacts: branch_ids
                .iter()
                .filter_map(|branch_id| runtime.history.branch_cell(branch_id))
                .map(|cell| cell.coordination().contact_count())
                .sum(),
        }
    }

    pub fn branches(&self) -> &[RelationalBranchIdentity] {
        &self.branches
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMvccCostObservation {
    sharing: RelationalBranchSharingObservation,
    counters: RelationalMvccCounterObservation,
}

/// MVCC counter evidence that remains available while a
/// performed publication is awaiting settlement and its sharing artifact is
/// not yet catalog-visible. Canonical settlement route visits, population
/// scans and maintenance are runtime-global; selected branch counters retain
/// their branch attribution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalMvccCounterObservation {
    canonical_settlement_route_visits: u64,
    retention_reclaim_batch_size: usize,
    branch_population_scans: u64,
    branch_cell_contacts: u64,
    sharing_cost_delta: crate::runtime::RelationalBranchSharingCostCounters,
    retention_cost_delta: crate::history::retention::RelationalRetentionCostCounters,
    interruption_cost_delta: crate::runtime::RelationalInterruptionCostCounters,
    maintenance_cost_delta: crate::history::retention::RelationalRetentionCostCounters,
}

impl RelationalMvccCostObservation {
    /// Runtime-global actual route visits while checking unsettled publication.
    pub const fn canonical_settlement_route_visits(&self) -> u64 {
        self.counters.canonical_settlement_route_visits()
    }
    /// The configured retention granule; publication-delta work is additional.
    pub const fn retention_reclaim_batch_size(&self) -> usize {
        self.counters.retention_reclaim_batch_size()
    }
    pub fn sharing(&self) -> &RelationalBranchSharingObservation {
        &self.sharing
    }
    pub const fn branch_population_scans(&self) -> u64 {
        self.counters.branch_population_scans()
    }
    pub const fn branch_cell_contacts(&self) -> u64 {
        self.counters.branch_cell_contacts()
    }

    pub const fn sharing_cost_delta(&self) -> crate::runtime::RelationalBranchSharingCostCounters {
        self.counters.sharing_cost_delta()
    }

    pub const fn retention_cost_delta(
        &self,
    ) -> crate::history::retention::RelationalRetentionCostCounters {
        self.counters.retention_cost_delta()
    }

    pub const fn interruption_cost_delta(
        &self,
    ) -> crate::runtime::RelationalInterruptionCostCounters {
        self.counters.interruption_cost_delta()
    }

    pub const fn maintenance_cost_delta(
        &self,
    ) -> crate::history::retention::RelationalRetentionCostCounters {
        self.counters.maintenance_cost_delta()
    }
}

impl RelationalMvccCounterObservation {
    /// Actual canonical-owner route visits, independent of retained-root work.
    pub const fn canonical_settlement_route_visits(&self) -> u64 {
        self.canonical_settlement_route_visits
    }
    /// The actual configured retention batch at observation. This bounds older
    /// generation visits, not delta candidates or physical-root/branch degree.
    pub const fn retention_reclaim_batch_size(&self) -> usize {
        self.retention_reclaim_batch_size
    }
    pub const fn branch_population_scans(&self) -> u64 {
        self.branch_population_scans
    }

    pub const fn branch_cell_contacts(&self) -> u64 {
        self.branch_cell_contacts
    }

    pub const fn sharing_cost_delta(&self) -> crate::runtime::RelationalBranchSharingCostCounters {
        self.sharing_cost_delta
    }

    pub const fn retention_cost_delta(
        &self,
    ) -> crate::history::retention::RelationalRetentionCostCounters {
        self.retention_cost_delta
    }

    pub const fn interruption_cost_delta(
        &self,
    ) -> crate::runtime::RelationalInterruptionCostCounters {
        self.interruption_cost_delta
    }

    pub const fn maintenance_cost_delta(
        &self,
    ) -> crate::history::retention::RelationalRetentionCostCounters {
        self.maintenance_cost_delta
    }
}

impl RelationalRuntime {
    pub fn branch_retention_cost_counters(
        &self,
        identity: &RelationalBranchIdentity,
    ) -> Result<
        crate::history::retention::RelationalRetentionCostCounters,
        RelationalBranchSharingInspectionDenial,
    > {
        if identity.runtime_instance_id() != self.runtime_instance_id() {
            return Err(RelationalBranchSharingInspectionDenial::ForeignRuntime);
        }
        self.history
            .branch_cell(identity.branch_id())
            .filter(|cell| cell.identity() == identity)
            .and_then(|cell| cell.head_retention().binding().ok())
            .map(|binding| binding.counters())
            .ok_or(RelationalBranchSharingInspectionDenial::UnknownBranch)
    }

    pub fn observe_mvcc_cost(
        &self,
        scope: &RelationalMvccCostScope,
    ) -> Result<RelationalMvccCostObservation, RelationalBranchSharingInspectionDenial> {
        let sharing = self.observe_branch_sharing(scope.branches())?;
        let counters = self.observe_mvcc_counters(scope)?;
        Ok(RelationalMvccCostObservation { sharing, counters })
    }

    pub fn observe_mvcc_counters(
        &self,
        scope: &RelationalMvccCostScope,
    ) -> Result<RelationalMvccCounterObservation, RelationalBranchSharingInspectionDenial> {
        validate_counter_scope(self, scope.branches())?;
        let branch_ids = scope
            .branches
            .iter()
            .map(|identity| identity.branch_id().clone())
            .collect::<Vec<_>>();
        let current = self.branch_sharing_cost_counters_for_branches(&branch_ids);
        let baseline = scope.baseline;
        let branch_population_scans = self
            .branch_population_scan_count()
            .saturating_sub(scope.baseline_global_population_scans);
        let branch_cell_contacts = scope
            .branches()
            .iter()
            .filter_map(|identity| self.history.branch_cell(identity.branch_id()))
            .map(|cell| cell.coordination().contact_count())
            .sum::<u64>()
            .saturating_sub(scope.baseline_coordination_contacts);
        Ok(RelationalMvccCounterObservation {
            canonical_settlement_route_visits: self
                .history
                .settlement_route_visit_count()
                .saturating_sub(scope.baseline_settlement_route_visits),
            retention_reclaim_batch_size: self.config.storage.retention.reclaim_batch_size,
            // Population traversal is a deliberately separate runtime-global
            // reconstruction lane. The selected branch counters below remain
            // owner-scoped; this field reports the explicit global delta.
            branch_population_scans,
            branch_cell_contacts,
            sharing_cost_delta: current.saturating_delta_since(baseline),
            retention_cost_delta: selected_retention_counters(self, scope.branches())
                .saturating_delta_since(scope.baseline_retention),
            interruption_cost_delta: selected_interruption_counters(self, scope.branches())
                .saturating_delta_since(scope.baseline_interruption),
            maintenance_cost_delta: self
                .retention_cost_counters()
                .maintenance_only()
                .saturating_delta_since(scope.baseline_maintenance),
        })
    }
}

fn validate_counter_scope(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> Result<(), RelationalBranchSharingInspectionDenial> {
    let mut seen = std::collections::BTreeSet::new();
    for identity in branches {
        if identity.runtime_instance_id() != runtime.runtime_instance_id() {
            return Err(RelationalBranchSharingInspectionDenial::ForeignRuntime);
        }
        if !seen.insert(identity) {
            return Err(RelationalBranchSharingInspectionDenial::DuplicateBranch);
        }
        let Some(cell) = runtime.history.branch_cell(identity.branch_id()) else {
            return Err(RelationalBranchSharingInspectionDenial::UnknownBranch);
        };
        if cell.identity() != identity {
            return Err(RelationalBranchSharingInspectionDenial::UnknownBranch);
        }
    }
    Ok(())
}

fn selected_interruption_counters(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> crate::runtime::RelationalInterruptionCostCounters {
    branches.iter().fold(
        crate::runtime::RelationalInterruptionCostCounters::default(),
        |sum, identity| {
            let counters = runtime
                .history
                .branch_cell(identity.branch_id())
                .filter(|cell| cell.identity() == identity)
                .and_then(|cell| cell.head_retention().binding().ok())
                .map(|binding| binding.interruption_counters())
                .unwrap_or_default();
            sum.saturating_add(counters)
        },
    )
}

fn selected_retention_counters(
    runtime: &RelationalRuntime,
    branches: &[RelationalBranchIdentity],
) -> crate::history::retention::RelationalRetentionCostCounters {
    branches.iter().fold(
        crate::history::retention::RelationalRetentionCostCounters::default(),
        |sum, identity| {
            let counters = runtime
                .history
                .branch_cell(identity.branch_id())
                .filter(|cell| cell.identity() == identity)
                .and_then(|cell| cell.head_retention().binding().ok())
                .map(|binding| binding.counters())
                .unwrap_or_default();
            sum.saturating_add(counters)
        },
    )
}
