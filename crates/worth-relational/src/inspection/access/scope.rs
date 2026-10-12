use std::collections::BTreeSet;

use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::inspection::data::{
    ConnectivityInspectionRequest, ConnectivityInspectionSummary, GraphInspectionRequest,
    GraphInspectionSummary, InspectionAccessPath, InspectionAvailability, InspectionDegradation,
    InspectionOrigin, InspectionScope,
};
use crate::storage::data::{RelationalReadView, RetentionPlan};
use crate::visibility::cache_state::cached_historical_state_for_version;

use super::InspectionAccess;

impl<'runtime> InspectionAccess<'runtime> {
    pub(crate) fn read_view_for_scope(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        scope: &InspectionScope,
    ) -> Option<RelationalReadView> {
        match scope {
            InspectionScope::Current => Some(
                self.runtime
                    .read_truth()
                    .read_version(execution, self.runtime.current_version_id()),
            ),
            InspectionScope::Version(version_id) => Some(
                self.runtime
                    .read_truth()
                    .read_version(execution, *version_id),
            ),
            InspectionScope::Snapshot(handle) => {
                self.runtime.read_truth().read_snapshot(handle, execution)
            }
        }
    }

    pub(crate) fn scope_version_id(&self, scope: &InspectionScope) -> VersionId {
        match scope {
            InspectionScope::Current => self.runtime.current_version_id(),
            InspectionScope::Version(version_id) => *version_id,
            InspectionScope::Snapshot(handle) => handle.version_id,
        }
    }

    pub(crate) fn scope_access_path(
        &self,
        scope: &InspectionScope,
        version_id: VersionId,
    ) -> InspectionAccessPath {
        match scope {
            InspectionScope::Current => InspectionAccessPath::DirectLookup,
            InspectionScope::Version(_)
                if cached_historical_state_for_version(self.runtime, version_id).is_some() =>
            {
                InspectionAccessPath::HistoricalRetainedRead
            }
            InspectionScope::Version(_) => InspectionAccessPath::HistoricalReconstructedRead,
            InspectionScope::Snapshot(_) => InspectionAccessPath::SnapshotRead,
        }
    }

    pub(crate) fn scope_availability(
        &self,
        scope: &InspectionScope,
        version_id: VersionId,
    ) -> InspectionAvailability {
        match scope {
            InspectionScope::Current => InspectionAvailability::Direct,
            InspectionScope::Version(_)
                if cached_historical_state_for_version(self.runtime, version_id).is_some() =>
            {
                InspectionAvailability::Direct
            }
            InspectionScope::Version(_) => InspectionAvailability::Reconstructed,
            InspectionScope::Snapshot(_) => InspectionAvailability::Direct,
        }
    }

    pub(crate) fn scope_origin(&self, scope: &InspectionScope) -> InspectionOrigin {
        match scope {
            InspectionScope::Current => InspectionOrigin::CurrentTruth,
            InspectionScope::Version(_) | InspectionScope::Snapshot(_) => {
                InspectionOrigin::VisibilitySnapshot
            }
        }
    }

    pub(crate) fn unavailable_graph_summary(
        &self,
        request: &GraphInspectionRequest,
        version_id: VersionId,
    ) -> GraphInspectionSummary {
        GraphInspectionSummary {
            scope: request.scope.clone(),
            version_id,
            partition_count: 0,
            entity_count: 0,
            relation_count: 0,
            entity_kinds: Vec::new(),
            relation_kinds: Vec::new(),
            origin: InspectionOrigin::VisibilitySnapshot,
            access_path: self.scope_access_path(&request.scope, version_id),
            availability: unavailable_scope_availability(&request.scope),
            degradations: summary_degradations(!request.summary_only, None),
        }
    }

    pub(crate) fn budget_exceeded_graph_summary(
        &self,
        request: &GraphInspectionRequest,
        version_id: VersionId,
        degradation: InspectionDegradation,
    ) -> GraphInspectionSummary {
        GraphInspectionSummary {
            scope: request.scope.clone(),
            version_id,
            partition_count: 0,
            entity_count: 0,
            relation_count: 0,
            entity_kinds: Vec::new(),
            relation_kinds: Vec::new(),
            origin: self.scope_origin(&request.scope),
            access_path: self.scope_access_path(&request.scope, version_id),
            availability: InspectionAvailability::UnavailableByBudget,
            degradations: summary_degradations(!request.summary_only, Some(degradation)),
        }
    }

    pub(crate) fn unavailable_connectivity_summary(
        &self,
        request: &ConnectivityInspectionRequest,
        version_id: VersionId,
    ) -> ConnectivityInspectionSummary {
        ConnectivityInspectionSummary {
            scope: request.scope.clone(),
            version_id,
            component_count: 0,
            largest_component_size: 0,
            enumerated_entity_count: 0,
            components: Vec::new(),
            origin: InspectionOrigin::VisibilitySnapshot,
            access_path: self.scope_access_path(&request.scope, version_id),
            resolution_context:
                crate::inspection::data::InspectionResolutionContext::ConnectivityTraversal,
            availability: unavailable_scope_availability(&request.scope),
            degradations: summary_degradations(request.include_members, None),
        }
    }

    pub(crate) fn budget_exceeded_connectivity_summary(
        &self,
        request: &ConnectivityInspectionRequest,
        version_id: VersionId,
        degradation: InspectionDegradation,
    ) -> ConnectivityInspectionSummary {
        ConnectivityInspectionSummary {
            scope: request.scope.clone(),
            version_id,
            component_count: 0,
            largest_component_size: 0,
            enumerated_entity_count: 0,
            components: Vec::new(),
            origin: self.scope_origin(&request.scope),
            access_path: self.scope_access_path(&request.scope, version_id),
            resolution_context:
                crate::inspection::data::InspectionResolutionContext::ConnectivityTraversal,
            availability: InspectionAvailability::UnavailableByBudget,
            degradations: summary_degradations(request.include_members, Some(degradation)),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ScopeFilter<T: Ord + Copy>(Option<BTreeSet<T>>);

impl<T: Ord + Copy> ScopeFilter<T> {
    pub(crate) fn from_scope(scope: Option<&Vec<T>>) -> Self {
        Self(scope.map(|values| values.iter().copied().collect()))
    }

    pub(crate) fn allows(&self, value: T) -> bool {
        self.0.as_ref().is_none_or(|scope| scope.contains(&value))
    }
}

pub(crate) type PartitionScopeFilter = ScopeFilter<PartitionId>;
pub(crate) type KindScopeFilter = ScopeFilter<KindId>;

pub(crate) fn summary_degradations(
    include_full_detail: bool,
    extra: Option<InspectionDegradation>,
) -> Vec<InspectionDegradation> {
    let mut degradations = Vec::new();
    if !include_full_detail {
        degradations.push(InspectionDegradation::SummaryOnly);
    }
    if let Some(extra) = extra {
        degradations.push(extra);
    }
    degradations
}

pub(crate) fn unavailable_scope_availability(scope: &InspectionScope) -> InspectionAvailability {
    match scope {
        InspectionScope::Snapshot(_) => InspectionAvailability::UnavailableByRetention,
        InspectionScope::Current | InspectionScope::Version(_) => {
            InspectionAvailability::UnavailableByMissingCanonicalArtifacts
        }
    }
}

pub(crate) fn empty_retention_plan(retention_fence_version: VersionId) -> RetentionPlan {
    RetentionPlan {
        retention_fence_version,
        active_snapshot_count: 0,
        branch_pinned_entities: 0,
        replay_pinned_entities: 0,
        snapshot_pinned_entities: 0,
        branch_pinned_relations: 0,
        replay_pinned_relations: 0,
        snapshot_pinned_relations: 0,
        reclaimable_entities: 0,
        reclaimable_relations: 0,
    }
}
