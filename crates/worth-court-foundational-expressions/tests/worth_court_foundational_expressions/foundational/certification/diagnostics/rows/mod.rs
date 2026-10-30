#[allow(unused_imports)]
use crate::certification::ordinary_context::diagnostics::rows::*;
use {
    worth_foundational::foundational_diagnostic_boundary_artifact_subject,
    worth_foundational::foundational_diagnostic_branch_candidate_subject,
    worth_foundational::foundational_diagnostic_branch_discard_subject,
    worth_foundational::foundational_diagnostic_commit_receipt_subject,
    worth_foundational::foundational_diagnostic_committed_authority_subject,
    worth_foundational::foundational_diagnostic_locator_boundary_artifact,
    worth_foundational::foundational_diagnostic_locator_transition,
    worth_foundational::foundational_diagnostic_merge_verdict_subject,
    worth_foundational::sort_foundational_diagnostic_rows,
};

#[test]
fn outcome_and_absence_families_remain_mechanically_distinct() {
    let mut outcomes = vec![
        worth_foundational::FoundationalDiagnosticOutcomeKind::Violation,
        worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
        worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
        worth_foundational::FoundationalDiagnosticOutcomeKind::Denied,
    ];
    outcomes.sort();
    assert_eq!(
        outcomes,
        vec![
            worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
            worth_foundational::FoundationalDiagnosticOutcomeKind::Denied,
            worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
            worth_foundational::FoundationalDiagnosticOutcomeKind::Violation,
        ]
    );

    let mut absence = vec![
        worth_foundational::FoundationalDiagnosticAbsenceCause::MissingEvidence,
        worth_foundational::FoundationalDiagnosticAbsenceCause::NotRetained,
        worth_foundational::FoundationalDiagnosticAbsenceCause::Redacted,
    ];
    absence.sort();
    assert_eq!(
        absence,
        vec![
            worth_foundational::FoundationalDiagnosticAbsenceCause::NotRetained,
            worth_foundational::FoundationalDiagnosticAbsenceCause::Redacted,
            worth_foundational::FoundationalDiagnosticAbsenceCause::MissingEvidence,
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
        worth_foundational::FoundationalTransitionLocator::CommittedDelta(
            worth_foundational::FoundationalCommittedDeltaLocator::new(
                commit(81),
                delta_locus("geometry-face", "face-7 updated"),
            ),
        ),
    );
    let locator_b = foundational_diagnostic_locator_transition(
        worth_foundational::FoundationalTransitionLocator::CommittedDelta(
            worth_foundational::FoundationalCommittedDeltaLocator::new(
                commit(81),
                delta_locus("geometry-face", "face-7 updated"),
            ),
        ),
    );
    assert_eq!(locator_a, locator_b);
}

#[test]
fn family_distinct_rows_stay_blind_consumer_interpretable() {
    let decision = worth_foundational::FoundationalDiagnosticDecisionRow::new(
        code("merge.admitted"),
        scope("transition.merge"),
        worth_foundational::FoundationalDiagnosticSeverity::Info,
        foundational_diagnostic_merge_verdict_subject(branch("feature"), branch("main")),
        foundational_diagnostic_locator_transition(
            worth_foundational::FoundationalTransitionLocator::MergeConflict(
                worth_foundational::FoundationalMergeConflictLocator::new(
                    branch("feature"),
                    branch("main"),
                    worth_foundational::FoundationalMergeConflictLocus::new(
                        "geometry-face",
                        "face-7/source",
                        "face-7/target",
                    ),
                ),
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
        labels(["decision", "merge"]),
        None,
        worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    let failure = worth_foundational::FoundationalDiagnosticFailureRow::new(
        code("coverage.omission"),
        scope("diagnostics.coverage"),
        worth_foundational::FoundationalDiagnosticSeverity::Failure,
        foundational_diagnostic_boundary_artifact_subject(
            worth_foundational::BoundaryArtifactId::new(41),
            worth_foundational::BoundaryArtifactField::Payload,
        ),
        foundational_diagnostic_locator_boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(
                worth_foundational::BoundaryArtifactId::new(41),
                worth_foundational::BoundaryArtifactField::Payload,
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Violation,
        labels(["coverage", "failure"]),
        worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission,
        worth_foundational::FoundationalDiagnosticLocalityClaim::SubjectNeighborhood,
        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::WidenedUnexpected,
    );
    let comparison = worth_foundational::FoundationalDiagnosticComparisonRow::new(
        code("diagnostics.parity"),
        scope("diagnostics.comparison"),
        worth_foundational::FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_committed_authority_subject(commit(81)),
        foundational_diagnostic_locator_transition(
            worth_foundational::FoundationalTransitionLocator::CommitParentage(
                worth_foundational::FoundationalCommitParentageLocator::new(
                    commit(81),
                    worth_foundational::FoundationalCommitParentBasis::new(
                        worth_foundational::EquivalenceBasisId::new(401),
                    ),
                ),
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Mismatch,
        labels(["comparison"]),
        None,
        worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized,
    );
    let support = worth_foundational::FoundationalDiagnosticSupportRow::new(
        code("support.present"),
        scope("diagnostics.support"),
        worth_foundational::FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_commit_receipt_subject(commit(81), receipt(33)),
        foundational_diagnostic_locator_transition(
            worth_foundational::FoundationalTransitionLocator::CommitParentage(
                worth_foundational::FoundationalCommitParentageLocator::new(
                    commit(81),
                    worth_foundational::FoundationalCommitParentBasis::new(
                        worth_foundational::EquivalenceBasisId::new(401),
                    ),
                ),
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Partial,
        labels(["support"]),
        worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect,
        ),
        worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    let provenance = worth_foundational::FoundationalDiagnosticProvenanceReadyRow::new(
        code("provenance.ready"),
        scope("diagnostics.provenance"),
        worth_foundational::FoundationalDiagnosticSeverity::Info,
        foundational_diagnostic_branch_discard_subject(branch("feature")),
        foundational_diagnostic_locator_boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(
                worth_foundational::BoundaryArtifactId::new(77),
                worth_foundational::BoundaryArtifactField::Basis,
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Deferred,
        labels(["provenance"]),
        foundational_diagnostic_locator_boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(
                worth_foundational::BoundaryArtifactId::new(77),
                worth_foundational::BoundaryArtifactField::Proofs,
            ),
        ),
        worth_foundational::FoundationalDiagnosticEvidencePosture::Reconstructed,
    );

    let rows = [
        worth_foundational::FoundationalDiagnosticRow::Decision(decision.clone()),
        worth_foundational::FoundationalDiagnosticRow::Failure(failure.clone()),
        worth_foundational::FoundationalDiagnosticRow::Comparison(comparison.clone()),
        worth_foundational::FoundationalDiagnosticRow::Support(support.clone()),
        worth_foundational::FoundationalDiagnosticRow::ProvenanceReady(provenance.clone()),
    ];

    assert_eq!(
        rows[0].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Decision
    );
    assert_eq!(
        rows[1].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Failure
    );
    assert_eq!(
        rows[2].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Comparison
    );
    assert_eq!(
        rows[3].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Support
    );
    assert_eq!(
        rows[4].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::ProvenanceReady
    );
    assert_eq!(
        decision.outcome_kind(),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted
    );
    assert_eq!(
        failure.breach_class(),
        worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission
    );
    assert_eq!(
        comparison.evidence_posture(),
        worth_foundational::FoundationalDiagnosticEvidencePosture::Summarized
    );
    assert_eq!(
        support.evidence_posture(),
        &worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Present(
            worth_foundational::FoundationalDiagnosticEvidencePosture::RetainedDirect
        )
    );
    assert_eq!(
        provenance.evidence_posture(),
        worth_foundational::FoundationalDiagnosticEvidencePosture::Reconstructed
    );
}

#[test]
fn omitted_support_rows_are_construction_bugs_not_implied_denials() {
    let omitted = worth_foundational::FoundationalDiagnosticSupportRow::new(
        code("support.omitted"),
        scope("diagnostics.support"),
        worth_foundational::FoundationalDiagnosticSeverity::Failure,
        foundational_diagnostic_committed_authority_subject(commit(91)),
        foundational_diagnostic_locator_boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(
                worth_foundational::BoundaryArtifactId::new(91),
                worth_foundational::BoundaryArtifactField::Payload,
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Violation,
        labels(["support", "omission"]),
        worth_foundational::FoundationalDiagnosticSupportEvidencePosture::OmittedConstructionBug(
            worth_foundational::FoundationalDiagnosticBreachClass::CoverageOmission,
        ),
        worth_foundational::FoundationalDiagnosticLocalityClaim::WidenedScope,
        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::WidenedUnexpected,
    );

    let absent = worth_foundational::FoundationalDiagnosticSupportRow::new(
        code("support.absent"),
        scope("diagnostics.support"),
        worth_foundational::FoundationalDiagnosticSeverity::Advisory,
        foundational_diagnostic_committed_authority_subject(commit(91)),
        foundational_diagnostic_locator_boundary_artifact(
            worth_foundational::BoundaryArtifactLocator::new(
                worth_foundational::BoundaryArtifactId::new(91),
                worth_foundational::BoundaryArtifactField::Payload,
            ),
        ),
        worth_foundational::FoundationalDiagnosticOutcomeKind::Unsupported,
        labels(["support", "absence"]),
        worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Absent(
            worth_foundational::FoundationalDiagnosticAbsenceCause::NotRetained,
        ),
        worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
        worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );

    assert_ne!(omitted.evidence_posture(), absent.evidence_posture());
}

#[test]
fn canonical_row_order_is_stable_across_input_order() {
    let mut rows = vec![
        worth_foundational::FoundationalDiagnosticRow::Support(
            worth_foundational::FoundationalDiagnosticSupportRow::new(
                code("zeta"),
                scope("diagnostics.support"),
                worth_foundational::FoundationalDiagnosticSeverity::Advisory,
                foundational_diagnostic_committed_authority_subject(commit(5)),
                foundational_diagnostic_locator_boundary_artifact(
                    worth_foundational::BoundaryArtifactLocator::new(
                        worth_foundational::BoundaryArtifactId::new(5),
                        worth_foundational::BoundaryArtifactField::Payload,
                    ),
                ),
                worth_foundational::FoundationalDiagnosticOutcomeKind::Partial,
                labels(["zeta"]),
                worth_foundational::FoundationalDiagnosticSupportEvidencePosture::Absent(
                    worth_foundational::FoundationalDiagnosticAbsenceCause::Redacted,
                ),
                worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
                worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
            ),
        ),
        worth_foundational::FoundationalDiagnosticRow::Decision(
            worth_foundational::FoundationalDiagnosticDecisionRow::new(
                code("alpha"),
                scope("diagnostics.decision"),
                worth_foundational::FoundationalDiagnosticSeverity::Info,
                foundational_diagnostic_branch_candidate_subject(branch("feature"), candidate(1)),
                foundational_diagnostic_locator_transition(
                    worth_foundational::FoundationalTransitionLocator::BranchCandidate(
                        worth_foundational::FoundationalBranchCandidateLocator::new(
                            branch("feature"),
                            candidate(1),
                        ),
                    ),
                ),
                worth_foundational::FoundationalDiagnosticOutcomeKind::Accepted,
                labels(["alpha"]),
                Some(worth_foundational::FoundationalDiagnosticDenialClass::DomainDenied),
                worth_foundational::FoundationalDiagnosticLocalityClaim::ExactSubject,
                worth_foundational::FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
            ),
        ),
    ];

    sort_foundational_diagnostic_rows(&mut rows);

    assert_eq!(
        rows[0].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Decision
    );
    assert_eq!(
        rows[1].family(),
        worth_foundational::FoundationalDiagnosticRowFamily::Support
    );
}
