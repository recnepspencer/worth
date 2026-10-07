use super::*;

pub(super) fn structural_correspondence(
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    candidate_budget: usize,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    let [subject] = left.rows() else {
        return stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            "structural correspondence requires exactly one subject row on the left basis",
            counters,
        );
    };
    let subject_identity = subject.identity().clone();
    let mut candidates = right
        .rows()
        .iter()
        .map(|row| row.identity().evidence_identity().as_str().to_string())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.dedup();
    resolve_correspondence(
        CorrespondenceEvaluationRequest::structural_only(
            candidates,
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            candidate_budget,
            StructuralCandidateOrderingContract::StableFingerprintOrder,
        ),
        subject_identity,
        pair,
        counters.resolve_correspondence(),
    )
}

pub(super) fn lineage_correspondence(
    left: WorthQueryReadResult,
    right: WorthQueryReadResult,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    let ([left_row], [right_row]) = (left.rows(), right.rows()) else {
        return stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            "authoritative lineage comparison requires exactly one row on each basis",
            counters,
        );
    };
    resolve_correspondence(
        CorrespondenceEvaluationRequest::lineage_only(
            left_row.identity().evidence_identity().as_str(),
            right_row.identity().evidence_identity().as_str(),
            StructuralCandidateDiscoveryPlan::IndexBackedBounded,
            1,
        ),
        left_row.identity().clone(),
        pair,
        counters.resolve_correspondence(),
    )
}

fn resolve_correspondence(
    request: CorrespondenceEvaluationRequest,
    subject: crate::memory_workspace::WorthQueryEntityIdentity,
    pair: WorthQueryComparisonBasisPairEvidence,
    counters: WorthQueryComparisonJourneyCounters,
) -> WorthQueryComparisonOutcome {
    match resolve_correspondence_evidence(request) {
        Ok(correspondence) if correspondence.outcome().as_denied().is_none() => {
            let posture = if correspondence.outcome().as_lineage_continuity().is_some() {
                WorthQueryComparisonCorrespondencePosture::AuthoritativeContinuity
            } else {
                WorthQueryComparisonCorrespondencePosture::Advisory
            };
            WorthQueryComparisonOutcome::Correspondence(WorthQueryComparisonCorrespondence::new(
                subject,
                correspondence,
                posture,
                pair,
                counters,
            ))
        }
        Ok(correspondence) => stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::NarrowCandidates,
            correspondence
                .outcome()
                .as_denied()
                .map(|denial| denial.reason())
                .unwrap_or("correspondence was denied"),
            counters,
        ),
        Err(error) => stopped(
            WorthQueryComparisonStopSource::CorrespondenceDenied,
            WorthQueryComparisonNextAction::ResolveAuthority,
            format!("correspondence resolution failed: {error:?}"),
            counters,
        ),
    }
}
