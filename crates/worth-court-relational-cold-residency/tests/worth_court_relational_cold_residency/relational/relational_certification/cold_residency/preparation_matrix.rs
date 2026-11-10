//! Six typed refusal classes and current-only invariant demands preserve public effects.
use super::{
    candidate_reads::PreparationDoor,
    candidate_source::{recovered, REMOTE},
    lineage_event_matrix::FAILURES,
};
use std::{collections::BTreeSet, sync::atomic::Ordering};
use worth_relational::facade::{
    durability::RelationalDurableMemberLocator,
    storage::{RelationalStorageSubject, RelationalStorageUnavailable},
};
use worth_testkit_relational::durable_medium::immutable_members::ColdRead;
#[derive(Default)]
struct Placements {
    heads: BTreeSet<RelationalDurableMemberLocator>,
    history: BTreeSet<RelationalDurableMemberLocator>,
}

#[test]
fn residency_failure_matrix_remote_invariant_heads_cover_every_preparation_door_and_class() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in PreparationDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = recovered(execution);
            let regions = fixture
                .directory
                .mirror
                .iter()
                .chain(
                    fixture
                        .directory
                        .roots
                        .iter()
                        .flat_map(|root| &root.regions),
                )
                .filter(|region| region.partition_id == REMOTE);
            let mut placements = Placements::default();
            for region in regions {
                placements.heads.insert(region.head);
                placements.history.insert(region.history);
            }
            let runtime = fixture.runtime;
            let source = fixture.source;
            let medium = fixture.medium;
            assert!(!placements.heads.is_empty());
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = medium.offered().len();
            let diagnostics = runtime.publication().diagnostics_snapshot().diagnostics;
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
            assert_eq!(medium.offered().len(), offered);
            assert_eq!(
                runtime.publication().diagnostics_snapshot().diagnostics,
                diagnostics
            );
            source.failures.lock().unwrap().clear();
            let reads = source.reads.load(Ordering::Relaxed);
            if sticky {
                assert_eq!(door.run(execution, &runtime), Err(failure));
                assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                assert_eq!(medium.offered().len(), offered);
                assert_eq!(
                    runtime.publication().diagnostics_snapshot().diagnostics,
                    diagnostics
                );
            } else {
                door.run(execution, &runtime)
                    .unwrap_or_else(|failure| panic!("retry {door:?} x {cause:?}: {failure:?}"));
                assert!(source.reads.load(Ordering::Relaxed) > reads);
                if matches!(door, PreparationDoor::Commit) {
                    assert!(runtime.history().latest_patch_stream_position() > frontier);
                    assert!(medium.offered().len() > offered);
                } else {
                    assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
                    assert_eq!(medium.offered().len(), offered);
                }
            }
        }
    }
}

#[test]
fn residency_failure_matrix_remote_current_invariants_never_consult_history() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in PreparationDoor::ALL {
        for (cause, _) in FAILURES {
            let fixture = recovered(execution);
            let regions = fixture
                .directory
                .mirror
                .iter()
                .chain(
                    fixture
                        .directory
                        .roots
                        .iter()
                        .flat_map(|root| &root.regions),
                )
                .filter(|region| region.partition_id == REMOTE);
            let mut placements = Placements::default();
            for region in regions {
                placements.heads.insert(region.head);
                placements.history.insert(region.history);
            }
            let runtime = fixture.runtime;
            let source = fixture.source;
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
