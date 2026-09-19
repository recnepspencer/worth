use std::collections::{BTreeMap, BTreeSet};

use worth_relational::facade::transactions::RecordRef;

use super::{
    commit_watch::WorthQueryConditionalCommitWatchSet,
    evaluation_binding::WorthQueryInactiveTemporalEvaluationBinding,
};

pub(super) fn authoritative_routes<Clock, Input>(
    active: bool,
    active_watch: &WorthQueryConditionalCommitWatchSet,
    inactive: &BTreeMap<
        crate::basis::WorthQueryProductBranchReadIdentity,
        WorthQueryInactiveTemporalEvaluationBinding<Clock, Input>,
    >,
) -> (Vec<RecordRef>, bool) {
    let mut records = BTreeSet::new();
    let mut whole_graph = false;
    if active {
        records.extend(active_watch.records().cloned());
        whole_graph |= active_watch.includes_whole_graph();
    }
    for binding in inactive.values() {
        records.extend(binding.commit_watch.records().cloned());
        whole_graph |= binding.commit_watch.includes_whole_graph();
    }
    (records.into_iter().collect(), whole_graph)
}
