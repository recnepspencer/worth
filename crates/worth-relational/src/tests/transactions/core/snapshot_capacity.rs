use crate::facade::config::PublicationConfig;
use crate::tests::support::*;

#[test]
fn active_snapshot_capacity_denies_before_handle_or_pin_admission() {
    let runtime = RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::AiWorkflow)
        .schema_registry(test_schema_registry())
        .publication(PublicationConfig {
            coherent_publication_required: true,
            max_patch_records_per_commit: 4_096,
            max_published_snapshot_handles: 8,
            max_active_snapshot_handles: 1,
            max_transaction_savepoints: 8,
            max_prepared_candidates: 8,
            candidate_max_lifetime_millis: 30_000,
            max_prepared_root_bytes: 268_435_456,
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
