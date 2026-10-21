//! Cached native ranges enter the same canonical prefix as fresh outcomes.
use super::{
    super::{
        evaluation,
        meter::{KernelFailure, NestedStopOwner, RunLimits},
        port::TaskOutcome,
        task_result::TaskResult,
    },
    prefix::Prefix,
};
use crate::{
    reduction::{RunRange, RunTree},
    ChargedBytes, ExecutionResourceLease, RetainedRunMetrics,
};
use worth_foundational::PartitionIdentity;

pub(crate) struct RetainedSettlement<'a, R> {
    pub(crate) tree: &'a RunTree<R>,
    pub(crate) metrics: &'a mut RetainedRunMetrics,
    after: Option<PartitionIdentity>,
}
impl<'a, R> RetainedSettlement<'a, R> {
    pub(crate) fn new(tree: &'a RunTree<R>, metrics: &'a mut RetainedRunMetrics) -> Self {
        Self {
            tree,
            metrics,
            after: None,
        }
    }
    pub(super) fn before<E: ChargedBytes>(
        &mut self,
        before: Option<PartitionIdentity>,
        prefix: &mut Prefix<R, E>,
        lease: Option<&ExecutionResourceLease<'_>>,
        limits: &RunLimits,
    ) {
        let visits = self.tree.between(self.after, before, &mut |range| {
            let id = match &range {
                RunRange::Summary(s) => s.first.expect("nonempty range"),
                RunRange::Slot(s) => s.identity(),
            };
            // Even a discarded carried suffix retains real request-stop
            // precedence. One poll covers a skipped immutable range.
            if let Err(stop) = limits.poll_request() {
                prefix.accept(
                    id,
                    TaskOutcome {
                        allocation_denial: None,
                        result: TaskResult::<(), E>::new(
                            Err(KernelFailure::Stop(stop)),
                            Some(NestedStopOwner::Request),
                        ),
                        work: 0,
                        span: 0,
                    },
                    limits,
                );
                return true;
            }
            match range {
                RunRange::Summary(s) => {
                    if prefix.stop.is_some()
                        && limits
                            .ceiling()
                            .is_none_or(|ceiling| s.maximum_slot_work <= ceiling)
                    {
                        prefix.discarded = prefix.discarded.saturating_add(s.work);
                        return true;
                    }
                    if prefix.stop.is_some()
                        || limits
                            .ceiling()
                            .is_some_and(|ceiling| s.maximum_slot_work > ceiling)
                        || prefix
                            .work
                            .checked_add(s.work)
                            .is_none_or(|w| limits.ceiling().is_some_and(|ceiling| w > ceiling))
                    {
                        return false;
                    }
                    prefix.work += s.work;
                    prefix.span = prefix.span.max(s.span);
                    self.metrics.carried_ranges += 1;
                    self.metrics.carried_slots += s.count as u64;
                }
                RunRange::Slot(slot) => {
                    if limits
                        .ceiling()
                        .is_none_or(|ceiling| slot.charged_work() <= ceiling)
                    {
                        if prefix.stop.is_some() {
                            prefix.discarded = prefix.discarded.saturating_add(slot.charged_work());
                            return true;
                        }
                        if prefix
                            .work
                            .checked_add(slot.charged_work())
                            .is_some_and(|w| limits.ceiling().is_none_or(|ceiling| w <= ceiling))
                        {
                            prefix.work += slot.charged_work();
                            prefix.span = prefix.span.max(slot.charged_span());
                            self.metrics.carried_ranges += 1;
                            self.metrics.carried_slots += 1;
                            return true;
                        }
                    }
                    self.metrics.carried_slot_visits += 1;
                    let (outcome, _, _) = evaluation::evaluate(
                        slot,
                        lease,
                        limits,
                        Some(0),
                        None,
                        None,
                        &|slot, context| {
                            slot.try_carry(context)
                                .expect("run eligibility checked before native effects")
                                .map(|_| ())
                                .map_err(KernelFailure::<E>::Stop)
                        },
                    );
                    if prefix.accept(id, outcome, limits).is_some() {
                        self.metrics.carried_slots += 1;
                    }
                }
            }
            true
        });
        self.metrics.range_visits += visits;
        self.after = before;
    }
}
