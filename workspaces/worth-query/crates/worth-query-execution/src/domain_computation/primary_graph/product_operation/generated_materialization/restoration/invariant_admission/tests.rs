use super::*;
use crate::domain_computation::primary_graph::WorthQueryProducerInvariantRequirement;

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
    }
}

#[test]
fn producer_restoration_requires_one_exact_passing_owner_receipt() {
    assert_eq!(
        admit_requirement_facts([fact(InvariantDecisionKind::Passed)], REQUIREMENT),
        Ok(())
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
        InvariantDecisionKind::NotApplicable,
        InvariantDecisionKind::Advisory,
        InvariantDecisionKind::Violated,
    ] {
        assert_eq!(
            admit_requirement_facts([fact(verdict)], REQUIREMENT),
            Err(Denial::RequiredInvariantDidNotPass)
        );
    }
}
