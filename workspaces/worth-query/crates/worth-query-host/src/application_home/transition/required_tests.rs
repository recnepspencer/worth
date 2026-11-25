use super::{index_maintenance_denial_is_budget_stop, transition_denial_is_budget_stop, Denial};
use worth_relational::facade::{
    content::RelationalContentRejection,
    errors::{ErrorContext, ErrorOperation, RelationalSubsystem},
    indexes::{
        DerivedIndexId, DerivedIndexMaintenanceDenial, DerivedIndexMaintenanceDenialKind,
        DerivedIndexMaintenanceWork, RelationalExecutionDenialCause,
    },
    transactions::{
        CommitExecutionDenial, CommitExecutionDenialKind, CommitLog, TransactionCommitError,
    },
};

fn commit_execution(cause: RelationalExecutionDenialCause) -> Denial {
    Denial::Commit(TransactionCommitError::Execution {
        denial: CommitExecutionDenial {
            kind: CommitExecutionDenialKind::Cause(cause),
            partition_identity: None,
        },
        context: ErrorContext::new(RelationalSubsystem::Transaction, ErrorOperation::Commit),
        commit_log: CommitLog::new(),
    })
}

#[test]
fn index_denial_classifier_separates_budget_stops_from_structural_and_content_refusals() {
    assert!(index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::WorkBudgetExceeded
    ));
    assert!(index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::ColdRecordSlotBudgetExceeded
    ));
    assert!(index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::Execution(
            RelationalExecutionDenialCause::WorkExhausted
        )
    ));
    assert!(index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::Execution(RelationalExecutionDenialCause::Cancelled)
    ));
    assert!(index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::Execution(
            RelationalExecutionDenialCause::PolicyMemoryExhausted {
                requested: 1024,
                admitted: 512,
                ancestor: 0,
            }
        )
    ));

    // Checkpoint content refusals carried in `Execution(_)` are structural,
    // NOT budget/cancellation stops.
    assert!(!index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::Execution(
            RelationalExecutionDenialCause::ContentInconsistency
        )
    ));
    assert!(!index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::Execution(RelationalExecutionDenialCause::Content(
            RelationalContentRejection::IntegrityMismatch
        ))
    ));

    // Structural index denials remain non-budget.
    assert!(!index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::IndexUnavailable(DerivedIndexId(1))
    ));
    assert!(!index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::PriorEntryMismatch
    ));
    assert!(!index_maintenance_denial_is_budget_stop(
        &DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired
    ));

    // Native-emitted `ColdRecordSlotBudgetExceeded` is a budget stop;
    // genuine structural `ColdReconstructionRequired` is not.
    assert!(transition_denial_is_budget_stop(&Denial::Indexes(
        DerivedIndexMaintenanceDenial {
            kind: DerivedIndexMaintenanceDenialKind::ColdRecordSlotBudgetExceeded,
            work: DerivedIndexMaintenanceWork::default(),
        }
    )));
    assert!(!transition_denial_is_budget_stop(&Denial::Indexes(
        DerivedIndexMaintenanceDenial {
            kind: DerivedIndexMaintenanceDenialKind::ColdReconstructionRequired,
            work: DerivedIndexMaintenanceWork::default(),
        }
    )));

    // Native index preparation routes `Execution(cause)` through
    // `TransactionCommitError::Execution`; preserve budget vs content split.
    assert!(transition_denial_is_budget_stop(&commit_execution(
        RelationalExecutionDenialCause::WorkExhausted
    )));
    assert!(transition_denial_is_budget_stop(&commit_execution(
        RelationalExecutionDenialCause::Cancelled
    )));
    assert!(!transition_denial_is_budget_stop(&commit_execution(
        RelationalExecutionDenialCause::ContentInconsistency
    )));
    assert!(!transition_denial_is_budget_stop(&commit_execution(
        RelationalExecutionDenialCause::Content(RelationalContentRejection::IntegrityMismatch)
    )));
}
