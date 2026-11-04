//! Reconstructed original DATA enters the same stream and commit catalog owners.
use super::original_schemas::OriginalSchemaGraph;
use crate::durability::checkpoints::region_directory::corrupt;
use crate::durability::data::{RecoveryPlan, RelationalRecoveryStop};
use crate::durability::source_initial_inputs::{
    image::*, recorded::RecordedEmptySchemaObservation,
};
use crate::runtime::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RelationalRuntime,
    RetainedBranchTransition,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

pub(super) fn restore_prefix(
    runtime: &RelationalRuntime,
    graph: &OriginalSchemaGraph,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let original = &graph.original;
    if original.current.main != runtime.config.history.main_branch {
        return Err(corrupt("original main branch mismatch").into());
    }
    let Some(InitialInputEntry::Birth(birth)) = original.current.entries.first() else {
        return Err(corrupt("original birth missing").into());
    };
    let birth = observation(birth, graph)?;
    crate::durability::checkpoint_admission::SelectedCheckpointCaptureBase::restore_original(
        runtime, original, request,
    )?;
    let covered = original.covered.as_ref();
    if let Some(artifact) = covered.and_then(|image| image.artifact.as_ref()) {
        runtime
            .durability
            .checkpoint_selection()
            .restore_initial_inputs(artifact.clone());
    }
    let installation = covered
        .and_then(|image| image.entries.get(1))
        .map(|entry| installation(entry, graph))
        .transpose()?;
    let count = original
        .normalized
        .as_ref()
        .map_or(0, |image| image.transitions.len());
    let memory = request
        .reserve_retained_memory(
            std::alloc::Layout::array::<RetainedBranchTransition>(count)
                .map_err(|_| corrupt("source transition roster overflow"))?
                .size() as u64,
        )
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?;
    let mut transitions = Vec::with_capacity(count);
    if let Some(normalized) = &original.normalized {
        for entry in &normalized.transitions {
            let retained = copy_transition(entry, graph, request)?;
            transitions.push(retained);
        }
    }
    runtime
        .history
        .canonical_checkpoint_gate()
        .restore_source_schema_inputs(
            birth.clone(),
            installation.clone(),
            original.current.clone(),
            transitions,
            memory,
            request,
        )
        .map_err(capture)?;
    if let Some(cell) = runtime
        .history
        .branch_cell(&runtime.config.history.main_branch)
        && let Some(mut empty) = OriginalEmptySchemaObservation::from_empty_cell(&cell)
    {
        empty.schema = installation
            .as_ref()
            .map_or_else(|| birth.schema.clone(), |entry| entry.after.schema.clone());
        runtime
            .history
            .restore_source_empty_schema(&empty, false)
            .map_err(corrupt)?;
    }
    Ok(())
}

pub(super) fn install_if_due(
    runtime: &RelationalRuntime,
    graph: &OriginalSchemaGraph,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let source = &graph.original;
    if source
        .covered
        .as_ref()
        .is_some_and(|image| image.entries.len() == 2)
    {
        return Ok(());
    }
    let Some(entry) = source.current.entries.get(1) else {
        return Ok(());
    };
    let original = runtime
        .history
        .capture_original_branch_schema_inputs(request)
        .map_err(capture)?;
    if original.installation.is_some() {
        return Ok(());
    }
    let install = installation(entry, graph)?;
    let head = runtime
        .history
        .canonical_checkpoint_gate()
        .branch_transition_head();
    if head < install.follows_transition {
        return Ok(());
    }
    if head != install.follows_transition {
        return Err(corrupt("original installation cut was skipped").into());
    }
    let current = runtime
        .history
        .branch_cell(&source.current.main)
        .and_then(|cell| OriginalEmptySchemaObservation::from_empty_cell(&cell))
        .ok_or_else(|| corrupt("original installation requires its Empty predecessor"))?;
    if current.generation != install.before.generation
        || current.truth_version != install.before.truth_version
        || current.lifecycle != install.before.lifecycle
        || !Arc::ptr_eq(&current.schema, &install.before.schema)
    {
        return Err(corrupt("original installation predecessor disagrees").into());
    }
    runtime
        .history
        .restore_source_empty_schema(&install.after, true)
        .map_err(corrupt)?;
    runtime
        .history
        .canonical_checkpoint_gate()
        .retain_original_empty_installation(install.before, install.after);
    Ok(())
}

pub(super) fn restore_catalog(
    runtime: &RelationalRuntime,
    graph: &OriginalSchemaGraph,
    plan: &RecoveryPlan,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let artifacts = runtime
        .history
        .commit_artifacts_for_request(request)
        .map_err(|cause| {
            crate::durability::checkpoint_history::capture::array_capture_stop(cause)
        })?;
    let mut catalog = crate::history::RelationalCommitCatalog::default();
    for artifact in artifacts.iter() {
        crate::execution::check_request(request).map_err(RelationalRecoveryStop::Request)?;
        let position = artifact
            .envelope_residency()
            .captured_position()
            .ok_or_else(|| corrupt("recovered original catalog lacks position"))?;
        let normalized = graph.original.normalized.as_ref().and_then(|image| {
            usize::try_from(position.0.checked_sub(1)?)
                .ok()
                .and_then(|index| image.canonical.get(index))
        });
        let (prestate, result, mutation) = match normalized {
            Some(record) => {
                if record.position != position
                    || record.commit != artifact.commit_id()
                    || artifact
                        .envelope_residency()
                        .journal_index()
                        .is_none_or(|index| index.payload_digest != record.payload_digest)
                {
                    return Err(corrupt("normalized original catalog subject mismatch").into());
                }
                (
                    record.prestate_schema,
                    record.result_schema,
                    record.mutation.clone(),
                )
            }
            None => {
                let commit = plan
                    .tail_log
                    .iter()
                    .find(|commit| commit.position() == position)
                    .ok_or_else(|| corrupt("original catalog position outside protected source"))?;
                let recorded = commit
                    .envelope()
                    .recorded_original_inputs()
                    .ok_or_else(|| corrupt("tail original attribution missing"))?;
                (
                    recorded.prestate_schema,
                    recorded.result_schema,
                    recorded.mutation.clone(),
                )
            }
        };
        let inputs = crate::history::OriginalCanonicalInputs {
            prestate_schema: graph.get(prestate)?,
            result_schema: graph.get(result)?,
            mutation,
        };
        let artifact = artifact
            .with_reconstructed_original_inputs(inputs, request)
            .map_err(RelationalRecoveryStop::Request)?;
        catalog
            .append_preencoded_recovery(artifact)
            .map_err(|cause| corrupt(format!("original catalog installation: {cause:?}")))?;
    }
    runtime.history.install_commit_catalog(catalog);
    runtime
        .history
        .canonical_checkpoint_gate()
        .install_checkpoint_catalog(runtime.history.commit_artifacts());
    Ok(())
}

pub(super) fn require_replay_prestate(
    runtime: &RelationalRuntime,
    graph: &OriginalSchemaGraph,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> Result<(), RelationalRecoveryStop> {
    let input = envelope
        .recorded_original_inputs()
        .ok_or_else(|| corrupt("tail original attribution missing"))?;
    let expected = graph.get(input.prestate_schema)?;
    let actual = runtime
        .history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| cell.root())
        .ok_or_else(|| corrupt("tail original predecessor missing"))?;
    if !Arc::ptr_eq(&actual.retained_schema_authority(), &expected) {
        return Err(corrupt("tail original prestate schema disagrees with its branch").into());
    }
    Ok(())
}

pub(super) fn bind_replayed_schema(
    runtime: &mut RelationalRuntime,
    graph: &OriginalSchemaGraph,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let envelope = positioned.envelope();
    let inputs = envelope
        .recorded_original_inputs()
        .ok_or_else(|| corrupt("replayed source role missing"))?;
    let schema = graph.get(inputs.result_schema)?;
    let root = runtime
        .history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| cell.root())
        .ok_or_else(|| corrupt("replayed source current root missing"))?;
    if Arc::ptr_eq(&root.retained_schema_authority(), &schema) {
        return Ok(());
    }
    let issuer = crate::branch::RelationalBranchRootIdentityIssuer::for_request(request)
        .map_err(root_stop)?;
    let descriptor = root
        .descriptor()
        .cloned()
        .ok_or_else(|| corrupt("replayed source root descriptor missing"))?;
    let positions = root.output_publication_positions.clone();
    // Source synchronization already chose this exact canonical owner. Schema
    // binding must not replace it with an equivalent, separately owned body.
    let canonical = root
        .canonical_envelope()
        .cloned()
        .ok_or_else(|| corrupt("replayed source root canonical owner missing"))?;
    let replacement = crate::branch::RelationalBranchRoot::readmit_unchanged_for_request(
        &issuer,
        &root,
        canonical,
        descriptor.clone(),
        schema,
        positions,
        request,
    )
    .map_err(root_stop)?;
    runtime
        .history
        .readmit_replayed_root_descriptor(
            envelope.commit.commit_id,
            descriptor,
            replacement,
            &runtime.services.symbols.interner_snapshot(),
        )
        .map_err(corrupt)?;
    Ok(())
}
pub(super) fn root_stop(
    cause: crate::branch::RelationalBranchRootCaptureDenial,
) -> RelationalRecoveryStop {
    match cause {
        crate::branch::RelationalBranchRootCaptureDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        crate::branch::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => cause.into(),
        cause => corrupt(format!("root readmission: {cause:?}")).into(),
    }
}

fn observation(
    record: &RecordedEmptySchemaObservation,
    graph: &OriginalSchemaGraph,
) -> Result<OriginalEmptySchemaObservation, RelationalRecoveryStop> {
    Ok(OriginalEmptySchemaObservation {
        generation: record.generation,
        truth_version: record.truth_version,
        lifecycle: record.lifecycle,
        schema: graph.get(record.schema_carrier)?,
    })
}
fn installation(
    entry: &InitialInputEntry,
    graph: &OriginalSchemaGraph,
) -> Result<OriginalEmptySchemaInstallation, RelationalRecoveryStop> {
    match entry {
        InitialInputEntry::Installation {
            before,
            after,
            follows_transition,
        } => Ok(OriginalEmptySchemaInstallation {
            before: observation(before, graph)?,
            after: observation(after, graph)?,
            follows_transition: *follows_transition,
        }),
        InitialInputEntry::Birth(_) => {
            Err(corrupt("birth cannot stand in for installation").into())
        }
    }
}
fn copy_transition(
    entry: &crate::durability::source_initial_inputs::normalized::TransitionOriginalRecord,
    graph: &OriginalSchemaGraph,
    request: &ExecutionRequest<'_, '_>,
) -> Result<RetainedBranchTransition, RelationalRecoveryStop> {
    use crate::branch::{PositionedBranchTransition, RelationalBranchTransition};
    let dynamic = match &entry.transition {
        RelationalBranchTransition::Fork {
            target,
            source,
            source_head,
            ..
        } => target.0.len().checked_add(source.0.len()).and_then(|n| {
            source_head
                .parent_commit_ids()
                .len()
                .checked_mul(8)
                .and_then(|parents| n.checked_add(parents))
        }),
        RelationalBranchTransition::Archive { branch, .. }
        | RelationalBranchTransition::DeletionRequested { branch, .. }
        | RelationalBranchTransition::Delete { branch, .. } => Some(branch.0.len()),
    }
    .ok_or_else(|| corrupt("transition source backing overflow"))?;
    let bytes = arc_size::<PositionedBranchTransition>()?
        .checked_add(arc_size::<worth_execution::ExecutionMemoryReservation>()?)
        .and_then(|n| n.checked_add(dynamic as u64))
        .ok_or_else(|| corrupt("transition source control overflow"))?;
    let memory = request
        .reserve_retained_memory(bytes)
        .map_err(|cause| RelationalRecoveryStop::Request(cause.into()))?
        .map(Arc::new);
    let positioned =
        PositionedBranchTransition::admit(entry.sequence, entry.follows, entry.transition.clone())
            .with_schema_carrier(Some(entry.schema))
            .with_source_memory(memory);
    Ok(RetainedBranchTransition {
        positioned: Arc::new(positioned),
        schema: graph.get(entry.schema)?,
    })
}
fn arc_size<T>() -> Result<u64, RelationalRecoveryStop> {
    std::alloc::Layout::new::<[usize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .map(|(layout, _)| layout.pad_to_align().size() as u64)
        .map_err(|_| corrupt("transition source layout overflow").into())
}
fn capture(cause: crate::runtime::OriginalBranchSchemaInputsDenial) -> RelationalRecoveryStop {
    match cause {
        crate::runtime::OriginalBranchSchemaInputsDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        crate::runtime::OriginalBranchSchemaInputsDenial::SizeOverflow => {
            RelationalRecoveryStop::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )
        }
        crate::runtime::OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
            corrupt("reconstructed original stream prefix mismatch").into()
        }
    }
}
