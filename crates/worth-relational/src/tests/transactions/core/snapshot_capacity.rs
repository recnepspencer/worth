use crate::facade::config::PublicationConfig;
use crate::tests::support::*;

#[test]
fn active_snapshot_capacity_denies_before_handle_or_pin_admission() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: None,
            max_published_snapshot_handles: Some(8),
            max_active_snapshot_handles: Some(1),
            max_transaction_savepoints: 8,
            max_prepared_candidates: Some(8),
            candidate_max_lifetime_millis: 30_000,
        })
        .build();
    let identity = runtime.main_branch_identity();
    let (_, basis) = runtime.observe_branch(&identity).unwrap();
    let first = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();

    assert_eq!(
        runtime
            .snapshots()
            .snapshot_for_observation(&basis.observation()),
        Err(
            crate::visibility::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                maximum_active_snapshots: 1,
            }
        )
    );
    assert_eq!(runtime.visibility.active_snapshot_count(), 1);

    assert!(runtime.snapshots().release_snapshot(&first).is_ok());
    let replacement = runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    assert!(runtime.snapshots().release_snapshot(&replacement).is_ok());
}

#[test]
fn ordinary_snapshot_populations_have_no_policy_and_keep_original_bindings() {
    let runtime = runtime_with_test_schema();
    let (_, basis) = runtime
        .observe_branch(&runtime.main_branch_identity())
        .unwrap();
    // A workload, not a policy fitted to a population. Each call issues a real
    // original handle; descriptor clones never create another registry entry.
    let mut handles = Vec::new();
    for _ in 0..5000 {
        handles.push(
            runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .unwrap(),
        );
    }
    assert_eq!(runtime.visibility.active_snapshot_count(), handles.len());
    let cloned = handles[0].clone();
    assert_eq!(runtime.visibility.active_snapshot_count(), handles.len());
    let request = crate::test_execution_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    assert!(runtime
        .read_truth()
        .read_snapshot(&cloned, &execution)
        .unwrap()
        .is_some());
    for handle in handles.drain(..) {
        runtime.snapshots().release_snapshot(&handle).unwrap();
    }
    assert_eq!(runtime.visibility.active_snapshot_count(), 0);
    assert!(runtime
        .read_truth()
        .read_snapshot(&cloned, &execution)
        .unwrap()
        .is_none());
}
