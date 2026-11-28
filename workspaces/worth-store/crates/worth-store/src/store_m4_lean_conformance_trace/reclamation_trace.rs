//! Conformance trace emitter for `StoreM4.ReclamationSafety`, driven by the
//! real compiled `ReclamationPlanValidator` (`plan_reclaim_intent`,
//! `resume_release`).

use super::Trace;
use crate::reclamation_plan_validator::{
    plan_reclaim_intent, resume_release, ReachSpan, ReclaimCoverage, ReclaimVerdict, ReleaseBarrier,
};

const fn span(space: u64, key: u64, first: u64, last: u64) -> ReachSpan {
    ReachSpan {
        space,
        key,
        first,
        last,
    }
}

fn spans_json(spans: &[ReachSpan]) -> String {
    let items: Vec<String> = spans
        .iter()
        .map(|s| format!("[{},{},{},{}]", s.space, s.key, s.first, s.last))
        .collect();
    format!("[{}]", items.join(","))
}

fn coverage_str(c: ReclaimCoverage) -> &'static str {
    match c {
        ReclaimCoverage::CanonicalSuccessor => "canonical_successor",
        ReclaimCoverage::DerivedOnly => "derived_only",
        ReclaimCoverage::Uncovered => "uncovered",
    }
}

fn verdict_str(v: ReclaimVerdict) -> &'static str {
    match v {
        ReclaimVerdict::Admit => "accept",
        ReclaimVerdict::Reachable => "reachable",
        ReclaimVerdict::Uncovered => "uncovered",
        ReclaimVerdict::Obligated => "obligated",
    }
}

fn barrier_str(b: ReleaseBarrier) -> &'static str {
    match b {
        ReleaseBarrier::IntentDurable => "intent_durable",
        ReleaseBarrier::FreeSpacePublished => "free_space_published",
        ReleaseBarrier::CompletionDurable => "completion_durable",
    }
}

fn push_verdict(
    trace: &mut Trace,
    candidate: &[ReachSpan],
    reach: &[ReachSpan],
    obligations: &[ReachSpan],
    coverage: ReclaimCoverage,
) -> ReclaimVerdict {
    let verdict = plan_reclaim_intent(candidate, reach, obligations, coverage);
    trace.push(
        "reclaim_verdict",
        format!(
            r#""candidate":{},"reach":{},"obligations":{},"coverage":"{}","verdict":"{}""#,
            spans_json(candidate),
            spans_json(reach),
            spans_json(obligations),
            coverage_str(coverage),
            verdict_str(verdict),
        ),
    );
    verdict
}

pub(super) fn record(trace: &mut Trace) {
    let candidate = [span(1, 0, 0, 9), span(2, 7, 3, 3)];
    let disjoint_reach = [span(1, 0, 10, 20), span(2, 8, 3, 3), span(3, 0, 0, 9)];

    // Disjoint + canonical coverage + no obligations => Admit.
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &disjoint_reach,
            &[],
            ReclaimCoverage::CanonicalSuccessor,
        ),
        ReclaimVerdict::Admit
    );

    // M5: protected root or retain_members custody overlapping candidate => Reachable.
    for held in [
        span(1, 0, 0, 0),
        span(1, 0, 5, 5),
        span(1, 0, 9, 9),
        span(2, 7, 0, 5),
    ] {
        assert_eq!(
            push_verdict(
                trace,
                &candidate,
                &[held],
                &[],
                ReclaimCoverage::CanonicalSuccessor,
            ),
            ReclaimVerdict::Reachable
        );
    }

    // M8: DerivedOnly and Uncovered both refuse with Uncovered.
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &disjoint_reach,
            &[],
            ReclaimCoverage::DerivedOnly,
        ),
        ReclaimVerdict::Uncovered
    );
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &disjoint_reach,
            &[],
            ReclaimCoverage::Uncovered,
        ),
        ReclaimVerdict::Uncovered
    );

    // Disjoint reach + canonical coverage + overlapping obligation => Obligated.
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &disjoint_reach,
            &[span(1, 0, 0, 100)],
            ReclaimCoverage::CanonicalSuccessor,
        ),
        ReclaimVerdict::Obligated
    );

    // Refusal priority: Reachable outranks Uncovered and Obligated; Uncovered outranks Obligated.
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &[span(1, 0, 4, 4)],
            &[span(1, 0, 4, 4)],
            ReclaimCoverage::DerivedOnly,
        ),
        ReclaimVerdict::Reachable
    );
    assert_eq!(
        push_verdict(
            trace,
            &candidate,
            &disjoint_reach,
            &[span(1, 0, 4, 4)],
            ReclaimCoverage::Uncovered,
        ),
        ReclaimVerdict::Uncovered
    );

    // M9: Chained-crash resume_release across all three durable barriers.
    for barrier in [
        ReleaseBarrier::IntentDurable,
        ReleaseBarrier::FreeSpacePublished,
        ReleaseBarrier::CompletionDurable,
    ] {
        let plan = resume_release(barrier);
        trace.push(
            "reclaim_resume",
            format!(
                r#""barrier":"{}","publish":{},"complete":{},"release":{}"#,
                barrier_str(barrier),
                plan.publish_free_space,
                plan.record_completion,
                plan.admit_to_allocator,
            ),
        );
    }
}
