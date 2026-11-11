//! Journal-body cells are selected independently of readable commit metadata.
use super::envelope_surfaces::EnvelopeSurface;

use worth_relational::facade::history::BoundedCanonicalCommitPatchDenial;
use worth_relational::facade::publication::{PatchStreamReadErrorClass, PatchStreamRequest};
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Execution;

#[derive(Debug, Clone, Copy)]
pub(super) enum EnvelopeDoor {
    Surface(EnvelopeSurface),
    Patches,
    Stream,
    RetainedExport,
    EmbeddedCheckpoint,
    MemoryCheckpoint,
    ReplayEnvelope,
    ReplayExecution,
    ChangeReceipt,
}
impl EnvelopeDoor {
    pub(super) fn read(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &worth_relational::facade::history::RelationalCommitReceipt,
        position: worth_relational::facade::publication::PatchStreamPosition,
    ) -> Result<usize, Execution> {
        match self {
            Self::ChangeReceipt => {
                let (_, basis) = runtime.observe_branch(&runtime.main_branch_identity()).unwrap();
                let selected = runtime.select_reachable_commit(&basis.observation(), receipt.commit_id)
                    .into_outcome().expect("selected indexed ancestor belongs to main");
                runtime.mint_change_receipt(execution, selected, None).map(|outcome| match outcome {
                    worth_proof::TransitionOutcome::Success(receipt) => receipt.patch().authoritative_record_patches.len(),
                    worth_proof::TransitionOutcome::Denied(cause)=>panic!("original ancestor denied: {cause:?}"),
 worth_proof::TransitionOutcome::Deferred(cause)=>panic!("original ancestor deferred: {cause:?}"),
 worth_proof::TransitionOutcome::Stale(cause)=>panic!("original ancestor stale: {cause:?}"),
 worth_proof::TransitionOutcome::RebindRequired(cause)=>panic!("original ancestor rebind: {cause:?}"),
 worth_proof::TransitionOutcome::Failed(cause)=>panic!("original ancestor failed: {cause:?}"),
                })
            }
            Self::ReplayEnvelope => runtime.replay().canonical_commit_envelope(execution, receipt.commit_id)
                .map(|body| {
                    let body=body.expect("selected envelope exists");
                    assert_eq!(body.commit.commit_id,receipt.commit_id);
                    // Every original fixture commit creates or changes the real name field.
                    // Its public authoritative patches therefore observe genuine body content.
                    body.patch.authoritative_record_patches.len()
                }),
            Self::ReplayExecution => runtime.replay_authority().replay_commit(
                worth_relational::facade::replay::RelationalReplayRequest {
                    commit_id: receipt.commit_id, branch_id: receipt.branch_id.clone(),
                    execution_mode: worth_relational::facade::replay::ReplayExecutionMode::SerialDeterministic,
                    verification_mode: worth_relational::facade::replay::ReplayVerificationMode::NormalRecoveryVerification,
                }, execution)
                .map(|outcome| {
                    assert!(outcome.failure.is_none(), "real replay scope must reconstruct: {outcome:?}");
                    outcome.reconstructed_commit_closure.len()
                }).map_err(super::checkpoint_read_refusal::memory_stop),
            Self::RetainedExport => runtime.durability()
                .canonical_commit_artifacts(execution)
                .after_position(worth_relational::facade::publication::PatchStreamPosition(position.0 - 1))
                .next().expect("selected index entry exists")
                .map(|artifact| artifact.as_bytes().len())
                .map_err(|denial| match denial {
                    worth_relational::facade::durability::RelationalCanonicalCommitArtifactEncodeDenial::StorageUnavailable(failure) => Execution::StorageUnavailable(failure),
                    other => panic!("funded real artifact: {other:?}"),
                }),
            Self::EmbeddedCheckpoint => runtime.durability_authority()
                .native_checkpoint(execution, execution.allocation_policy())
                .map(|checkpoint| checkpoint.bytes().len())
                .map_err(super::checkpoint_read_refusal::native_stop),
            Self::MemoryCheckpoint => {
                let image=runtime.durability_authority().checkpoint(execution).map_err(super::checkpoint_read_refusal::memory_stop)?;
                let plan=runtime.durability().recovery_plan(execution,worth_relational::facade::durability::RecoveryVerificationMode::NormalRecoveryVerification).map_err(super::checkpoint_read_refusal::memory_stop)?;
                assert_eq!(plan.checkpoint.as_ref(),Some(&image),"the retained plan consumes the actual captured image");
                let medium=worth_testkit_relational::durable_medium::TestDurableMedium::new();
                let mut receiver=worth_testkit_relational::durable_publication::durable_runtime(&medium);
                let recovered=receiver.durability_recovery().recover(execution,plan).map_err(super::checkpoint_read_refusal::memory_stop)?;
                Ok(recovered.coverage.checkpoint_commits)
            }
            Self::Surface(surface) => surface.read(execution, runtime, receipt),
            Self::Patches => runtime
                .history()
                .bounded_canonical_commit_patches(
                    execution,
                    receipt.commit_id,
                    std::num::NonZeroUsize::new(32).unwrap(),
                )
                .map(|patches| patches.len())
                .map_err(|denial| match denial {
                    BoundedCanonicalCommitPatchDenial::ReadRequest(cause) => cause,
                    BoundedCanonicalCommitPatchDenial::CommitUnavailable=>panic!("original commit exists"),
 BoundedCanonicalCommitPatchDenial::WorkExhausted{required_records}=>panic!("original patch bound fits {required_records}"),
                }),
            Self::Stream => runtime
                .publication()
                .read_patch_stream(
                    execution,
                    PatchStreamRequest {
                        after_position: Some(worth_relational::facade::publication::PatchStreamPosition(
                            position.0 - 1,
                        )),
                        max_commits: 1,
                    },
                )
                .map(|batch| batch.patches.len())
                .map_err(|denial| match denial.class {
                    PatchStreamReadErrorClass::ReadRequest(cause) => cause,
                    PatchStreamReadErrorClass::UnknownResumePosition=>panic!("original resume position exists"),
 PatchStreamReadErrorClass::InvalidBatchSize=>panic!("one original commit is a valid batch"),
 PatchStreamReadErrorClass::RetainedHistoryGap=>panic!("original source has no gap"),
                }),
        }
    }
}
