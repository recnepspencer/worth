//! Compare decisive meanings; scheduling and unavailable evidence cannot pass.

use super::verification::ConsumedOutputVerification;
use super::verification::ConsumedOutputVerificationStop;
use super::verification::EvidenceView;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationDecision;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationImage;
use crate::domain_computation::primary_graph::output_lineage::invalidation::FullVerificationStop;

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
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    roots: impl ExactSizeIterator<Item = EvidenceView<'evidence>>,
    image: Result<FullVerificationImage<'_>, FullVerificationStop>,
    marked: Result<ConsumedOutputVerification, ConsumedOutputVerificationStop>,
) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
    // The independent observer runs even when optimized verification stops.
    // It cannot clear pending obligations or promote incomplete evidence.
    let full = image.and_then(|image| {
        for root in roots {
            if image.verify_source_view(execution, root)? == FullVerificationDecision::Changed {
                return Ok(FullVerificationDecision::Changed);
            }
        }
        Ok(FullVerificationDecision::Current)
    });
    match compare(marked, full) {
        EquivalenceComparison::Equivalent => marked,
        // The full oracle cannot discharge a native pending obligation.
        // Preserve that non-current semantic result without claiming equivalence.
        EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::PendingUpstream)
            if marked == Ok(ConsumedOutputVerification::PendingUpstream) =>
        {
            marked
        }
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
            | ConsumedOutputVerification::ChangedUpstream
            | ConsumedOutputVerification::ChangedOutput
            | ConsumedOutputVerification::Unaligned,
        ) => DecisiveMeaning::Changed,
        Ok(ConsumedOutputVerification::PendingUpstream) => {
            return EquivalenceComparison::Inconclusive(
                ConsumedOutputVerificationStop::PendingUpstream,
            )
        }
        Err(stop) => return EquivalenceComparison::Inconclusive(stop),
    };
    let full = match full {
        Ok(FullVerificationDecision::Current) => DecisiveMeaning::Current,
        Ok(FullVerificationDecision::Changed) => DecisiveMeaning::Changed,
        Err(FullVerificationStop::Admission(stop)) => {
            return EquivalenceComparison::Inconclusive(super::verification::map_admission_stop(stop));
        }
        Err(FullVerificationStop::SourceRead(crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::Execution(cause))) => return EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::Execution(cause)),
        Err(FullVerificationStop::SourceRead(
            crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded,
        )) => return EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::WorkExhausted),
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
    use super::super::VerificationCapacityStop;
    use super::*;
    use worth_relational::facade::mvcc::CompanionPreflightStop;

    #[test]
    fn source_request_stops_keep_their_value_in_equivalence() {
        use crate::domain_computation::primary_graph::application_attempt::WorthQuerySourceCurrentnessFailure as Read;
        use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
        for cause in [
            Cause::Cancelled,
            Cause::DeadlineElapsed,
            Cause::WorkExhausted,
        ] {
            assert_eq!(
                compare(
                    Ok(ConsumedOutputVerification::Current),
                    Err(FullVerificationStop::SourceRead(Read::Execution(cause)))
                ),
                EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::Execution(
                    cause
                ))
            );
        }
    }

    #[test]
    fn observer_capacity_is_not_missing_evidence() {
        let stop = CompanionPreflightStop::RetainedCompanionCapacityExhausted {
            requested: 1,
            retained: 1,
            maximum: 1,
        };
        assert_eq!(
            compare(
                Ok(ConsumedOutputVerification::Current),
                Err(FullVerificationStop::Admission(stop)),
            ),
            EquivalenceComparison::Inconclusive(ConsumedOutputVerificationStop::CapacityExhausted(
                VerificationCapacityStop::Meter(stop)
            )),
        );
    }

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
