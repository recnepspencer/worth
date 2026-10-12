use super::{
    commit_watch::WorthQueryConditionalCommitWatchSet,
    evaluation_affinity::WorthQueryConditionalEvaluationAffinity,
    evaluation_binding::WorthQueryInactiveTemporalEvaluationBinding,
};
use std::collections::BTreeMap;

pub(super) fn select<Clock, Input>(
    active_affinity: Option<&WorthQueryConditionalEvaluationAffinity>,
    active_cursor: Option<worth_relational::facade::publication::PatchStreamPosition>,
    active_watch: &WorthQueryConditionalCommitWatchSet,
    inactive_bindings: &BTreeMap<
        crate::basis::WorthQueryProductBranchReadPosition,
        WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>,
    >,
    selected: &crate::basis::WorthQueryProductBranchReadIdentity,
) -> Option<WorthQueryPredecessorEvaluationState> {
    let active = active_affinity.and_then(|affinity| {
        affinity.is_predecessor_of(selected).then_some((
            affinity.identity().position(),
            WorthQueryPredecessorEvaluationState {
                cursor: active_cursor,
                commit_watch: active_watch.clone(),
            },
        ))
    });
    inactive_bindings
        .values()
        .filter(|binding| binding.affinity.is_predecessor_of(selected))
        .map(|binding| {
            (
                binding.affinity.identity().position(),
                WorthQueryPredecessorEvaluationState {
                    cursor: binding.authoritative_commit_cursor,
                    commit_watch: binding.commit_watch.clone(),
                },
            )
        })
        .chain(active)
        .max_by(|left, right| left.0.cmp(right.0))
        .map(|(_, state)| state)
}

pub(super) struct WorthQueryPredecessorEvaluationState {
    pub(super) cursor: Option<worth_relational::facade::publication::PatchStreamPosition>,
    pub(super) commit_watch: super::commit_watch::WorthQueryConditionalCommitWatchSet,
}
