use worth_foundational::EquivalenceBasisId;
use worth_foundational::FoundationalMergeConflictLocator;
use worth_foundational::{
    foundational_diagnostic_boundary_artifact_subject,
    foundational_diagnostic_branch_candidate_subject,
    foundational_diagnostic_branch_discard_subject, foundational_diagnostic_code,
    foundational_diagnostic_commit_receipt_subject,
    foundational_diagnostic_committed_authority_subject,
    foundational_diagnostic_locator_boundary_artifact, foundational_diagnostic_locator_transition,
    foundational_diagnostic_merge_verdict_subject, foundational_diagnostic_scope,
    sort_foundational_diagnostic_rows, BoundaryArtifactField, BoundaryArtifactId,
    BoundaryArtifactLocator, BoundaryHandle, FoundationalBranchCandidateId, FoundationalBranchId,
    FoundationalCommitId, FoundationalCommitParentBasis, FoundationalCommitReceiptIdentity,
    FoundationalDiagnosticAbsenceCause, FoundationalDiagnosticBreachClass,
    FoundationalDiagnosticComparisonRow, FoundationalDiagnosticDecisionRow,
    FoundationalDiagnosticDenialClass, FoundationalDiagnosticEvidencePosture,
    FoundationalDiagnosticFailureRow, FoundationalDiagnosticLocalityClaim,
    FoundationalDiagnosticOutcomeKind, FoundationalDiagnosticProvenanceReadyRow,
    FoundationalDiagnosticRow, FoundationalDiagnosticRowFamily,
    FoundationalDiagnosticSemanticLabelSet, FoundationalDiagnosticSeverity,
    FoundationalDiagnosticSupportEvidencePosture, FoundationalDiagnosticSupportRow,
    FoundationalDiagnosticWidenedFalloutPosture, FoundationalMergeConflictLocus,
    FoundationalTransitionLocator,
};
use worth_foundational::{FoundationalCommittedDeltaLocator, FoundationalCommittedDeltaLocus};

#[test]
fn outcome_and_absence_families_remain_mechanically_distinct() {
    let mut outcomes = vec![
        FoundationalDiagnosticOutcomeKind::Violation,
        FoundationalDiagnosticOutcomeKind::Accepted,
        FoundationalDiagnosticOutcomeKind::Mismatch,
        FoundationalDiagnosticOutcomeKind::Denied,
    ];
    outcomes.sort();
    assert_eq!(
        outcomes,
        vec![
            FoundationalDiagnosticOutcomeKind::Accepted,
            FoundationalDiagnosticOutcomeKind::Denied,
            FoundationalDiagnosticOutcomeKind::Mismatch,
            FoundationalDiagnosticOutcomeKind::Violation,
        ]
    );

    let mut absence = vec![
        FoundationalDiagnosticAbsenceCause::MissingEvidence,
        FoundationalDiagnosticAbsenceCause::NotRetained,
        FoundationalDiagnosticAbsenceCause::Redacted,
    ];
    absence.sort();
    assert_eq!(
        absence,
        vec![
            FoundationalDiagnosticAbsenceCause::NotRetained,
            FoundationalDiagnosticAbsenceCause::Redacted,
            FoundationalDiagnosticAbsenceCause::MissingEvidence,
        ]
    );
}

#[test]
fn subjects_and_locators_preserve_parity_for_independent_producers() {
    let subject_a =
        foundational_diagnostic_branch_candidate_subject(branch("feature"), candidate(17));
    let subject_b =
        foundational_diagnostic_branch_candidate_subject(branch("feature"), candidate(17));
    assert_eq!(subject_a, subject_b);

    let locator_a = foundational_diagnostic_locator_transition(
        FoundationalTransitionLocator::CommittedDelta(FoundationalCommittedDeltaLocator::new(
            commit(81),
            delta_locus("geometry-face", "face-7 updated"),
        )),
    );
    let locator_b = foundational_diagnostic_locator_transition(
        FoundationalTransitionLocator::CommittedDelta(FoundationalCommittedDeltaLocator::new(
            commit(81),
            delta_locus("geometry-face", "face-7 updated"),
        )),
    );
    assert_eq!(locator_a, locator_b);
}

#[test]
fn family_distinct_rows_stay_blind_consumer_interpretable() {
    let decision = FoundationalDiagnosticDecisionRow::new(
        code("merge.admitted"),
        scope("transition.merge"),
        FoundationalDiagnosticSeverity::Info,
        foundational_diagnostic_merge_verdict_subject(branch("feature"), branch("main")),
        foundational_diagnostic_locator_transition(FoundationalTransitionLocator::MergeConflict(
            FoundationalMergeConflictLocator::new(
                branch("feature"),
                branch("main"),
                FoundationalMergeConflictLocus::new(
                    "geometry-face",
                    "face-7/source",
                    "face-7/target",
                ),
            ),
        )),
        FoundationalDiagnosticOutcomeKind::Accepted,
        labels(["decision", "merge"]),
        None,
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    let failure = FoundationalDiagnosticFailureRow::new(
        code("coverage.omission"),
        scope("diagnostics.coverage"),
        FoundationalDiagnosticSeverity::Failure,
        foundational_diagnostic_boundary_artifact_subject(
            BoundaryArtifactId::new(41),
            BoundaryArtifactField::Payload,
        ),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(41),
            BoundaryArtifactField::Payload,
        )),
        FoundationalDiagnosticOutcomeKind::Violation,
        labels(["coverage", "failure"]),
        FoundationalDiagnosticBreachClass::CoverageOmission,
        FoundationalDiagnosticLocalityClaim::SubjectNeighborhood,
        FoundationalDiagnosticWidenedFalloutPosture::WidenedUnexpected,
    );
    let comparison = FoundationalDiagnosticComparisonRow::new(
        code("diagnostics.parity"),
        scope("diagnostics.comparison"),
        FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_committed_authority_subject(commit(81)),
        foundational_diagnostic_locator_transition(FoundationalTransitionLocator::CommitParentage(
            worth_foundational::FoundationalCommitParentageLocator::new(
                commit(81),
                FoundationalCommitParentBasis::new(EquivalenceBasisId::new(401)),
            ),
        )),
        FoundationalDiagnosticOutcomeKind::Mismatch,
        labels(["comparison"]),
        None,
        FoundationalDiagnosticEvidencePosture::Summarized,
    );
    let support = FoundationalDiagnosticSupportRow::new(
        code("support.present"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_commit_receipt_subject(commit(81), receipt(33)),
        foundational_diagnostic_locator_transition(FoundationalTransitionLocator::CommitParentage(
            worth_foundational::FoundationalCommitParentageLocator::new(
                commit(81),
                FoundationalCommitParentBasis::new(EquivalenceBasisId::new(401)),
            ),
        )),
        FoundationalDiagnosticOutcomeKind::Partial,
        labels(["support"]),
        FoundationalDiagnosticSupportEvidencePosture::Present(
            FoundationalDiagnosticEvidencePosture::RetainedDirect,
        ),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    let provenance = FoundationalDiagnosticProvenanceReadyRow::new(
        code("provenance.ready"),
        scope("diagnostics.provenance"),
        FoundationalDiagnosticSeverity::Info,
        foundational_diagnostic_branch_discard_subject(branch("feature")),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(77),
            BoundaryArtifactField::Basis,
        )),
        FoundationalDiagnosticOutcomeKind::Deferred,
        labels(["provenance"]),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(77),
            BoundaryArtifactField::Proofs,
        )),
        FoundationalDiagnosticEvidencePosture::Reconstructed,
    );

    let rows = [
        FoundationalDiagnosticRow::Decision(decision.clone()),
        FoundationalDiagnosticRow::Failure(failure.clone()),
        FoundationalDiagnosticRow::Comparison(comparison.clone()),
        FoundationalDiagnosticRow::Support(support.clone()),
        FoundationalDiagnosticRow::ProvenanceReady(provenance.clone()),
    ];

    assert_eq!(rows[0].family(), FoundationalDiagnosticRowFamily::Decision);
    assert_eq!(rows[1].family(), FoundationalDiagnosticRowFamily::Failure);
    assert_eq!(
        rows[2].family(),
        FoundationalDiagnosticRowFamily::Comparison
    );
    assert_eq!(rows[3].family(), FoundationalDiagnosticRowFamily::Support);
    assert_eq!(
        rows[4].family(),
        FoundationalDiagnosticRowFamily::ProvenanceReady
    );
    assert_eq!(
        decision.outcome_kind(),
        FoundationalDiagnosticOutcomeKind::Accepted
    );
    assert_eq!(
        failure.breach_class(),
        FoundationalDiagnosticBreachClass::CoverageOmission
    );
    assert_eq!(
        comparison.evidence_posture(),
        FoundationalDiagnosticEvidencePosture::Summarized
    );
    assert_eq!(
        support.evidence_posture(),
        &FoundationalDiagnosticSupportEvidencePosture::Present(
            FoundationalDiagnosticEvidencePosture::RetainedDirect
        )
    );
    assert_eq!(
        provenance.evidence_posture(),
        FoundationalDiagnosticEvidencePosture::Reconstructed
    );
}

#[test]
fn omitted_support_rows_are_construction_bugs_not_implied_denials() {
    let omitted = FoundationalDiagnosticSupportRow::new(
        code("support.omitted"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Failure,
        foundational_diagnostic_committed_authority_subject(commit(91)),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(91),
            BoundaryArtifactField::Payload,
        )),
        FoundationalDiagnosticOutcomeKind::Violation,
        labels(["support", "omission"]),
        FoundationalDiagnosticSupportEvidencePosture::OmittedConstructionBug(
            FoundationalDiagnosticBreachClass::CoverageOmission,
        ),
        FoundationalDiagnosticLocalityClaim::WidenedScope,
        FoundationalDiagnosticWidenedFalloutPosture::WidenedUnexpected,
    );

    let absent = FoundationalDiagnosticSupportRow::new(
        code("support.absent"),
        scope("diagnostics.support"),
        FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_committed_authority_subject(commit(91)),
        foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
            BoundaryArtifactId::new(91),
            BoundaryArtifactField::Payload,
        )),
        FoundationalDiagnosticOutcomeKind::Unsupported,
        labels(["support", "absence"]),
        FoundationalDiagnosticSupportEvidencePosture::Absent(
            FoundationalDiagnosticAbsenceCause::NotRetained,
        ),
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );

    assert_ne!(omitted.evidence_posture(), absent.evidence_posture());
}

#[test]
fn canonical_row_order_is_stable_across_input_order() {
    let mut rows = vec![
        FoundationalDiagnosticRow::Support(FoundationalDiagnosticSupportRow::new(
            code("zeta"),
            scope("diagnostics.support"),
            FoundationalDiagnosticSeverity::Advisory,
            foundational_diagnostic_committed_authority_subject(commit(5)),
            foundational_diagnostic_locator_boundary_artifact(BoundaryArtifactLocator::new(
                BoundaryArtifactId::new(5),
                BoundaryArtifactField::Payload,
            )),
            FoundationalDiagnosticOutcomeKind::Partial,
            labels(["zeta"]),
            FoundationalDiagnosticSupportEvidencePosture::Absent(
                FoundationalDiagnosticAbsenceCause::Redacted,
            ),
            FoundationalDiagnosticLocalityClaim::ExactSubject,
            FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
        )),
        FoundationalDiagnosticRow::Decision(FoundationalDiagnosticDecisionRow::new(
            code("alpha"),
            scope("diagnostics.decision"),
            FoundationalDiagnosticSeverity::Info,
            foundational_diagnostic_branch_candidate_subject(branch("feature"), candidate(1)),
            foundational_diagnostic_locator_transition(
                FoundationalTransitionLocator::BranchCandidate(
                    worth_foundational::FoundationalBranchCandidateLocator::new(
                        branch("feature"),
                        candidate(1),
                    ),
                ),
            ),
            FoundationalDiagnosticOutcomeKind::Accepted,
            labels(["alpha"]),
            Some(FoundationalDiagnosticDenialClass::DomainDenied),
            FoundationalDiagnosticLocalityClaim::ExactSubject,
            FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
        )),
    ];

    sort_foundational_diagnostic_rows(&mut rows);

    assert_eq!(rows[0].family(), FoundationalDiagnosticRowFamily::Decision);
    assert_eq!(rows[1].family(), FoundationalDiagnosticRowFamily::Support);
}

fn code(value: &str) -> worth_foundational::FoundationalDiagnosticCodeId {
    foundational_diagnostic_code(value).expect("valid diagnostic code")
}

fn scope(value: &str) -> worth_foundational::FoundationalDiagnosticScopeId {
    foundational_diagnostic_scope(value).expect("valid diagnostic scope")
}

fn branch(name: &str) -> FoundationalBranchId {
    FoundationalBranchId::new(name).expect("valid branch id")
}

fn candidate(value: u64) -> FoundationalBranchCandidateId {
    FoundationalBranchCandidateId::new(BoundaryHandle::new(value))
}

fn commit(value: u64) -> FoundationalCommitId {
    FoundationalCommitId::new(BoundaryHandle::new(value))
}

fn receipt(value: u64) -> FoundationalCommitReceiptIdentity {
    FoundationalCommitReceiptIdentity::new(BoundaryHandle::new(value))
}

fn delta_locus(category: &str, detail: &str) -> FoundationalCommittedDeltaLocus {
    FoundationalCommittedDeltaLocus::new(category, detail)
}

fn labels<const N: usize>(values: [&str; N]) -> FoundationalDiagnosticSemanticLabelSet {
    FoundationalDiagnosticSemanticLabelSet::new(values.into_iter().map(code))
}
