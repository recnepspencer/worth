//! Journal-body cells are selected independently of readable commit metadata.
use super::cold_envelopes::EnvelopeSurface;
use super::cold_read_matrix::FAILURES;
use super::cold_source::ColdRead;
use super::*;
use crate::execution::RelationalExecutionDenialCause as Execution;
use crate::history::BoundedCanonicalCommitPatchDenial;
use crate::publication::patch::data::{PatchStreamReadErrorClass, PatchStreamRequest};
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailable};

#[derive(Debug, Clone, Copy)]
enum EnvelopeDoor {
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
    fn read(
        self,
        runtime: &crate::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::history::data::RelationalCommitReceipt,
        position: crate::publication::patch::data::PatchStreamPosition,
    ) -> Result<usize, Execution> {
        match self {
            Self::ChangeReceipt => {
                let (_, basis) = runtime.observe_branch(&runtime.main_branch_identity()).unwrap();
                let selected = runtime.select_reachable_commit(&basis.observation(), receipt.commit_id)
                    .into_outcome().expect("selected indexed ancestor belongs to main");
                runtime.mint_change_receipt(execution, selected, None).map(|outcome| match outcome {
                    worth_proof::TransitionOutcome::Success(receipt) => receipt.patch().authoritative_record_patches.len(),
                    other => panic!("a real canonical ancestor must mint its patch: {other:?}"),
                })
            }
            Self::ReplayEnvelope => runtime.replay().canonical_commit_envelope(execution, receipt.commit_id)
                .map(|body| body.expect("selected envelope exists").touched_record_refs().len()),
            Self::ReplayExecution => runtime.replay_authority().replay_commit(
                crate::replay::data::RelationalReplayRequest {
                    commit_id: receipt.commit_id, branch_id: receipt.branch_id.clone(),
                    execution_mode: crate::replay::data::ReplayExecutionMode::SerialDeterministic,
                    verification_mode: crate::replay::data::ReplayVerificationMode::NormalRecoveryVerification,
                }, execution)
                .map(|outcome| {
                    assert!(outcome.failure.is_none(), "real replay scope must reconstruct: {outcome:?}");
                    outcome.reconstructed_commit_closure.len()
                }).map_err(|stop| match stop {
                    RelationalRecoveryStop::Durability(error) => Execution::StorageUnavailable(error.storage_unavailable.expect("cold replay envelope")),
                    other => panic!("funded replay scope: {other:?}"),
                }),
            Self::RetainedExport => runtime.durability()
                .canonical_commit_artifacts(execution.allocation_policy())
                .after_position(crate::publication::patch::data::PatchStreamPosition(position.0 - 1))
                .next().expect("selected index entry exists")
                .map(|artifact| artifact.as_bytes().len())
                .map_err(|denial| match denial {
                    crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial::StorageUnavailable(failure) => Execution::StorageUnavailable(failure),
                    other => panic!("funded real artifact: {other:?}"),
                }),
            Self::EmbeddedCheckpoint => runtime.durability_authority()
                .native_checkpoint(execution, execution.allocation_policy())
                .map(|checkpoint| checkpoint.bytes().len())
                .map_err(|denial| match denial {
                    crate::durability::data::RelationalNativeCheckpointCaptureDenial::Durability(error) =>
                        Execution::StorageUnavailable(error.storage_unavailable.expect("cold journal refusal")),
                    other => panic!("funded real capture: {other:?}"),
                }),
            Self::MemoryCheckpoint => runtime.durability_authority().checkpoint(execution)
                .map(|checkpoint| checkpoint.envelopes.len())
                .map_err(|denial| match denial {
                    RelationalRecoveryStop::Durability(error) =>
                        Execution::StorageUnavailable(error.storage_unavailable.expect("cold journal refusal")),
                    other => panic!("funded real image: {other:?}"),
                }),
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
                    other => panic!("producer patch bound fits: {other:?}"),
                }),
            Self::Stream => runtime
                .publication()
                .read_patch_stream(
                    execution,
                    PatchStreamRequest {
                        after_position: Some(crate::publication::patch::data::PatchStreamPosition(
                            position.0 - 1,
                        )),
                        max_commits: 1,
                    },
                )
                .map(|batch| batch.patches.len())
                .map_err(|denial| match denial.class {
                    PatchStreamReadErrorClass::ReadRequest(cause) => cause,
                    other => panic!("producer stream position is admitted: {other:?}"),
                }),
        }
    }
}

#[cfg(test)]
#[path = "cold_envelope_matrix/exhaustive.rs"]
mod exhaustive;
