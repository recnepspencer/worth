//! Node-local consequence of replacing the installed dependency topology.
use crate::data::graph::storage::invalidation_causes::PendingCauseSetId;
use crate::data::graph::DependencySetId;
use crate::data::node::{NodeHotData, NodeState, NodeWarmData};
use crate::data::proof::invalidation::binding::PendingDependencyRevalidation;

pub(in crate::data::graph) fn replace_dependency_topology(
    hot: &mut NodeHotData,
    warm: &mut NodeWarmData,
    dependencies: DependencySetId,
    pending: PendingDependencyRevalidation,
) {
    let invalidates = hot.pending_cause_set_id != PendingCauseSetId::EMPTY;
    hot.dependencies_id = dependencies;
    hot.dependency_revision = pending.dependency_revision();
    hot.pending_cause_set_id = PendingCauseSetId::EMPTY;
    if invalidates {
        hot.dirty_aspects = crate::data::aspect::AspectMask::EMPTY;
        hot.dirty_partition_scope_aspects = crate::data::aspect::AspectMask::EMPTY;
        warm.dirty_partition_scope_payload.clear();
    }
    if invalidates || matches!(hot.state, NodeState::Clean) {
        hot.state = NodeState::MaybeStale;
    }
    warm.pending_dependency_revalidation = Some(pending);
}
