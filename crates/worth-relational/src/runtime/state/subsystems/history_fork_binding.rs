use std::sync::Arc;

use crate::branch::{
    RelationalBranchBasisRegistryMetrics, RelationalBranchReferenceCell,
    RelationalBranchReferenceRegistry, RelationalForkTargetReservation,
    RelationalForkTargetReservationDenial,
};
use crate::history::data::BranchId;
use crate::history::retention::{
    RelationalBranchRetentionBinding, RelationalRetentionAcquisitionDenial,
};
use crate::identity::data::VersionId;

use super::{
    BranchHeadVersionIndexAuthority, RelationalForkMaterializationCost,
    RelationalPhase4ReferenceCostOwner,
};

/// Cloneable, branch-scoped owner binding used by fork services. It exposes
/// no commit catalog, replay inventory, or mutable runtime access.
#[derive(Debug, Clone)]
pub(crate) struct RelationalForkOwnerBinding {
    branches: RelationalBranchReferenceRegistry,
    costs: RelationalPhase4ReferenceCostOwner,
    head_versions: BranchHeadVersionIndexAuthority,
    retention: RelationalBranchRetentionBinding,
    basis_metrics: Arc<RelationalBranchBasisRegistryMetrics>,
    visibility: Option<RelationalForkVisibilityBinding>,
}

#[derive(Debug, Clone)]
struct RelationalForkVisibilityBinding {
    cache: Arc<super::super::visibility::VisibilityCache>,
    protect_branch_heads: bool,
    instrumentation: super::super::services::RuntimeInstrumentation,
}

impl RelationalForkOwnerBinding {
    pub(super) fn new(
        branches: RelationalBranchReferenceRegistry,
        costs: RelationalPhase4ReferenceCostOwner,
        head_versions: BranchHeadVersionIndexAuthority,
        retention: RelationalBranchRetentionBinding,
        basis_metrics: Arc<RelationalBranchBasisRegistryMetrics>,
    ) -> Self {
        Self {
            branches,
            costs,
            head_versions,
            retention,
            basis_metrics,
            visibility: None,
        }
    }

    pub(crate) fn with_visibility(
        mut self,
        cache: Arc<super::super::visibility::VisibilityCache>,
        protect_branch_heads: bool,
        instrumentation: super::super::services::RuntimeInstrumentation,
    ) -> Self {
        self.visibility = Some(RelationalForkVisibilityBinding {
            cache,
            protect_branch_heads,
            instrumentation,
        });
        self
    }

    pub(crate) fn branch_cell(
        &self,
        branch_id: &BranchId,
    ) -> Option<RelationalBranchReferenceCell> {
        self.branches.get(branch_id)
    }

    pub(crate) fn reserve_target(
        &self,
        branch_id: BranchId,
    ) -> Result<RelationalForkTargetReservation, RelationalForkTargetReservationDenial> {
        self.branches.reserve_fork_target(branch_id)
    }

    pub(crate) fn install_target(
        &self,
        reservation: RelationalForkTargetReservation,
        mut cell: RelationalBranchReferenceCell,
    ) {
        self.bind_target_basis_registry_metrics(&mut cell);
        reservation.install(cell);
    }

    pub(crate) fn bind_target_basis_registry_metrics(
        &self,
        cell: &mut RelationalBranchReferenceCell,
    ) {
        cell.bind_basis_registry_metrics(Arc::clone(&self.basis_metrics));
    }

    pub(crate) fn owns_reservation(&self, reservation: &RelationalForkTargetReservation) -> bool {
        self.branches.owns_reservation(reservation)
    }

    pub(crate) fn install_head(
        &self,
        cell: &RelationalBranchReferenceCell,
        root: &Arc<crate::branch::RelationalBranchRoot>,
    ) -> Result<(), RelationalRetentionAcquisitionDenial> {
        let obligation = self.retention.install_head(cell.identity().clone(), root)?;
        cell.head_retention()
            .install(obligation)
            .map_err(|_| RelationalRetentionAcquisitionDenial::OwnerUnavailable)
    }

    pub(crate) fn record_lookup(&self) {
        self.costs.record_branch_cell_lookup();
        self.costs.record_branch_cell_contact();
    }

    pub(crate) fn record_install(
        &self,
        cell: &RelationalBranchReferenceCell,
        cost: RelationalForkMaterializationCost,
        head_version: Option<VersionId>,
    ) {
        self.costs.record_reference_allocation();
        self.costs.record_branch_cell_contact();
        cell.publication_cell().record_sharing_cost(|costs| {
            costs.shared_root_acquisitions = costs.shared_root_acquisitions.saturating_add(1);
            costs.copied_truth_bytes = costs
                .copied_truth_bytes
                .saturating_add(cost.authoritative_bytes);
            costs.copied_commit_envelopes = costs
                .copied_commit_envelopes
                .saturating_add(cost.copied_commit_envelopes);
            costs.fork_materialized_entity_count = costs
                .fork_materialized_entity_count
                .saturating_add(cost.entity_count);
            costs.fork_materialized_relation_count = costs
                .fork_materialized_relation_count
                .saturating_add(cost.relation_count);
            costs.fork_materialized_authoritative_bytes = costs
                .fork_materialized_authoritative_bytes
                .saturating_add(cost.authoritative_bytes);
        });
        self.head_versions.move_head(None, head_version);
        if let (Some(visibility), Some(version_id)) = (&self.visibility, head_version) {
            if visibility.protect_branch_heads {
                let root = cell
                    .root()
                    .expect("installed fork cell retains its exact source root");
                let key = crate::visibility::snapshot_states::VisibilitySnapshotStateKey::for_root(
                    cell.identity().branch_id().clone(),
                    version_id,
                    &root,
                );
                visibility.cache.track_branch_head_state(&key);
                visibility.cache.update_residency(&key, |residency| {
                    residency.branch_head_refs = residency.branch_head_refs.saturating_add(1);
                });
                visibility.instrumentation.count(|counters| {
                    counters.visibility_cache_branch_head_promotions = counters
                        .visibility_cache_branch_head_promotions
                        .saturating_add(1);
                });
            }
        }
    }
}
