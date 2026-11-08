//! Discarded branch bodies do not become selected-capture work.
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::durability::{RecoveryVerificationMode, RelationalCheckpointMember};
use worth_relational::facade::transactions::{
    CreateIntent, EntityMutationIntent, EntitySpec, MutationIntent, UpdateEntityFieldsIntent,
    WorkerIntentBatch,
};
use worth_testkit_relational::{
    aspect_storage::publications::begin_main_transaction, durable_publication::field_patch,
};
use worth_testkit_relational::{
    aspect_storage::publications::release_test_commit_snapshot,
    initial_input_source::{
        capture_external, capture_selected_external, home_image_input, initialized, receiver,
    },
};

#[test]
fn selected_state_recapture_work_does_not_follow_discarded_covered_update_bytes() {
    let small = recapture_after_overwrite(256);
    let large = recapture_after_overwrite(1 << 20);
    assert!(
        large.0.abs_diff(small.0) <= 65536,
        "selected current state excludes discarded input bytes: {small:?}, {large:?}"
    );
    assert!(
        large.3 >= small.3 + (512 << 10),
        "the discarded covered canonical bodies really differ in size: {small:?}, {large:?}"
    );
    // Independent state/output and work observations, deliberately broad
    // headroom. No exact unit quote and no history-dependent fixture allowance.
    assert!(
        large.1 <= small.1 * 2 + 4096,
        "captured state must stay comparable: {small:?}, {large:?}"
    );
    assert!(
        large.2 <= small.2 * 2 + 65536,
        "covered update bytes are not capture work: {small:?}, {large:?}"
    );
}

fn recapture_after_overwrite(bytes: usize) -> (usize, usize, u64, usize) {
    let frame = (1 << 20) + bytes as u64 + 256 + (64 << 10);
    // Bound the original four-frame setup and its transient representations;
    // the later measured state recapture has its own broad finite ceiling.
    let memory = (64 << 20) + 32 * frame;
    let work = (16 << 20) + 64 * frame;
    let lease = worth_execution::ExecutionResourceLease::serial(worth_execution::LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            // Four original frames, the large transient value and their
            // ordinary source/region representations fit this finite budget.
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
    let request = &ExecutionRequest::leased(&lease);
    let (mut runtime, medium) = initialized(request);
    // Both main branches retain the SAME current state and value vocabulary.
    // The varying frame belongs to a real branch deleted before capture.
    let mut transaction = begin_main_transaction(&runtime);
    transaction
        .push_batch(
            WorkerIntentBatch::new("seed").push(MutationIntent::Create(CreateIntent::Entity(
                EntitySpec {
                    partition_id: worth_relational::facade::identity::PartitionId::main(),
                    kind_id: worth_relational::facade::identity::KindId(1),
                    client_key: "stable-key".into(),
                    fields: field_patch("name", "name", &"x".repeat(1 << 20)),
                },
            ))),
            request.allocation_policy(),
        )
        .unwrap();
    let created = transaction.commit(request, &runtime).unwrap();
    let entity = created.semantic_result().created_entities()[0].assigned;
    release_test_commit_snapshot(&runtime, &created);
    drop(created);
    let update = |name: &str| {
        MutationIntent::Entity(EntityMutationIntent::UpdateFields(
            UpdateEntityFieldsIntent {
                entity_id: entity,
                fields: field_patch("name", "name", name),
            },
        ))
    };
    for name in ["x".repeat(256), "final".to_owned()] {
        let mut transaction = begin_main_transaction(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("main-update").push(update(&name)),
                request.allocation_policy(),
            )
            .unwrap();
        let result = transaction.commit(request, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &result);
    }
    let branch = worth_relational::facade::history::BranchId("discarded-input".into());
    let (_, source) = runtime
        .observe_fork_source(runtime.main_branch_identity().branch_id())
        .unwrap();
    runtime
        .fork_branch(request, branch.clone(), source)
        .unwrap();
    let identity = runtime.branch_identity(&branch).unwrap();
    let basis = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(
            &basis,
            worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
        )
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("discarded-update").push(update(&"x".repeat(bytes))),
            request.allocation_policy(),
        )
        .unwrap();
    let result = transaction.commit(request, &runtime).unwrap();
    release_test_commit_snapshot(&runtime, &result);
    drop((result, basis));
    assert!(runtime
        .delete_branch(request, &identity)
        .unwrap()
        .deleted()
        .is_some());
    let first = capture_external(&runtime, &medium, request);
    let original_bytes = first
        .checkpoint()
        .members()
        .iter()
        .filter_map(|member| match member {
            RelationalCheckpointMember::New { bytes, .. }
                if bytes.bytes().starts_with(b"WRTHRCCA") =>
            {
                Some(bytes.len())
            }
            RelationalCheckpointMember::New { .. } | RelationalCheckpointMember::Carried { .. } => {
                None
            }
        })
        .sum::<usize>();
    let selected_state_bytes = first.source_artifact().as_bytes().len()
        + first
            .checkpoint()
            .members()
            .iter()
            .filter_map(|member| match member {
                RelationalCheckpointMember::New { bytes, .. }
                    if !bytes.bytes().starts_with(b"WRTHRCCA") =>
                {
                    Some(bytes.len())
                }
                RelationalCheckpointMember::New { .. }
                | RelationalCheckpointMember::Carried { .. } => None,
            })
            .sum::<usize>();
    let mut members =
        ExecutionRequestArrayBuilder::allocate(*request, first.checkpoint().members().len())
            .unwrap();
    for member in first.checkpoint().members() {
        members.push(member.locator()).unwrap();
    }
    let members = members.seal().unwrap();
    medium.file_checkpoint(first.checkpoint().clone());
    let input = home_image_input(&medium, &first, request).unwrap();
    drop((first, runtime));
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
    let before = medium.canonical_member_reads();
    let (second, report) = request
        .run(
            worth_execution::ExecutionWorkCeiling::new(Some(64 << 20)),
            |_| capture_selected_external(&receiver, &medium, &members, request),
        )
        .unwrap();
    assert_eq!(
        medium.canonical_member_reads(),
        before,
        "covered frames must remain cold"
    );
    assert_eq!(second.source_artifact().covered().0, 4);
    let current_state_bytes = second
        .checkpoint()
        .members()
        .iter()
        .filter_map(|member| match member {
            RelationalCheckpointMember::New { bytes, .. } => Some(bytes.len()),
            RelationalCheckpointMember::Carried { .. } => None,
        })
        .sum();
    (
        selected_state_bytes,
        current_state_bytes,
        report.charged_work(),
        original_bytes,
    )
}

mod cold_materialization;
