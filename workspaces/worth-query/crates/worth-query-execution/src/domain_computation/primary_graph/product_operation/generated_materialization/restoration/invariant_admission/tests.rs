use super::*;
use crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement;
use CustomInvariantNonApplicability as Basis;

const REQUIREMENT: WorthQueryProducerInvariantRequirement =
    WorthQueryProducerInvariantRequirement::new(
        "positive-turn",
        1,
        0,
        ApplicationInvariantExecutionPoint::CommitBoundary,
    );

fn fact(verdict: InvariantDecisionKind) -> ReceiptFact<'static> {
    ReceiptFact {
        identifier: "positive-turn",
        major: 1,
        minor: 0,
        execution_point: InvariantExecutionPoint::CommitBoundary,
        verdict,
        non_applicability: None,
    }
}

fn not_applicable(basis: Basis) -> ReceiptFact<'static> {
    ReceiptFact {
        non_applicability: Some(basis),
        ..fact(InvariantDecisionKind::NotApplicable)
    }
}

#[test]
fn producer_restoration_requires_one_exact_passing_owner_receipt() {
    assert_eq!(
        admit_requirement_facts([fact(InvariantDecisionKind::Passed)], REQUIREMENT),
        Ok(RequirementSatisfaction::Affirmed)
    );
    assert_eq!(
        admit_requirement_facts([], REQUIREMENT),
        Err(Denial::MissingRequiredInvariant)
    );
    assert_eq!(
        admit_requirement_facts(
            [ReceiptFact {
                minor: 1,
                ..fact(InvariantDecisionKind::Passed)
            }],
            REQUIREMENT,
        ),
        Err(Denial::RequiredInvariantVersionMismatch)
    );
    assert_eq!(
        admit_requirement_facts([fact(InvariantDecisionKind::Passed); 2], REQUIREMENT,),
        Err(Denial::DuplicateRequiredInvariant)
    );
    for verdict in [
        InvariantDecisionKind::Advisory,
        InvariantDecisionKind::Violated,
    ] {
        assert_eq!(
            admit_requirement_facts([fact(verdict)], REQUIREMENT),
            Err(Denial::RequiredInvariantDidNotPass)
        );
    }
}

#[test]
fn a_candidate_outside_the_rule_closure_satisfies_the_requirement_vacuously() {
    assert_eq!(
        admit_requirement_facts(
            [not_applicable(Basis::CandidateOutsideClosure)],
            REQUIREMENT
        ),
        Ok(RequirementSatisfaction::Vacuous)
    );
}

/// Kills the mutant that admits every not-applicable receipt: scoped or
/// refused non-applicability never proves the publication left the closure.
#[test]
fn scoped_or_unexplained_non_applicability_still_refuses() {
    for receipt in [
        not_applicable(Basis::NoApplicableTouch),
        not_applicable(Basis::PreparationRefused),
        fact(InvariantDecisionKind::NotApplicable),
    ] {
        assert_eq!(
            admit_requirement_facts([receipt], REQUIREMENT),
            Err(Denial::RequiredInvariantDidNotPass)
        );
    }
    // A basis never upgrades a verdict that is not a non-applicability.
    assert_eq!(
        admit_requirement_facts(
            [ReceiptFact {
                non_applicability: Some(Basis::CandidateOutsideClosure),
                ..fact(InvariantDecisionKind::Violated)
            }],
            REQUIREMENT,
        ),
        Err(Denial::RequiredInvariantDidNotPass)
    );
}
