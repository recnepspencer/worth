//! Store's physical reclamation planner (`M4.5`, `D5`).
//!
//! Every holder that keeps physical bytes alive projects an opaque
//! [`ProtectedReachSet`] for one candidate's window: in-process root
//! registrations (which carry upper owners' read leases and staged checkpoint
//! captures), durable root pins with their transitive ancestors, and
//! unresolved retirement or recovery obligations. The verified
//! `ReclamationPlanValidator` alone decides admission, so a reclaim intent
//! never meets the reach, never prunes before a canonical successor covers
//! the candidate, and never overlaps an unresolved obligation.

use super::{DisplacedArtifact, PhysicalRetirementDenial, RetiredArtifact};
use crate::reclamation_plan_validator::{
    plan_reclaim_intent, ReachSpan, ReclaimCoverage, ReclaimVerdict,
};

/// Root generations; every candidate carries one span here.
const ROOT_SPACE: u64 = 1;
/// Inline segment tails, keyed by segment, ranged by generation.
const INLINE_TAIL_SPACE: u64 = 2;

/// The window a holder must project its protection for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum ReachWindow {
    /// Exactly one root, plus the inline tail it names.
    Root {
        generation: u64,
        inline_tail: (u64, u64),
    },
    /// Every root at or below `generation`.
    RootsAtOrBelow(u64),
}

/// The opaque union of everything that still reaches a candidate's window.
#[derive(Debug, Default)]
pub(in crate::physical_runtime) struct ProtectedReachSet {
    spans: Vec<ReachSpan>,
}

impl ProtectedReachSet {
    pub(in crate::physical_runtime) fn protect_root(&mut self, generation: u64) {
        self.spans.push(root_span(generation, generation));
    }

    pub(in crate::physical_runtime) fn protect_inline_tail(
        &mut self,
        segment: u64,
        generation: u64,
    ) {
        self.spans.push(inline_span(segment, generation));
    }

    /// A holder whose state is not yet known (unrestored pins) protects all.
    pub(in crate::physical_runtime) fn protect_all_roots(&mut self) {
        self.spans.push(root_span(0, u64::MAX));
    }

    #[cfg(test)]
    pub(in crate::physical_runtime) fn len(&self) -> usize {
        self.spans.len()
    }
}

const fn inline_span(segment: u64, generation: u64) -> ReachSpan {
    ReachSpan {
        space: INLINE_TAIL_SPACE,
        key: segment,
        first: generation,
        last: generation,
    }
}

const fn root_span(first: u64, last: u64) -> ReachSpan {
    ReachSpan {
        space: ROOT_SPACE,
        key: 0,
        first,
        last,
    }
}

/// The key spans one displaced artifact occupies while any root reads it.
#[derive(Debug, Clone, Copy)]
pub(in crate::physical_runtime) struct ReclaimCandidate {
    spans: [ReachSpan; 2],
    len: usize,
    window: ReachWindow,
}

impl ReclaimCandidate {
    pub(in crate::physical_runtime) fn of(displaced: &DisplacedArtifact) -> Self {
        let source = displaced.source_root;
        match displaced.artifact {
            RetiredArtifact::Segment {
                segment,
                generation,
            } => Self {
                spans: [root_span(source, source), inline_span(segment, generation)],
                len: 2,
                window: ReachWindow::Root {
                    generation: source,
                    inline_tail: (segment, generation),
                },
            },
            // Every root from the one that placed the extent generation through
            // the rewrite's source root reads it.
            RetiredArtifact::Extent { .. } => Self::at_or_below(source),
            RetiredArtifact::Arena { .. } => Self::at_or_below(source.saturating_sub(1)),
        }
    }

    const fn at_or_below(generation: u64) -> Self {
        let span = root_span(0, generation);
        Self {
            spans: [span, span],
            len: 1,
            window: ReachWindow::RootsAtOrBelow(generation),
        }
    }

    pub(in crate::physical_runtime) const fn window(&self) -> ReachWindow {
        self.window
    }

    fn spans(&self) -> &[ReachSpan] {
        &self.spans[..self.len]
    }
}

/// `ReclaimIntent` admission through the verified validator. `covered` is
/// true only when a durably published canonical successor root no longer
/// reads the candidate; `unresolved` is any other pending root-changing
/// publication or recovered retirement obligation.
pub(in crate::physical_runtime) fn plan_reclaim(
    candidate: &ReclaimCandidate,
    reach: &ProtectedReachSet,
    covered: bool,
    unresolved: bool,
) -> Option<PhysicalRetirementDenial> {
    let coverage = if covered {
        ReclaimCoverage::CanonicalSuccessor
    } else {
        ReclaimCoverage::Uncovered
    };
    let obligation = [root_span(0, u64::MAX)];
    let obligations: &[ReachSpan] = if unresolved { &obligation } else { &[] };
    match plan_reclaim_intent(candidate.spans(), &reach.spans, obligations, coverage) {
        ReclaimVerdict::Admit => None,
        ReclaimVerdict::Reachable => Some(PhysicalRetirementDenial::Protected),
        ReclaimVerdict::Uncovered => Some(PhysicalRetirementDenial::Retained),
        ReclaimVerdict::Obligated => Some(PhysicalRetirementDenial::Unresolved),
    }
}

/// Canonical coverage before prune: the completed checkpoint captured a
/// canonical successor root past the candidate's source root (an arena is
/// forgotten by the root that publishes its removal, so that root suffices).
pub(in crate::physical_runtime) fn checkpoint_covers(
    displaced: &DisplacedArtifact,
    captured_root: u64,
) -> bool {
    let canonical = match displaced.artifact {
        RetiredArtifact::Arena { .. } => captured_root >= displaced.source_root,
        _ => captured_root > displaced.source_root,
    };
    let coverage = if canonical {
        ReclaimCoverage::CanonicalSuccessor
    } else {
        ReclaimCoverage::Uncovered
    };
    plan_reclaim_intent(&[], &[], &[], coverage) == ReclaimVerdict::Admit
}

#[cfg(test)]
#[path = "reclaim_plan_tests.rs"]
mod tests;
