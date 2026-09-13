use crate::branch::{
    SignalBranchAdvanceDenial, SignalBranchForkOperationDenial, SignalBranchRestoreDenial,
    SignalBranchRetirementDenial, SignalBranchSnapshotCaptureDenial,
};
use crate::state::SignalBranchId;

use super::super::branch_execution_cell::advance::map_advance_cell_denial;
use super::super::branch_execution_cell::fork::map_fork_cell_denial;
use super::super::branch_execution_cell::restoration::map_restore_cell_denial;
use super::super::branch_execution_cell::retirement::map_retirement_cell_denial;
use super::super::branch_execution_cell::snapshot::map_snapshot_cell_denial;
use super::super::SignalBranchCellAdmissionDenial;

#[derive(Clone, Copy)]
enum ExpectedCellPosture {
    OwnerUnavailable,
    OwnerCellMisuse,
    OwnerReentry,
    RetirementInProgress,
    RetiredBranch,
    QuarantinedBranch,
}

const COMPLETE_CELL_DENIALS: [(SignalBranchCellAdmissionDenial, ExpectedCellPosture); 7] = [
    (
        SignalBranchCellAdmissionDenial::ForeignOwner,
        ExpectedCellPosture::OwnerUnavailable,
    ),
    (
        SignalBranchCellAdmissionDenial::ExpiredLifecycle,
        ExpectedCellPosture::OwnerUnavailable,
    ),
    (
        SignalBranchCellAdmissionDenial::SecondCellWhileHeld,
        ExpectedCellPosture::OwnerCellMisuse,
    ),
    (
        SignalBranchCellAdmissionDenial::ExecutingThreadReentry,
        ExpectedCellPosture::OwnerReentry,
    ),
    (
        SignalBranchCellAdmissionDenial::RetirementInProgress,
        ExpectedCellPosture::RetirementInProgress,
    ),
    (
        SignalBranchCellAdmissionDenial::RetiredIncarnation,
        ExpectedCellPosture::RetiredBranch,
    ),
    (
        SignalBranchCellAdmissionDenial::PoisonedIncarnation,
        ExpectedCellPosture::QuarantinedBranch,
    ),
];

#[test]
fn every_operation_preserves_reachable_cell_posture_without_unknown_fallback() {
    let branch_id = SignalBranchId(41);
    let mut completed_mappings = 0;
    for (denial, expected) in COMPLETE_CELL_DENIALS {
        assert_advance_posture(denial, expected, branch_id);
        completed_mappings += 1;
        assert_fork_posture(denial, expected, branch_id);
        completed_mappings += 1;
        assert_snapshot_posture(denial, expected, branch_id);
        completed_mappings += 1;
        assert_restore_posture(denial, expected, branch_id);
        completed_mappings += 1;
        assert_retirement_posture(denial, expected, branch_id);
        completed_mappings += 1;
    }
    assert_eq!(COMPLETE_CELL_DENIALS.len(), 7);
    assert_eq!(completed_mappings, 35);
}

fn assert_advance_posture(
    denial: SignalBranchCellAdmissionDenial,
    expected: ExpectedCellPosture,
    branch_id: SignalBranchId,
) {
    let mapped = map_advance_cell_denial(denial, branch_id);
    assert!(match (expected, mapped) {
        (ExpectedCellPosture::OwnerUnavailable, SignalBranchAdvanceDenial::OwnerUnavailable(_))
        | (ExpectedCellPosture::OwnerReentry, SignalBranchAdvanceDenial::OwnerReentry) => true,
        (
            ExpectedCellPosture::OwnerCellMisuse,
            SignalBranchAdvanceDenial::OwnerCellMisuse {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetirementInProgress,
            SignalBranchAdvanceDenial::RetirementInProgress {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetiredBranch,
            SignalBranchAdvanceDenial::RetiredBranch {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::QuarantinedBranch,
            SignalBranchAdvanceDenial::QuarantinedBranch {
                branch_id: observed,
            },
        ) => observed == branch_id,
        _ => false,
    });
}

fn assert_fork_posture(
    denial: SignalBranchCellAdmissionDenial,
    expected: ExpectedCellPosture,
    branch_id: SignalBranchId,
) {
    let mapped = map_fork_cell_denial(denial, branch_id);
    assert!(match (expected, mapped) {
        (
            ExpectedCellPosture::OwnerUnavailable,
            SignalBranchForkOperationDenial::OwnerUnavailable(_),
        )
        | (ExpectedCellPosture::OwnerReentry, SignalBranchForkOperationDenial::OwnerReentry) => {
            true
        }
        (
            ExpectedCellPosture::OwnerCellMisuse,
            SignalBranchForkOperationDenial::OwnerCellMisuse {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetirementInProgress,
            SignalBranchForkOperationDenial::RetirementInProgress {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetiredBranch,
            SignalBranchForkOperationDenial::RetiredBranch {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::QuarantinedBranch,
            SignalBranchForkOperationDenial::QuarantinedBranch {
                branch_id: observed,
            },
        ) => observed == branch_id,
        _ => false,
    });
}

fn assert_snapshot_posture(
    denial: SignalBranchCellAdmissionDenial,
    expected: ExpectedCellPosture,
    branch_id: SignalBranchId,
) {
    let mapped = map_snapshot_cell_denial(denial, branch_id);
    assert!(match (expected, mapped) {
        (
            ExpectedCellPosture::OwnerUnavailable,
            SignalBranchSnapshotCaptureDenial::OwnerUnavailable(_),
        )
        | (ExpectedCellPosture::OwnerReentry, SignalBranchSnapshotCaptureDenial::OwnerReentry) =>
            true,
        (
            ExpectedCellPosture::OwnerCellMisuse,
            SignalBranchSnapshotCaptureDenial::OwnerCellMisuse {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetirementInProgress,
            SignalBranchSnapshotCaptureDenial::RetirementInProgress {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetiredBranch,
            SignalBranchSnapshotCaptureDenial::RetiredBranch {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::QuarantinedBranch,
            SignalBranchSnapshotCaptureDenial::QuarantinedBranch {
                branch_id: observed,
            },
        ) => observed == branch_id,
        _ => false,
    });
}

fn assert_restore_posture(
    denial: SignalBranchCellAdmissionDenial,
    expected: ExpectedCellPosture,
    branch_id: SignalBranchId,
) {
    let mapped = map_restore_cell_denial(denial, branch_id);
    assert!(match (expected, mapped) {
        (ExpectedCellPosture::OwnerUnavailable, SignalBranchRestoreDenial::OwnerUnavailable(_))
        | (ExpectedCellPosture::OwnerReentry, SignalBranchRestoreDenial::OwnerReentry) => true,
        (
            ExpectedCellPosture::OwnerCellMisuse,
            SignalBranchRestoreDenial::OwnerCellMisuse {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetirementInProgress,
            SignalBranchRestoreDenial::RetirementInProgress {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetiredBranch,
            SignalBranchRestoreDenial::RetiredBranch {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::QuarantinedBranch,
            SignalBranchRestoreDenial::QuarantinedBranch {
                branch_id: observed,
            },
        ) => observed == branch_id,
        _ => false,
    });
}

fn assert_retirement_posture(
    denial: SignalBranchCellAdmissionDenial,
    expected: ExpectedCellPosture,
    branch_id: SignalBranchId,
) {
    let mapped = map_retirement_cell_denial(denial, branch_id);
    assert!(match (expected, mapped) {
        (
            ExpectedCellPosture::OwnerUnavailable,
            SignalBranchRetirementDenial::OwnerUnavailable(_),
        )
        | (ExpectedCellPosture::OwnerReentry, SignalBranchRetirementDenial::OwnerReentry) => true,
        (
            ExpectedCellPosture::OwnerCellMisuse,
            SignalBranchRetirementDenial::OwnerCellMisuse {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetirementInProgress,
            SignalBranchRetirementDenial::RetirementInProgress {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::RetiredBranch,
            SignalBranchRetirementDenial::RetiredBranch {
                branch_id: observed,
            },
        )
        | (
            ExpectedCellPosture::QuarantinedBranch,
            SignalBranchRetirementDenial::QuarantinedBranch {
                branch_id: observed,
            },
        ) => observed == branch_id,
        _ => false,
    });
}
