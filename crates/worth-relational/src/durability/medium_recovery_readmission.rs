//! Admit the selected medium's checkpoint and contiguous journal tails.

use crate::durability::canonical_commit_artifact::{
    RelationalCanonicalCommitArtifact, RelationalCanonicalCommitArtifactRefusal,
};
use crate::durability::data::{
    DurabilityError, DurableCheckpoint, RecoveryBranchTransitions, RecoveryFailureClass,
    RelationalRecoveryStop,
};
use crate::durability::medium::RelationalDurableColdSource;
use crate::durability::migration::ReadmittedCanonicalCommit;
use crate::durability::published_medium::{
    RelationalDurableCheckpointSource, RelationalDurableRecoverySource,
};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::Arc;

pub(crate) struct ReadmittedMediumRecovery {
    pub(crate) checkpoint: Option<DurableCheckpoint>,
    pub(crate) tail: Vec<ReadmittedCanonicalCommit>,
    pub(crate) branch_transitions: RecoveryBranchTransitions,
    pub(crate) original_source: Option<
        Arc<crate::durability::source_initial_inputs::source_admission::SourceOriginalInputs>,
    >,
    pub(crate) cold_source: Arc<dyn RelationalDurableColdSource>,
    pub(crate) source_memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(crate) fn readmit_medium_recovery(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: RelationalDurableRecoverySource,
) -> Result<ReadmittedMediumRecovery, RelationalRecoveryStop> {
    let parts = source.into_parts();
    if parts
        .checkpoint
        .as_ref()
        .is_some_and(|source| source.admission_image().is_some())
        && parts.readmitted_originals.is_none()
    {
        return Err(RelationalRecoveryStop::AdmissionImage(
            crate::durability::checkpoint_admission::RelationalCheckpointAdmissionRefusal::BindingMismatch(
                crate::durability::checkpoint_admission::CheckpointAdmissionBindingMismatch::Scope,
            ),
        ));
    }
    let complete = parts
        .checkpoint
        .as_ref()
        .and_then(|source| source.complete_artifact())
        .cloned();
    let checkpoint = parts
        .checkpoint
        .map(|source| readmit_covered_checkpoint(execution, source, parts.cold_source.clone()))
        .transpose()?;
    let covered = checkpoint
        .as_ref()
        .map_or(PatchStreamPosition(0), |(_, covered)| *covered);
    let checkpoint = checkpoint.map(|(checkpoint, _)| checkpoint);
    let original_source = match parts.readmitted_originals {
        Some(original) => Some(original),
        None => parts.initial_inputs.map(|initial| {
        crate::durability::source_initial_inputs::source_admission::readmit(initial, complete.as_ref(),
            checkpoint.as_ref(), None, execution).map(std::sync::Arc::new).map_err(|cause| match cause {
                crate::durability::source_initial_inputs::RelationalInitialInputsRefusal::Request(cause) => RelationalRecoveryStop::Request(cause),
                crate::durability::source_initial_inputs::RelationalInitialInputsRefusal::Native(cause) => cause,
                cause => DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint,
                    format!("original source inputs refused: {cause:?}")).into(),
            })
        }).transpose()?,
    };
    let tail = readmit_contiguous_tail(execution, covered, &parts.tail)?
        .into_iter()
        .map(|commit| commit.bind_journal_source(parts.cold_source.clone()))
        .collect::<Result<Vec<_>, _>>()?;
    let bounds = super::medium_transition_tail::TransitionTailBounds {
        covered_transitions: checkpoint.as_ref().map_or(
            crate::branch::transition::RelationalBranchTransitionSequence(0),
            |checkpoint| checkpoint.coverage.up_to_branch_transition,
        ),
        covered_position: covered,
        commit_head: tail.last().map_or(covered, |commit| commit.position()),
    };
    let transitions = super::medium_transition_tail::readmit_transition_tail(
        execution,
        &bounds,
        &parts.transitions,
    )?;
    Ok(ReadmittedMediumRecovery {
        checkpoint,
        tail,
        branch_transitions: RecoveryBranchTransitions::Contiguous(transitions),
        original_source,
        cold_source: parts.cold_source,
        source_memory: parts._source_memory,
    })
}

/// Decode the checkpoint and require its own coverage to be the position and
/// transition sequence its owner filed it under.
fn readmit_covered_checkpoint(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: RelationalDurableCheckpointSource,
    cold_source: std::sync::Arc<dyn crate::durability::medium::RelationalDurableColdSource>,
) -> Result<(DurableCheckpoint, PatchStreamPosition), RelationalRecoveryStop> {
    let filed_under = source.covered();
    let filed_transitions = source.covered_transitions();
    let checkpoint = match source.admission_image().cloned() {
        Some(image) => {
            crate::durability::log::native_checkpoint_codec::decode_checkpoint_with_image(
                execution,
                source.checkpoint().bytes(),
                cold_source,
                image,
            )
        }
        None => crate::durability::log::native_checkpoint_codec::decode_checkpoint_from_source(
            execution,
            source.checkpoint().bytes(),
            Some(cold_source),
        ),
    }?
    .checkpoint;
    let covered = crate::durability::checkpoint_coverage::checkpoint_coverage(&checkpoint);
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
    if checkpoint.coverage.up_to_branch_transition != filed_transitions {
        return Err(DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!(
                "checkpoint covers branch transition {} but was filed under transition {}",
                checkpoint.coverage.up_to_branch_transition.0, filed_transitions.0
            ),
        )
        .into());
    }
    Ok((checkpoint, covered))
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
        let readmitted = artifact
            .readmit_framed(execution)
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
        RelationalCanonicalCommitArtifactRefusal::Content(cause) => cause.into(),
        RelationalCanonicalCommitArtifactRefusal::FieldText(cause) => {
            match crate::symbols::data::field_text_recovery_stop(cause) {
                RelationalRecoveryStop::Durability(mut error)
                    if error.class == RecoveryFailureClass::CorruptCheckpoint =>
                {
                    error.class = RecoveryFailureClass::CorruptCanonicalArtifact;
                    error.into()
                }
                stop => stop,
            }
        }
        refusal => DurabilityError::new(
            RecoveryFailureClass::CorruptCanonicalArtifact,
            format!("canonical artifact at position {position} refused: {refusal:?}"),
        )
        .into(),
    }
}
