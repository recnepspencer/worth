use worth_relational::facade::runtime::{InvariantDecisionKind, InvariantExecutionPoint};

use super::{close, ReceiptFact, RequirementFact};

const REQUIRED: (&str, u16, u16, InvariantExecutionPoint) =
    ("required", 1, 0, InvariantExecutionPoint::CommitBoundary);
const EXTRA: (&str, u16, u16, InvariantExecutionPoint) =
    ("unused", 1, 0, InvariantExecutionPoint::CommitBoundary);

fn receipt(
    key: (&'static str, u16, u16, InvariantExecutionPoint),
    decision: InvariantDecisionKind,
) -> ReceiptFact<'static> {
    ReceiptFact {
        key,
        decision,
        bound: true,
        work: 1,
    }
}

fn required() -> RequirementFact<'static> {
    RequirementFact {
        key: REQUIRED,
        maximum_work: 2,
    }
}

#[test]
fn required_non_applicability_and_unrequested_non_applicability_close() {
    let receipts = [
        receipt(REQUIRED, InvariantDecisionKind::NotApplicable),
        receipt(EXTRA, InvariantDecisionKind::NotApplicable),
    ];
    assert!(close(&receipts, &[required()], 3).is_ok());
}

#[test]
fn unrequested_execution_and_failed_required_decisions_deny() {
    let receipts = [
        receipt(REQUIRED, InvariantDecisionKind::Passed),
        receipt(EXTRA, InvariantDecisionKind::Passed),
    ];
    assert!(close(&receipts, &[required()], 0).is_err());
    for decision in [
        InvariantDecisionKind::Violated,
        InvariantDecisionKind::Advisory,
    ] {
        assert!(close(&[receipt(REQUIRED, decision)], &[required()], 0).is_err());
    }
}

#[test]
fn ordering_candidate_binding_and_work_are_required() {
    let passed = receipt(REQUIRED, InvariantDecisionKind::Passed);
    assert!(close(&[passed, passed], &[required()], 0).is_err());
    assert!(close(
        &[receipt(EXTRA, InvariantDecisionKind::NotApplicable), passed],
        &[required()],
        0
    )
    .is_err());
    assert!(close(
        &[ReceiptFact {
            bound: false,
            ..passed
        }],
        &[required()],
        0
    )
    .is_err());
    assert!(close(
        &[
            passed,
            ReceiptFact {
                bound: false,
                ..receipt(EXTRA, InvariantDecisionKind::NotApplicable)
            }
        ],
        &[required()],
        0
    )
    .is_err());
    assert!(close(&[ReceiptFact { work: 3, ..passed }], &[required()], 0).is_err());
}
