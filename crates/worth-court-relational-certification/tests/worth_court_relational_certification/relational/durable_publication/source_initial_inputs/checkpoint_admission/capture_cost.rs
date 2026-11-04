//! Discarded update bodies cannot become work in a selected-state recapture.
use super::*;

#[test]
fn selected_checkpoint_without_captured_original_base_refuses_without_body_demand() {
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let (runtime, medium) = initialized(request);
    let created = create_entity_outcome(request, &runtime, "selected-without-image");
    release_test_commit_snapshot(&runtime, &created);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            request,
            request.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(medium.clone()),
            },
        )
        .unwrap();
    medium.file_checkpoint(checkpoint.clone());
    request
        .run_native_with_checkpoint(|context| {
            checkpoint.acknowledge_selected(request, context).unwrap()
        })
        .unwrap();
    let reads = (
        medium.canonical_member_reads(),
        medium.canonical_journal_reads(),
    );
    assert!(matches!(
        runtime.complete_checkpoint_service().prepare(
            request,
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: Some(medium.clone()),
            }
        ),
        Err(RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable)
    ));
    assert_eq!(
        (
            medium.canonical_member_reads(),
            medium.canonical_journal_reads()
        ),
        reads
    );
}

#[test]
fn selected_embedded_capture_refuses_before_canonical_body_demand() {
    let caller = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&caller);
    let (runtime, medium) = initialized(request);
    let created = create_entity_outcome(request, &runtime, "covered-before-embedded-demand");
    release_test_commit_snapshot(&runtime, &created);
    drop(created);
    let image = capture_external(&runtime, &medium, request);
    medium.file_checkpoint(image.checkpoint().clone());
    let tail = create_entity_outcome(request, &runtime, "uncovered-before-embedded-demand");
    release_test_commit_snapshot(&runtime, &tail);
    drop(tail);
    let input = home_image_input(&medium, &image, request).unwrap();
    drop((image, runtime));
    let mut receiver = receiver(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiver
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    let before = (
        medium.canonical_member_reads(),
        medium.canonical_journal_reads(),
    );
    assert!(matches!(
        receiver
            .complete_checkpoint_service()
            .prepare(request, RelationalCheckpointArtifactPolicy::Embedded),
        Err(RelationalNativeCheckpointCaptureDenial::OriginalAuthorityUnavailable)
    ));
    assert_eq!(
        (
            medium.canonical_member_reads(),
            medium.canonical_journal_reads()
        ),
        before,
        "selected Embedded refusal precedes demands for covered and uncovered frames",
    );
}

#[test]
fn checkpoint_capture_restores_many_branches_with_long_common_names() {
    use worth_relational::facade::history::BranchId;
    const BRANCHES: usize = 8;
    const PREFIX_BYTES: usize = 4096;
    // The actual state includes repeated names in cells, transitions and
    // component descriptors. Fund those representations and byte comparisons,
    // independently of any retained canonical-body capacity.
    let names_bytes = (BRANCHES * (PREFIX_BYTES + 4)) as u64;
    let memory = (64 << 20) + names_bytes * 64;
    let work = (32 << 20) + names_bytes * 256;
    let caller = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(memory),
                Some(work),
            ),
        ),
        cancellation: worth_execution::CancellationToken::new(),
        deadline: None,
    })
    .unwrap();
    let request = &ExecutionRequest::leased(&caller);
    let (runtime, medium) = initialized(request);
    let created = create_entity_outcome(request, &runtime, "long-branch-names");
    let receipt = created.commit.clone();
    release_test_commit_snapshot(&runtime, &created);
    drop(created);
    let prefix = "p".repeat(PREFIX_BYTES);
    let names = (0..BRANCHES)
        .map(|index| BranchId(format!("{prefix}-{index:03}")))
        .collect::<Vec<_>>();
    for name in &names {
        let (_, basis) = runtime
            .observe_fork_source(runtime.main_branch_identity().branch_id())
            .unwrap();
        runtime.fork_branch(request, name.clone(), basis).unwrap();
    }
    let (complete, _) = request
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(work)),
            |_| capture_external(&runtime, &medium, request),
        )
        .unwrap();
    assert_eq!(complete.source_artifact().covered().0, 1);
    assert_eq!(
        complete.source_artifact().covered_transitions().0,
        BRANCHES as u64
    );
    medium.file_checkpoint(complete.checkpoint().clone());
    let input = home_image_input(&medium, &complete, request).unwrap();
    drop((complete, runtime));
    let reads = medium.canonical_member_reads();
    let mut receiver = receiver(&medium);
    let plan = receiver
        .durability()
        .recovery_plan_from_source(
            request,
            input,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiver
        .durability_recovery()
        .recover_with_authority(request, plan)
        .unwrap();
    for name in &names {
        let (actual, _) = receiver.observe_fork_source(name).unwrap();
        assert_eq!(actual.source_branch(), name);
        assert_eq!(
            actual
                .observation()
                .target()
                .as_basis()
                .unwrap()
                .selected_commit_id(),
            receipt.commit_id.0,
        );
    }
    assert_eq!(medium.canonical_member_reads(), reads);
}
