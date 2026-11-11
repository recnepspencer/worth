//! Non-head canonical bodies reach content admission only on historical demand.
use super::{recovery::publish, transaction::fixture};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition, storage::RelationalStorageUnavailableCause,
};

mod source;
pub(super) use source::Source;

pub(super) fn reopen_with_tail(
    execution: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    manifest: Vec<u8>,
    source: Arc<Source>,
    tail: Vec<RelationalCanonicalCommitArtifact>,
) -> worth_relational::facade::runtime::RelationalRuntime {
    let expected_replays = tail.len();
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(manifest),
        )),
        tail,
        vec![],
        source.clone(),
    );
    let (mut target, _) = fixture();
    let plan = target
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let recovered = target
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(recovered.coverage.replayed_tail_commits, expected_replays);
    target
}

// A decoded historical body cannot certify field-content absence.
#[test]
fn content_cold_history_is_unavailable_while_a_genuine_dangling_reference_is_missing() {
    use worth_foundational::facade::ContentRefId;
    use worth_relational::facade::{
        content::RelationalContentDenial as Denial,
        mvcc::RelationalTransactionIntent,
        transactions::{ConflictClass, TransactionCommitError},
    };
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&original);
    let (runtime, contract) = fixture();
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    // The older body is an actual Journal tail, not a selected member.
    let (identity, entity) = publish(&runtime, &execution, &contract);
    let older = runtime.history().latest_patch_stream_position().unwrap();
    publish(&runtime, &execution, &contract);
    let tail = runtime
        .durability()
        .canonical_commit_artifacts(&execution)
        .map(Result::unwrap)
        .collect();
    let source = Arc::new(Source::capture(
        &runtime,
        &execution,
        &checkpoint,
        older,
        None,
    ));
    let target = reopen_with_tail(
        &execution,
        &checkpoint,
        checkpoint.bytes().to_vec(),
        source.clone(),
        tail,
    );
    drop(runtime);
    drop(checkpoint);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        0,
        "the historical body starts cold"
    );
    let artifact = target
        .durability()
        .canonical_commit_artifacts(&execution)
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(artifact.position(), older);
    assert_eq!(
        source.reads.load(Ordering::SeqCst),
        1,
        "a genuine cold journal demand ran"
    );
    let decoded =
        RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), &execution).unwrap();
    assert_eq!(
        decoded
            .envelope()
            .content_for_field(
                &execution,
                entity,
                "test.entity",
                &super::historical_custody::locator("left"),
            )
            .unwrap_err(),
        Denial::HistoryNotResident,
    );
    let (_, basis) = target
        .observe_branch(&target.main_branch_identity())
        .unwrap();
    assert_eq!(
        basis
            .read_content(&execution, identity, super::transaction::FORMAT)
            .unwrap()
            .bytes(),
        super::expected::FRAME
    );
    let mut transaction = target
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            super::transaction::batch(&contract, ContentRefId([0xff; 32])),
            execution.allocation_policy(),
        )
        .unwrap();
    let TransactionCommitError::Conflict { error, .. } =
        transaction.commit(&execution, &target).unwrap_err()
    else {
        panic!("a genuine dangling field must refuse before publication")
    };
    assert_eq!(
        error.class,
        ConflictClass::ContentAdmission {
            denial: Denial::MissingContent
        }
    );
    assert_eq!(
        target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1
            .descriptor(),
        basis.descriptor()
    );
}
