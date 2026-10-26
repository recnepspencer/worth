//! A discarded current invariant round preserves typed faults and side effects.
use super::super::cold_read_matrix::FAILURES;
use super::super::cold_source::ColdRead;
use super::*;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

impl PreparationDoor {
    const ALL: [Self; 3] = [Self::Validation, Self::Candidate, Self::Commit];
    fn run(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &crate::runtime::RelationalRuntime,
    ) -> Result<(), RelationalStorageUnavailable> {
        let transaction = transaction(execution, runtime);
        match self {
            Self::Validation => transaction.validate(execution, runtime).map(|_| ()),
            Self::Candidate => runtime
                .prepare_branch_transaction(execution, transaction)
                .map(|_| ()),
            Self::Commit => transaction.commit(execution, runtime).map(|_| ()),
        }
        .map_err(storage)
    }
}
#[test]
fn residency_failure_matrix_remote_invariant_heads_cover_every_preparation_door_and_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in PreparationDoor::ALL {
        for cause in FAILURES {
            let (runtime, source, _, placements) = recovered(execution);
            assert!(!placements.heads.is_empty());
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&runtime).offered().len();
            let diagnostics = runtime.publication.diagnostics.snapshot();
            let symbols = runtime.services.symbols.snapshot();
            let before = runtime.performance_access().counters();
            source.failures.lock().unwrap().extend(
                placements
                    .heads
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            let failure = door.run(execution, &runtime).unwrap_err();
            assert_eq!(
                failure,
                RelationalStorageUnavailable {
                    subject: RelationalStorageSubject::Region(REMOTE),
                    cause
                },
                "{door:?}"
            );
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
            assert_eq!(runtime.publication.diagnostics.snapshot(), diagnostics);
            assert_eq!(runtime.services.symbols.snapshot(), symbols);
            let after = runtime.performance_access().counters();
            assert_eq!(
                after.preparation_packet_count,
                before.preparation_packet_count
            );
            assert_eq!(
                after.invariant_entity_slot_scans,
                before.invariant_entity_slot_scans
            );
            source.failures.lock().unwrap().clear();
            let reads = source.reads.load(Ordering::Relaxed);
            if cause.is_proven_damage() {
                assert_eq!(door.run(execution, &runtime), Err(failure));
                assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
                assert_eq!(runtime.publication.diagnostics.snapshot(), diagnostics);
                assert_eq!(runtime.services.symbols.snapshot(), symbols);
            } else {
                door.run(execution, &runtime)
                    .unwrap_or_else(|failure| panic!("retry {door:?} x {cause:?}: {failure:?}"));
                assert!(source.reads.load(Ordering::Relaxed) > reads);
                if matches!(door, PreparationDoor::Commit) {
                    assert!(runtime.history().latest_patch_stream_position() > frontier);
                    assert!(attached_test_medium(&runtime).offered().len() > offered);
                } else {
                    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                    assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
                }
            }
        }
    }
}
#[test]
fn residency_failure_matrix_remote_current_invariants_never_consult_history() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in PreparationDoor::ALL {
        for cause in FAILURES {
            let (runtime, source, _, placements) = recovered(execution);
            assert!(!placements.history.is_empty());
            source.failures.lock().unwrap().extend(
                placements
                    .history
                    .iter()
                    .map(|member| (ColdRead::Member(*member), cause)),
            );
            door.run(execution, &runtime).unwrap();
            assert!(
                !source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .any(|member| placements.history.contains(member)),
                "{door:?} x {cause:?}"
            );
        }
    }
}
