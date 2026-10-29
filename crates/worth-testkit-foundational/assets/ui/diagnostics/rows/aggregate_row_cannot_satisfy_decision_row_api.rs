use worth_foundational::{
    foundational_diagnostic_branch_candidate_subject, foundational_diagnostic_code,
    foundational_diagnostic_locator_transition, foundational_diagnostic_scope, BoundaryHandle,
    FoundationalBranchCandidateId, FoundationalBranchCandidateLocator, FoundationalBranchId,
    FoundationalDiagnosticDecisionRow, FoundationalDiagnosticLocalityClaim,
    FoundationalDiagnosticOutcomeKind, FoundationalDiagnosticRow,
    FoundationalDiagnosticSemanticLabelSet, FoundationalDiagnosticSeverity,
    FoundationalDiagnosticWidenedFalloutPosture, FoundationalTransitionLocator,
};

fn needs_decision_row(_row: FoundationalDiagnosticDecisionRow) {}

fn main() {
    let branch = FoundationalBranchId::new("feature").unwrap();
    let candidate = FoundationalBranchCandidateId::new(BoundaryHandle::new(1));
    let row = FoundationalDiagnosticDecisionRow::new(
        foundational_diagnostic_code("merge.admitted").unwrap(),
        foundational_diagnostic_scope("transition.merge").unwrap(),
        FoundationalDiagnosticSeverity::Info,
        foundational_diagnostic_branch_candidate_subject(branch.clone(), candidate),
        foundational_diagnostic_locator_transition(FoundationalTransitionLocator::BranchCandidate(
            FoundationalBranchCandidateLocator::new(branch, candidate),
        )),
        FoundationalDiagnosticOutcomeKind::Accepted,
        FoundationalDiagnosticSemanticLabelSet::new([]),
        None,
        FoundationalDiagnosticLocalityClaim::ExactSubject,
        FoundationalDiagnosticWidenedFalloutPosture::NotWidened,
    );
    let aggregate = FoundationalDiagnosticRow::Decision(row);
    needs_decision_row(aggregate);
}
