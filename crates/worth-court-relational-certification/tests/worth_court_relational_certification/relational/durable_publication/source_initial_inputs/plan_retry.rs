//! An interrupted actual member read cannot mutate a retained plan's directory.
use super::*;

#[test]
fn cloned_complete_plan_retries_after_member_read_cancellation_without_directory_contamination() {
    let setup=worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&setup);
    let (runtime, medium) = initialized(&request);
    let commit = create_entity_outcome(&request, &runtime, "covered-before-stop");
    release_test_commit_snapshot(&runtime, &commit);
    let image = capture_external(&runtime, &medium, &request);
    medium.file_checkpoint(image.checkpoint().clone());
    let canonical = image
        .checkpoint()
        .members()
        .iter()
        .find_map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes }
                if bytes.bytes().starts_with(b"WRTHRCCA") =>
            {
                Some(*locator)
            }
            RelationalCheckpointMember::New { .. } | RelationalCheckpointMember::Carried { .. } => {
                None
            }
        })
        .expect("actual canonical frame member");
    let source: Arc<dyn RelationalDurableColdSource> = medium.clone();
    let original = source
        .checkpoint_selected_read_protection(&request)
        .unwrap();
    let before_probe = medium.canonical_member_reads();
    let prefix = source
        .read_member_prefix_owned(&request, original.as_ref(), canonical, 10)
        .unwrap();
    RelationalCanonicalCommitArtifact::preflight_frame_version(prefix.bytes()).unwrap();
    assert_eq!(
        medium.canonical_member_reads(),
        before_probe,
        "a version prefix probe is not a full body"
    );
    let later = create_entity_outcome(&request, &runtime, "tail-after-checkpoint");
    release_test_commit_snapshot(&runtime, &later);
    let input = home_input(&medium, image.source_artifact().clone(), &request).unwrap();
    let mut receiver = receiver(&medium);
    let before = receiver.runtime_instance_id();
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            &request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let retry = plan.clone();
    let cancellation = worth_execution::CancellationSource::new();
    let stopping = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    medium.cancel_next_member_read_for_test(cancellation.clone());
    let stopped = receiver
        .durability_recovery()
        .recover_with_authority(&ExecutionRequest::serial(&stopping), plan)
        .unwrap_err();
    assert!(
        cancellation.is_cancelled(),
        "the actual member ingress was reached"
    );
    assert!(matches!(
        stopped,
        RelationalRecoveryStop::Request(
            worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
        )
    ));
    assert_eq!(receiver.runtime_instance_id(), before);
    receiver
        .durability_recovery()
        .recover_with_authority(&request, retry)
        .unwrap();
    let artifacts: Vec<_> = receiver
        .durability()
        .canonical_commit_artifacts(&request)
        .map(|artifact| artifact.unwrap())
        .collect();
    assert_eq!(artifacts.len(), 2);
    assert!(
        medium.canonical_member_reads() > 0,
        "selected canonical bodies are observed at their actual read seam"
    );
    assert_eq!(
        capture(&receiver, &request).source_artifact().covered().0,
        2
    );
}
