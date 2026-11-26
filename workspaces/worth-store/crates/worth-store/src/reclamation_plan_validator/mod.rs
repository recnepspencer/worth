#![allow(unexpected_cfgs)]

//! Pure `ReclamationPlanValidator` kernel for Store Milestone 4 (`M4.5`).
//!
//! Plain `cargo` compiles these exact bodies without `vstd`;
//! `tools/verus/run_verus.py check` proves them against
//! `ReclaimPreservesAllReachableRoots`, `CanonicalCoverageBeforePrune` and
//! `InterruptedReclaimIdempotent`:
//!
//! - a candidate is admitted for a reclaim intent exactly when none of its
//!   spans meets the opaque protected reach, a canonical successor (never a
//!   derived accelerator) covers it, and no unresolved obligation meets it;
//! - each step of a maintenance pass admits an artifact only while the
//!   pass stays within its `ReclaimBudget` slice;
//! - recovery resume releases a range into the allocator at most once.
//!
//! Spans are opaque: the kernel never learns what a space or key names, so
//! upper owners project their protection without lending Store their
//! vocabulary.

#[macro_use]
mod macros;
mod resume;
#[cfg(all(test, not(verus_keep_ghost)))]
mod tests;

pub(crate) use resume::{resume_release, ReleaseBarrier};

#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

verus_items! {
    /// One protected or candidate key span. `key` is an exact coordinate
    /// within `space`; `[first, last]` is an inclusive range under it.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct ReachSpan {
        pub(crate) space: u64,
        pub(crate) key: u64,
        pub(crate) first: u64,
        pub(crate) last: u64,
    }

    /// What covers a candidate's live meaning once it is gone.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReclaimCoverage {
        /// A durably published canonical successor (root or checkpoint).
        CanonicalSuccessor,
        /// Only a derived index or materialization: never sole truth. No
        /// Store producer presents it; the variant pins the refusal.
        #[cfg_attr(not(test), allow(dead_code))]
        DerivedOnly,
        Uncovered,
    }

    /// The bounded slice one incremental maintenance pass may reclaim.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct ReclaimBudget {
        pub(crate) max_artifacts: u64,
        pub(crate) max_bytes: u64,
    }

    /// The reclaim-intent decision, in refusal priority order.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) enum ReclaimVerdict {
        Admit,
        /// A span meets the protected reach.
        Reachable,
        /// No canonical successor covers the candidate.
        Uncovered,
        /// An unresolved retirement or recovery obligation meets it.
        Obligated,
    }
}

#[cfg(verus_keep_ghost)]
verus! {
    spec fn spec_overlaps(a: ReachSpan, b: ReachSpan) -> bool {
        a.space == b.space && a.key == b.key && a.first <= b.last && b.first <= a.last
    }

    spec fn spec_span_clear(c: ReachSpan, reach: Seq<ReachSpan>) -> bool {
        forall|j: int| #![auto] 0 <= j < reach.len() ==> !spec_overlaps(c, reach[j])
    }

    spec fn spec_disjoint(candidate: Seq<ReachSpan>, reach: Seq<ReachSpan>) -> bool {
        forall|i: int| #![auto] 0 <= i < candidate.len() ==> spec_span_clear(candidate[i], reach)
    }

    spec fn spec_admissible(
        candidate: Seq<ReachSpan>,
        reach: Seq<ReachSpan>,
        obligations: Seq<ReachSpan>,
        coverage: ReclaimCoverage,
    ) -> bool {
        spec_disjoint(candidate, reach)
            && coverage == ReclaimCoverage::CanonicalSuccessor
            && spec_disjoint(candidate, obligations)
    }
}

verus_fn! {
    /// Same space, same exact key, intersecting inclusive ranges.
    pub(crate) fn spans_overlap(a: ReachSpan, b: ReachSpan) -> (result: bool);
    ensures { result == spec_overlaps(a, b), }
    body {
        a.space == b.space && a.key == b.key && a.first <= b.last && b.first <= a.last
    }
}

verus_scan_fn! {
    /// `c` meets no span of `reach`.
    fn span_clear(c: ReachSpan, reach: &[ReachSpan]) -> (result: bool);
    ensures { result == spec_span_clear(c, reach@), }
    init {
        let mut j: usize = 0;
    }
    while (j < reach.len())
    invariant {
        0 <= j <= reach@.len(),
        forall|k: int| #![auto] 0 <= k < j ==> !spec_overlaps(c, reach@[k]),
    }
    decreases { reach@.len() - j, }
    step {
        if spans_overlap(c, reach[j]) {
            return false;
        }
        j += 1;
    }
    after {
        true
    }
}

verus_scan_fn! {
    /// `candidate ∩ reach == ∅`.
    pub(crate) fn candidate_disjoint(candidate: &[ReachSpan], reach: &[ReachSpan]) -> (result: bool);
    ensures { result == spec_disjoint(candidate@, reach@), }
    init {
        let mut i: usize = 0;
    }
    while (i < candidate.len())
    invariant {
        0 <= i <= candidate@.len(),
        forall|k: int| #![auto] 0 <= k < i ==> spec_span_clear(candidate@[k], reach@),
    }
    decreases { candidate@.len() - i, }
    step {
        if !span_clear(candidate[i], reach) {
            return false;
        }
        i += 1;
    }
    after {
        true
    }
}

verus_fn! {
    /// The `ReclaimIntent` admission: reach first, then canonical coverage,
    /// then unresolved obligations.
    pub(crate) fn plan_reclaim_intent(
        candidate: &[ReachSpan],
        reach: &[ReachSpan],
        obligations: &[ReachSpan],
        coverage: ReclaimCoverage,
    ) -> (result: ReclaimVerdict);
    ensures {
        (result == ReclaimVerdict::Admit)
            == spec_admissible(candidate@, reach@, obligations@, coverage),
        (result == ReclaimVerdict::Reachable) == !spec_disjoint(candidate@, reach@),
        (result == ReclaimVerdict::Uncovered) == (spec_disjoint(candidate@, reach@)
            && coverage != ReclaimCoverage::CanonicalSuccessor),
        (result == ReclaimVerdict::Obligated) == (spec_disjoint(candidate@, reach@)
            && coverage == ReclaimCoverage::CanonicalSuccessor
            && !spec_disjoint(candidate@, obligations@)),
    }
    body {
        if !candidate_disjoint(candidate, reach) {
            ReclaimVerdict::Reachable
        } else if !matches!(coverage, ReclaimCoverage::CanonicalSuccessor) {
            ReclaimVerdict::Uncovered
        } else if !candidate_disjoint(candidate, obligations) {
            ReclaimVerdict::Obligated
        } else {
            ReclaimVerdict::Admit
        }
    }
}

verus_fn! {
    /// Whether one more artifact of `next_bytes` still fits this pass.
    pub(crate) fn budget_admits(
        selected_artifacts: u64,
        selected_bytes: u64,
        next_bytes: u64,
        budget: ReclaimBudget,
    ) -> (result: bool);
    ensures {
        result == ((selected_artifacts as int) + 1 <= budget.max_artifacts as int
            && (selected_bytes as int) + (next_bytes as int) <= budget.max_bytes as int),
    }
    body {
        selected_artifacts < budget.max_artifacts
            && selected_bytes <= budget.max_bytes
            && next_bytes <= budget.max_bytes - selected_bytes
    }
}
