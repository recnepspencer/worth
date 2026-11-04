//! A real initialization successor occupies its own axis even when C stays zero.
use super::*;

#[test]
fn birth_checkpoint_reopens_with_archived_empty_installation_tail_at_zero_commits() {
    let policy = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&policy);
    let medium = TestDurableMedium::new();
    let mut runtime = worth_testkit_relational::durable_publication::durable_runtime(&medium);
    // The closed fresh owner issues the move-only opportunity while its actual
    // source is absent. Birth-only admission precedes any transition publication.
    let permission = runtime.begin_fresh_home_initialization(&request).unwrap();
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            &request,
            RelationalDurableRecoverySource::new(None, vec![], vec![], source),
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .admit_fresh_home_source(&request, plan)
        .unwrap();
    let birth = runtime
        .complete_checkpoint_service()
        .prepare(&request, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    let birth_coordinate = birth.source_artifact().initial_coordinate();
    assert_eq!(birth_coordinate.0, 1);
    assert_eq!(birth.source_artifact().covered().0, 0);
    medium.file_checkpoint(birth.checkpoint().clone());
    let main = runtime.main_branch_identity();
    runtime.archive_branch(&request, &main).unwrap();
    runtime
        .prepare_fresh_home_schema_installation(&request, permission)
        .unwrap()
        .install_with_managed_sources(&request, additions(), vec![], vec![])
        .unwrap();
    let installed = runtime
        .complete_checkpoint_service()
        .prepare(&request, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    assert_eq!(installed.source_artifact().covered().0, 0);
    assert_eq!(installed.source_artifact().initial_coordinate().0, 2);
    assert_eq!(installed.initial_predecessor(), Some(birth_coordinate.1));
    let input = home_image_input(&medium, &birth, &request).unwrap();
    drop((installed, birth, runtime));
    let mut receiver = receiver(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiver
        .durability_recovery()
        .recover_with_authority(&request, plan)
        .unwrap();
    let complete = receiver
        .complete_checkpoint_service()
        .prepare(&request, RelationalCheckpointArtifactPolicy::Embedded)
        .unwrap();
    assert_eq!(complete.source_artifact().covered().0, 0);
    assert_eq!(complete.source_artifact().covered_transitions().0, 1);
    assert_eq!(complete.source_artifact().initial_coordinate().0, 2);
    let image = initial(complete.source_artifact());
    let installation = &image["entries"][1]["Installation"];
    assert_eq!(installation["follows_transition"], 1);
    assert_eq!(installation["before"]["lifecycle"], "Archived");
    assert_eq!(
        installation["before"]["lifecycle"],
        installation["after"]["lifecycle"]
    );
    assert_eq!(
        installation["before"]["generation"].as_u64().unwrap() + 1,
        installation["after"]["generation"].as_u64().unwrap()
    );
}
