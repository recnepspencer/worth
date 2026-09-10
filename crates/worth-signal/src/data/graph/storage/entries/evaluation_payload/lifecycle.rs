//! Node-local lifecycle transformations; cause storage and waiter indexes remain caller-owned.
use crate::data::node::{NodeHotData, NodeState, NodeWarmData};
use crate::data::output::PartitionSubscription;

pub(in crate::data::graph) fn transition_clean(hot: &mut NodeHotData, warm: &mut NodeWarmData) {
    hot.state = NodeState::Clean;
    hot.dirty_aspects = crate::data::aspect::AspectMask::EMPTY;
    hot.dirty_partition_scope_aspects = crate::data::aspect::AspectMask::EMPTY;
    hot.pending_cause_set_id =
        crate::data::graph::storage::invalidation_causes::PendingCauseSetId::EMPTY;
    warm.dirty_partition_scope_payload.clear();
    warm.pending_dependency_revalidation = None;
    warm.direct_invalidation_basis = None;
}

pub(in crate::data::graph) fn transition_dirty(
    hot: &mut NodeHotData,
    warm: &mut NodeWarmData,
    aspect: crate::data::aspect::Aspect,
    scopes: &[PartitionSubscription],
    invalidates_dependency_causes: bool,
) {
    {
        warm.direct_invalidation_generation = warm
            .direct_invalidation_generation
            .checked_add(1)
            .expect("direct invalidation generation overflow");
        let generation = warm.direct_invalidation_generation;
        match warm.direct_invalidation_basis.as_mut() {
            Some(basis) => basis.merge_seed(generation, aspect, scopes.iter().cloned()),
            None => {
                warm.direct_invalidation_basis = Some(
                        crate::data::proof::invalidation::source_seed::DirectInvalidationBasis::from_seed(
                            generation,
                            aspect,
                            scopes.iter().cloned(),
                        ),
                    );
            }
        }
    }
    let was_clean = matches!(hot.state, NodeState::Clean);
    let already_dirty_for_aspect = !invalidates_dependency_causes
        && hot
            .dirty_aspects
            .contains(crate::data::aspect::AspectMask::from_aspect(aspect));
    let has_scoped_payload = {
        if invalidates_dependency_causes {
            warm.dirty_partition_scope_payload.clear();
        }
        merge_dirty_partition_scopes(warm, aspect, scopes, was_clean, already_dirty_for_aspect)
    };
    if invalidates_dependency_causes {
        hot.pending_cause_set_id =
            crate::data::graph::storage::invalidation_causes::PendingCauseSetId::EMPTY;
        hot.dirty_aspects = crate::data::aspect::AspectMask::EMPTY;
        hot.dirty_partition_scope_aspects = crate::data::aspect::AspectMask::EMPTY;
    }
    hot.state = NodeState::Dirty;
    hot.dirty_aspects.insert(aspect);
    if has_scoped_payload {
        hot.dirty_partition_scope_aspects.insert(aspect);
    } else {
        sync_dirty_partition_scope_flag(hot, aspect);
    }
}

fn merge_dirty_partition_scopes(
    warm: &mut NodeWarmData,
    changed_aspect: crate::data::aspect::Aspect,
    changed_scopes: &[PartitionSubscription],
    was_clean: bool,
    already_dirty_for_aspect: bool,
) -> bool {
    if changed_scopes.is_empty() {
        warm.dirty_partition_scope_payload
            .retain(|(candidate_aspect, _)| *candidate_aspect != changed_aspect);
        return false;
    }
    if !was_clean
        && already_dirty_for_aspect
        && warm
            .dirty_partition_scope_payload
            .iter()
            .find(|(candidate_aspect, _)| *candidate_aspect == changed_aspect)
            .is_none()
    {
        return false;
    }
    for scope in changed_scopes {
        if !warm
            .dirty_partition_scope_payload
            .iter()
            .any(|(candidate_aspect, candidate_scope)| {
                *candidate_aspect == changed_aspect && *candidate_scope == *scope
            })
        {
            warm.dirty_partition_scope_payload
                .push((changed_aspect, scope.clone()));
            warm.dirty_partition_scope_payload.sort_unstable();
        }
    }
    true
}

fn sync_dirty_partition_scope_flag(hot: &mut NodeHotData, aspect: crate::data::aspect::Aspect) {
    hot.dirty_partition_scope_aspects = crate::data::aspect::AspectMask::from_bits(
        hot.dirty_partition_scope_aspects.bits()
            & !crate::data::aspect::AspectMask::from_aspect(aspect).bits(),
    );
}
