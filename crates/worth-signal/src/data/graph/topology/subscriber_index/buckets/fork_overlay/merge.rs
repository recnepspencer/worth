use super::{SetDelta, SetMergeTraversal};
use crate::data::error::SignalError;
use crate::data::graph::subscription_candidates;
use crate::data::handle::NodeId;
use crate::data::retained_storage::ordered_lookup_steps;
use crate::logic::evaluation::EvaluationWork;
use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Unbounded};

fn admit_merge(
    base: Option<&BTreeSet<NodeId>>,
    delta: Option<&SetDelta>,
    target: &mut Vec<NodeId>,
    work: &mut EvaluationWork<'_>,
) -> Result<(), SignalError> {
    let base_len = base.map_or(0, BTreeSet::len);
    let removed = delta.map_or(0, |delta| delta.removed.len());
    let added = delta.map_or(0, |delta| delta.added.len());
    let intervals = delta.map_or(0, |delta| delta.retired_base_intervals.len());
    // Removed members are a subset of base. Native range traversal skips whole
    // retired runs; its bound depends on live members and interval seeks, never
    // a walk over the retired population. Fixed NodeId keys cost two words.
    let live = base_len.checked_sub(removed);
    let count = live.and_then(|n| n.checked_add(added));
    let bound = (|| {
        let seeks = if intervals == 0 {
            0
        } else {
            intervals.checked_add(1)?
        };
        let seek_work = seeks
            .checked_mul(ordered_lookup_steps(base_len))?
            .checked_mul(4)?;
        // Iterator navigation conservatively allows a whole 64-slot node per
        // yielded interval/member, plus root setup, for std and im storage.
        let iteration = count?
            .checked_add(intervals)?
            .checked_add(1)?
            .checked_mul(128)?;
        seek_work.checked_add(iteration)
    })();
    work.reserve(bound)?;
    subscription_candidates::reserve_additional(target, count.expect("admitted merge count"), work)
}

pub(in super::super) fn extend_merged_set(
    base: Option<&BTreeSet<NodeId>>,
    delta: Option<&SetDelta>,
    target: &mut Vec<NodeId>,
    work: &mut EvaluationWork<'_>,
) -> Result<SetMergeTraversal, SignalError> {
    admit_merge(base, delta, target, work)?;
    let mut traversal = SetMergeTraversal::default();
    if let Some(base) = base {
        extend_live_base(base, delta, target, &mut traversal);
    }
    if let Some(delta) = delta {
        target.extend(delta.added.iter().copied());
    }
    Ok(traversal)
}

fn extend_live_base(
    base: &BTreeSet<NodeId>,
    delta: Option<&SetDelta>,
    target: &mut Vec<NodeId>,
    traversal: &mut SetMergeTraversal,
) {
    let Some(delta) = delta.filter(|delta| !delta.retired_base_intervals.is_empty()) else {
        traversal.base_members += base.len();
        target.extend(base.iter().copied());
        return;
    };

    let mut prior_retired_end = None;
    for (retired_start, retired_end) in &delta.retired_base_intervals {
        extend_base_run(
            base,
            prior_retired_end,
            Some(*retired_start),
            target,
            traversal,
        );
        prior_retired_end = Some(*retired_end);
    }
    extend_base_run(base, prior_retired_end, None, target, traversal);
}

fn extend_base_run(
    base: &BTreeSet<NodeId>,
    after: Option<NodeId>,
    before: Option<NodeId>,
    target: &mut Vec<NodeId>,
    traversal: &mut SetMergeTraversal,
) {
    let range = match (after, before) {
        (Some(after), Some(before)) => base.range((Excluded(after), Excluded(before))),
        (Some(after), None) => base.range((Excluded(after), Unbounded)),
        (None, Some(before)) => base.range((Unbounded, Excluded(before))),
        (None, None) => unreachable!("retired traversal always has a boundary"),
    };
    traversal.range_seeks += 1;
    for member in range {
        traversal.base_members += 1;
        target.push(*member);
    }
}
