use super::*;

#[test]
fn residency_failure_matrix_index_doors_preserve_every_failure_class() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    for door in IndexDoor::ALL {
        for (cause, sticky) in FAILURES {
            let fixture = super::super::cold_index_fixture::reopened(execution);
            let runtime = fixture.runtime;
            let source = fixture.source;
            let entity = fixture.entity;
            let index = fixture.index;
            let generation = fixture.generation;
            let member = fixture.member;
            assert!(runtime
                .read_truth()
                .visible_entity_at_version(execution, entity, runtime.current_version_id())
                .unwrap()
                .is_some());
            let (_, observation_basis) = runtime
                .observe_branch(&runtime.main_branch_identity())
                .unwrap();
            let observed_reads = source.reads.load(Ordering::Relaxed);
            assert_eq!(
                runtime
                    .index_access()
                    .generation_residency_for_observation(index, &observation_basis.observation()),
                crate::indexes::data::DerivedIndexGenerationResidency::DeferredImage
            );
            assert_eq!(
                runtime.index_access().generation_residency_for_observation(
                    DerivedIndexId(0),
                    &observation_basis.observation()
                ),
                crate::indexes::data::DerivedIndexGenerationResidency::Absent
            );
            assert_eq!(source.reads.load(Ordering::Relaxed), observed_reads);
            let frontier = runtime.history().latest_patch_stream_position();
            let offered = attached_test_medium(&runtime).offered().len();
            let reservations = runtime.patch_position_reservation_counters();
            source
                .failures
                .lock()
                .unwrap()
                .insert(ColdRead::Member(member), cause);
            let expected = Execution::StorageUnavailable(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Index(index),
                cause,
            });
            assert_eq!(
                door.read(&runtime, execution, index, &generation),
                Err(expected),
                "{door:?} x {cause:?}: a cold index must not become absence"
            );
            assert_eq!(runtime.history().latest_patch_stream_position(), frontier);
            assert_eq!(attached_test_medium(&runtime).offered().len(), offered);
            assert_eq!(
                runtime.patch_position_reservation_counters(),
                reservations,
                "index refusal is pre-position"
            );
            source.failures.lock().unwrap().clear();
            let reads = source.reads.load(Ordering::Relaxed);
            let failed_reads = || {
                source
                    .member_reads
                    .lock()
                    .unwrap()
                    .iter()
                    .filter(|read| **read == member)
                    .count()
            };
            let failed_before = failed_reads();
            if sticky {
                assert_eq!(
                    door.read(&runtime, execution, index, &generation),
                    Err(expected)
                );
                if matches!(
                    door,
                    IndexDoor::MemoryCheckpoint | IndexDoor::EmbeddedCheckpoint
                ) {
                    assert_eq!(failed_reads(), failed_before);
                } else {
                    assert_eq!(source.reads.load(Ordering::Relaxed), reads);
                }
            } else {
                assert!(
                    door.read(&runtime, execution, index, &generation).unwrap(),
                    "retry must retain a genuine generation"
                );
                assert!(source.reads.load(Ordering::Relaxed) > reads);
            }
        }
    }
}
