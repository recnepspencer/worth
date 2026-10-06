//! Compare decisive meanings; scheduling and unavailable evidence cannot pass.

use super::verification::{
    ConsumedOutputVerification, ConsumedOutputVerificationStop, EvidenceView,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    FullVerificationDecision, FullVerificationImage, FullVerificationStop,
    InvalidationEditAdmission,
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DecisiveMeaning {
    Current,
    Changed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EquivalenceComparison {
    Equivalent,
    Inconclusive(ConsumedOutputVerificationStop),
    Mismatch {
        marked: DecisiveMeaning,
        full: DecisiveMeaning,
    },
}

pub(super) fn check<'evidence>(
    roots: impl ExactSizeIterator<Item = EvidenceView<'evidence>>,
    image: Result<FullVerificationImage<'_>, FullVerificationStop>,
    marked: Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
    admission: &mut InvalidationEditAdmission,
) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
    // The independent observer runs even when optimized verification stops.
    // It cannot clear pending obligations or promote incomplete evidence.
    let full = image.and_then(|image| {
        for root in roots {
            if image.verify_source_view(root, admission)? == FullVerificationDecision::Changed {
                return Ok(FullVerificationDecision::Changed);
            }
        }
        Ok(FullVerificationDecision::Current)
    });
    match compare(marked, full) {
        EquivalenceComparison::Equivalent => marked,
        EquivalenceComparison::Inconclusive(stop) => Err(stop),
        EquivalenceComparison::Mismatch { marked, full } => {
            panic!("invalidation equivalence mismatch: marking={marked:?}, full={full:?}")
        }
    }
}

fn compare(
    marked: Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
    full: Result<FullVerificationDecision, FullVerificationStop>,
) -> EquivalenceComparison {
    let marked = match marked {
        Ok(ConsumedOutputVerification::Current) => DecisiveMeaning::Current,
        Ok(
            ConsumedOutputVerification::ChangedDirectFact(_)
            | ConsumedOutputVerification::ChangedUpstream,
        ) => DecisiveMeaning::Changed,
        Err(stop) => return EquivalenceComparison::Inconclusive(stop),
    };
    let full = match full {
        Ok(FullVerificationDecision::Current) => DecisiveMeaning::Current,
        Ok(FullVerificationDecision::Changed) => DecisiveMeaning::Changed,
        Err(FullVerificationStop::Admission(
            CompanionPreflightStop::WorkExhausted { .. } | CompanionPreflightStop::WorkCounterOverflow,
        ) | FullVerificationStop::SourceRead(
            crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded,
        )) => return EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::WorkExhausted),
        Err(FullVerificationStop::Admission(CompanionPreflightStop::Interrupted(event))) => {
            return EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::Interrupted(
                event,
            ))
        }
        Err(FullVerificationStop::ActorImageChanged) => return EquivalenceComparison::Inconclusive(
            ConsumedOutputVerificationStop::RetryCurrentness(worth_relational::facade::mvcc::CompanionCellEditStop::TopologyGenerationChanged)),
        Err(_) => return EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::Unavailable),
    };
    if marked == full {
        EquivalenceComparison::Equivalent
    } else {
        EquivalenceComparison::Mismatch { marked, full }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_ordinals_compare_by_meaning_and_pending_is_never_equivalent() {
        assert_eq!(
            compare(
                Ok(ConsumedOutputVerification::ChangedDirectFact(7)),
                Ok(FullVerificationDecision::Changed)
            ),
            EquivalenceComparison::Equivalent
        );
        assert_eq!(
            compare(
                Err(ConsumedOutputVerificationStop::PendingUpstream),
                Ok(FullVerificationDecision::Current)
            ),
            EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::PendingUpstream)
        );
        assert!(matches!(
            compare(
                Ok(ConsumedOutputVerification::Current),
                Ok(FullVerificationDecision::Changed)
            ),
            EquivalenceComparison::Mismatch { .. }
        ));
        assert_eq!(
            compare(
                Ok(ConsumedOutputVerification::Current),
                Err(FullVerificationStop::OutputEvidenceUnavailable)
            ),
            EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::Unavailable)
        );
    }
}
