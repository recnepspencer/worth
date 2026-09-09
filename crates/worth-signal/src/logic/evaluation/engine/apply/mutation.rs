use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::output_equivalence::OutputEquivalencePolicy;
use crate::logic::evaluation::{AppliedEffectReport, EvaluationEffect, PendingDependencySnapshot};

pub(super) fn apply_evaluation_effect(
    graph: &mut SignalGraph,
    effect: EvaluationEffect,
    output_equivalence: OutputEquivalencePolicy,
    comparator_resolver: &mut impl ComparatorPolicyResolver,
    defer_snapshot_commit: bool,
    work: &mut crate::logic::evaluation::EvaluationWork<'_>,
) -> Result<(AppliedEffectReport, Option<PendingDependencySnapshot>), SignalError> {
    graph.apply_effect(
        effect,
        output_equivalence,
        comparator_resolver,
        defer_snapshot_commit,
        work,
    )
}
