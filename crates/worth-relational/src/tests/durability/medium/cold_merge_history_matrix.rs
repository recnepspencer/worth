//! Every merge history door must consult an indexed nonhead source envelope.
#[path = "cold_merge_history_fixture.rs"]
mod fixture;
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::merge::data::{
    MergeExecutionPreparationError, MergeExecutionRequest, MergeIntent, MergePlanningError,
    MergePlanningRequest,
};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};
use crate::transactions::data::{CommitPreparationReason, RecordRef, TransactionCommitError};
use fixture::{branches, Fixture};

#[derive(Debug, Clone, Copy)]
enum Door {
    HistoryPlan,
    IndependentPreparation,
    TransactionParents,
}
impl Door {
    const ALL: [Self; 3] = [
        Self::HistoryPlan,
        Self::IndependentPreparation,
        Self::TransactionParents,
    ];
    fn run(
        self,
        fixture: &Fixture,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<usize, RelationalStorageUnavailable> {
        let (target, source) = branches();
        match self {
            Self::HistoryPlan => {
                let plan = fixture
                    .runtime
                    .merge_access()
                    .plan_history_scope_for_test(
                        execution,
                        MergePlanningRequest::new(target, source, MergeIntent::ReconcileIntoTarget),
                    )
                    .map_err(planning_failure)?;
                let overlap = RecordRef::Entity(fixture.overlap);
                assert!(plan
                    .target_delta
                    .touched_records
                    .iter()
                    .any(|row| row.target == overlap));
                let row = plan
                    .source_delta
                    .touched_records
                    .iter()
                    .find(|row| row.target == overlap)
                    .expect("the intermediate change remains in the source delta");
                assert!(row
                    .commit_ids()
                    .any(|id| id == fixture.intermediate.commit_id));
                Ok(row.envelopes.len())
            }
            Self::IndependentPreparation => {
                match fixture.runtime.prepare_merge_execution(
                    execution,
                    MergeExecutionRequest::new(target, source, MergeIntent::ReconcileIntoTarget),
                ) {
                    Err(MergeExecutionPreparationError::Planning(error)) => {
                        Err(planning_failure(error))
                    }
                    Err(MergeExecutionPreparationError::NotExecutionReady(report)) => {
                        assert!(report
                            .denied_records
                            .iter()
                            .any(|row| row.record == RecordRef::Entity(fixture.overlap)));
                        Ok(report.denied_records.len())
                    }
                    other => panic!("the real overlap needs explicit resolution: {other:?}"),
                }
            }
            Self::TransactionParents => {
                let transaction =
                    test_owner_begin_merge_transaction(&fixture.runtime, target, [source]);
                match transaction.commit(execution, &fixture.runtime) {
                    Err(TransactionCommitError::Preparation { error, .. }) => {
                        match error.reason() {
                            CommitPreparationReason::StorageUnavailable(failure) => Err(failure),
                            other => panic!("exact cold preparation refusal: {other:?}"),
                        }
                    }
                    Err(TransactionCommitError::Conflict { error, .. }) => {
                        assert_eq!(
                            error.code,
                            crate::diagnostics::data::DiagnosticCode::MergeConflictOverlap
                        );
                        Ok(1)
                    }
                    other => panic!("an older source overlap cannot publish a merge: {other:?}"),
                }
            }
        }
    }
}
fn planning_failure(error: MergePlanningError) -> RelationalStorageUnavailable {
    match error {
        MergePlanningError::Execution(Execution::StorageUnavailable(failure)) => failure,
        other => panic!("exact cold planning refusal: {other:?}"),
    }
}
#[test]
fn a_cold_intermediate_source_commit_cannot_disappear_from_merge_parent_conflicts() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let fixture = Fixture::reopened(execution);
    let before = fixture.runtime.history().latest_patch_stream_position();
    assert!(Door::TransactionParents.run(&fixture, execution).unwrap() > 0);
    assert_eq!(
        fixture.runtime.history().latest_patch_stream_position(),
        before
    );
}
#[test]
fn residency_failure_matrix_nonhead_merge_history_covers_every_door_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in Door::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = Fixture::reopened(execution);
            let frontier = fixture.runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&fixture.runtime).offered().len();
            let reservations = fixture.runtime.patch_position_reservation_counters();
            fixture
                .source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Commit(fixture.position), cause);
            let expected = RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Envelope(fixture.position),
                cause,
            };
            let actual = door.run(&fixture, execution);
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier,
                "{door:?} must not reserve a position"
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            assert_eq!(actual, Err(expected), "{door:?} x {cause:?}");
            fixture.source.failures.lock().unwrap().clear();
            let reads = || {
                fixture
                    .source
                    .commit_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|position| **position == fixture.position)
                    .count()
            };
            let before = reads();
            if sticky {
                assert_eq!(door.run(&fixture, execution), Err(expected));
                assert_eq!(reads(), before);
            } else {
                assert!(door.run(&fixture, execution).unwrap() > 0);
                assert!(reads() > before);
            }
            assert_eq!(
                fixture.runtime.history().latest_patch_stream_position(),
                frontier
            );
            assert_eq!(
                fixture.runtime.patch_position_reservation_counters(),
                reservations
            );
            assert_eq!(
                attached_test_medium(&fixture.runtime).offered().len(),
                offered
            );
        }
    }
}
