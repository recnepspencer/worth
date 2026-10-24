//! The recovery plan for a home whose history lives in a durable medium.
//!
//! The medium's owner selects the checkpoint and the journal tail; this file
//! readmits both before either becomes authority. Every disagreement refuses:
//! a damaged artifact, a gap, or a checkpoint filed under the wrong position
//! never shortens the recovered history.

use crate::capabilities::RuntimeConfigSource;
use crate::durability::access::{
    authority_continuity_for_envelopes, descriptor_semantics_version_for_envelopes,
};
use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal,
};
use crate::durability::data::{
    DurabilityError, DurableCheckpoint, RecoveryCursor, RecoveryFailureClass,
    RecoveryIntegrityReport, RecoveryPlan, RecoveryVerificationMode, RelationalRecoveryStop,
};
use crate::durability::medium::{
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use crate::durability::migration::ReadmittedCanonicalCommit;
use crate::publication::patch::data::PatchStreamPosition;
use crate::runtime::RelationalRuntime;

pub(super) fn medium_recovery_plan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    source: RelationalDurableRecoverySource,
    verification_mode: RecoveryVerificationMode,
) -> Result<RecoveryPlan, RelationalRecoveryStop> {
    let (checkpoint_source, tail_artifacts) = source.into_parts();
    let checkpoint = checkpoint_source
        .map(|source| readmit_covered_checkpoint(execution, source))
        .transpose()?;
    let covered = checkpoint
        .as_ref()
        .map_or(PatchStreamPosition(0), |(_, covered)| *covered);
    let checkpoint = checkpoint.map(|(checkpoint, _)| checkpoint);
    let tail = readmit_contiguous_tail(execution, covered, &tail_artifacts)?;
    let checkpoint_envelopes = checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.envelopes.as_slice())
        .unwrap_or(&[]);
    let descriptor_semantics_version =
        descriptor_semantics_version_for_envelopes(checkpoint_envelopes, &tail);
    let mut continuity = authority_continuity_for_envelopes(runtime, checkpoint_envelopes, &tail);
    if let Some(checkpoint) = &checkpoint {
        super::native_checkpoint_recovery_plan::bind_source_runtime_identity(
            runtime,
            checkpoint,
            &mut continuity,
        );
    }
    let restore_authoritative_envelope_commit_ids = tail
        .iter()
        .map(|entry| entry.envelope().commit.commit_id)
        .collect();
    Ok(RecoveryPlan::new(
        runtime.runtime_config().clone(),
        None,
        None,
        checkpoint,
        tail,
        RecoveryCursor {
            checkpoint_id: None,
            segment_ids: Vec::new(),
        },
        RecoveryIntegrityReport {
            selected_checkpoint_id: None,
            skipped_corrupt_checkpoints: Vec::new(),
            verified_segment_ids: Vec::new(),
            corrupt_segment_id: None,
        },
        continuity,
        verification_mode,
        descriptor_semantics_version,
        restore_authoritative_envelope_commit_ids,
    )
    .with_commit_strategy_executors(runtime.commit_strategy_executor_registry().clone()))
}

/// Decode the checkpoint and require its own coverage to be the position its
/// owner filed it under.
fn readmit_covered_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: RelationalDurableCheckpointSource,
) -> Result<(DurableCheckpoint, PatchStreamPosition), RelationalRecoveryStop> {
    let filed_under = source.covered();
    let checkpoint = crate::durability::log::native_file_codec::decode_checkpoint(
        execution,
        source.into_checkpoint().bytes(),
    )?
    .checkpoint;
    let covered = checkpoint_coverage(&checkpoint);
    if covered != filed_under {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "checkpoint covers position {} but was filed under position {}",
                covered.0, filed_under.0
            ),
        )
        .into());
    }
    Ok((checkpoint, covered))
}

/// The last position a checkpoint's retained envelopes include, or zero for a
/// checkpoint of an empty history.
pub(crate) fn checkpoint_coverage(checkpoint: &DurableCheckpoint) -> PatchStreamPosition {
    checkpoint
        .envelopes
        .iter()
        .map(crate::history::data::PositionedCanonicalCommit::position)
        .max()
        .unwrap_or(PatchStreamPosition(0))
}

/// Readmit every tail artifact, requiring positions `covered + 1`, `+ 2`, ...
fn readmit_contiguous_tail(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    covered: PatchStreamPosition,
    artifacts: &[RelationalCanonicalCommitArtifact],
) -> Result<Vec<ReadmittedCanonicalCommit>, RelationalRecoveryStop> {
    let mut tail = Vec::with_capacity(artifacts.len());
    let mut expected = covered.0;
    for artifact in artifacts {
        expected = expected.checked_add(1).ok_or_else(|| {
            DurabilityError::new(
                RecoveryFailureClass::NonContiguousCanonicalTail,
                "canonical tail position overflows",
            )
        })?;
        if artifact.position().0 != expected {
            return Err(DurabilityError::new(
                RecoveryFailureClass::NonContiguousCanonicalTail,
                format!(
                    "canonical tail holds position {} where position {expected} is required",
                    artifact.position().0
                ),
            )
            .into());
        }
        let readmitted = RelationalCanonicalCommitArtifact::readmit(artifact.as_bytes(), execution)
            .map_err(|refusal| artifact_refusal_stop(expected, refusal))?;
        tail.push(readmitted);
    }
    Ok(tail)
}

fn artifact_refusal_stop(
    position: u64,
    refusal: RelationalCanonicalCommitArtifactRefusal,
) -> RelationalRecoveryStop {
    match refusal {
        RelationalCanonicalCommitArtifactRefusal::Request(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        refusal => DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("canonical artifact at position {position} refused: {refusal:?}"),
        )
        .into(),
    }
}
