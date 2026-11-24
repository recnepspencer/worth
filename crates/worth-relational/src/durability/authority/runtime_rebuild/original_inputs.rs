//! Reconstructed original DATA enters the same stream and commit catalog owners.
use super::original_schemas::OriginalSchemaGraph;
use crate::branch::transition::original_inputs::{
    OriginalEmptySchemaInstallation, OriginalEmptySchemaObservation, RetainedBranchTransition,
};
use crate::branch::transition::sequence::RelationalBranchTransitionSequence;
use crate::durability::checkpoints::region_directory::corrupt;
use crate::durability::data::{RecoveryPlan, RelationalRecoveryStop};
use crate::durability::source_initial_inputs::producer_roster::{
    OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial,
};
use crate::durability::source_initial_inputs::{
    image::*, recorded::RecordedEmptySchemaObservation,
};
use crate::history::data::BranchId;
use crate::runtime::state::subsystems::durability::checkpoint_selection::RelationalCheckpointManifestSelection;
use crate::runtime::state::subsystems::history::HistorySubsystem;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use std::sync::Arc;
mod transition;
use transition::copy_transition;
use worth_execution::ExecutionRequest;

pub(super) trait RecoveredBranchSchemaRoutes {
    fn restore_prefix(
        &self,
        birth: OriginalEmptySchemaObservation,
        installation: Option<OriginalEmptySchemaInstallation>,
        initial: Arc<InitialInputsImage>,
        transitions: Vec<RetainedBranchTransition>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), OriginalBranchSchemaInputsDenial>;

    fn capture(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<OriginalBranchSchemaInputs, OriginalBranchSchemaInputsDenial>;

    fn transition_head(&self) -> RelationalBranchTransitionSequence;

    fn retain_installation(&self, installation: OriginalEmptySchemaInstallation);
}

pub(super) fn restore_prefix(
    history: &HistorySubsystem,
    routes: &impl RecoveredBranchSchemaRoutes,
    main: &BranchId,
    selection: &RelationalCheckpointManifestSelection,
    graph: &OriginalSchemaGraph,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let original = &graph.original;
    if &original.current.main != main {
        return Err(corrupt("original main branch mismatch").into());
    }
    let Some(InitialInputEntry::Birth(birth)) = original.current.entries.first() else {
        return Err(corrupt("original birth missing").into());
    };
    let birth = observation(birth, graph)?;
    crate::durability::checkpoint_admission::SelectedCheckpointCaptureBase::restore_original(
        selection, original, request,
    )?;
    let covered = original.covered.as_ref();
    if let Some(artifact) = covered.and_then(|image| image.artifact.as_ref()) {
        selection.restore_initial_inputs(artifact.clone());
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
    routes
        .restore_prefix(
            birth.clone(),
            installation.clone(),
            original.current.clone(),
            transitions,
            memory,
            request,
        )
        .map_err(capture)?;
    if let Some(cell) = history.branch_cell(main)
        && let Some(mut empty) = OriginalEmptySchemaObservation::from_empty_cell(&cell)
    {
        empty.schema = installation
            .as_ref()
            .map_or_else(|| birth.schema.clone(), |entry| entry.after.schema.clone());
        history
            .restore_source_empty_schema(&empty, false)
            .map_err(corrupt)?;
    }
    Ok(())
}

pub(super) fn install_if_due(
    history: &HistorySubsystem,
    routes: &impl RecoveredBranchSchemaRoutes,
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
    let original = routes.capture(request).map_err(capture)?;
    if original.installation.is_some() {
        return Ok(());
    }
    let install = installation(entry, graph)?;
    let head = routes.transition_head();
    if head < install.follows_transition {
        return Ok(());
    }
    if head != install.follows_transition {
        return Err(corrupt("original installation cut was skipped").into());
    }
    let current = history
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
    history
        .restore_source_empty_schema(&install.after, true)
        .map_err(corrupt)?;
    routes.retain_installation(install);
    Ok(())
}

pub(super) fn restore_catalog(
    history: &HistorySubsystem,
    graph: &OriginalSchemaGraph,
    plan: &RecoveryPlan,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let artifacts = history
        .commit_artifacts_for_request(request)
        .map_err(|cause| {
            crate::durability::checkpoint_history::capture::array_capture_stop(cause)
        })?;
    let mut catalog = crate::history::commit::RelationalCommitCatalog::default();
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
        let inputs = crate::history::commit::OriginalCanonicalInputs {
            prestate_schema: graph.get(prestate)?,
            result_schema: graph.get(result)?,
            mutation,
        };
        let artifact = artifact
            .with_reconstructed_original_inputs(inputs, request)
            .map_err(RelationalRecoveryStop::Request)?;
        catalog
            .append_preencoded_recovery(Arc::new(artifact))
            .map_err(|cause| corrupt(format!("original catalog installation: {cause:?}")))?;
    }
    history.install_commit_catalog(catalog);
    Ok(())
}

pub(super) fn require_replay_prestate(
    history: &HistorySubsystem,
    graph: &OriginalSchemaGraph,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> Result<(), RelationalRecoveryStop> {
    let input = envelope
        .recorded_original_inputs()
        .ok_or_else(|| corrupt("tail original attribution missing"))?;
    let expected = graph.get(input.prestate_schema)?;
    let actual = history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| cell.root())
        .ok_or_else(|| corrupt("tail original predecessor missing"))?;
    if !Arc::ptr_eq(&actual.retained_schema_authority(), &expected) {
        return Err(corrupt("tail original prestate schema disagrees with its branch").into());
    }
    Ok(())
}

/// A schema-changing tail commit replays under the result schema it recorded;
/// an ordinary commit keeps its predecessor root's own schema.
pub(super) fn recorded_result_schema(
    graph: &OriginalSchemaGraph,
    envelope: &crate::history::data::CanonicalCommitEnvelope,
) -> Result<Option<crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema>, RelationalRecoveryStop>{
    let inputs = envelope
        .recorded_original_inputs()
        .ok_or_else(|| corrupt("tail original attribution missing"))?;
    if inputs.result_schema == inputs.prestate_schema {
        return Ok(None);
    }
    crate::durability::authority::runtime_rebuild::recorded_result_schema::RecordedReplayResultSchema::admit(graph.get(inputs.result_schema)?, envelope)
        .map(Some)
        .map_err(Into::into)
}

pub(super) fn bind_replayed_schema(
    lane: &crate::storage::substrate::maintenance::ReconstructionLane<'_>,
    history: &HistorySubsystem,
    symbols: &RuntimeSymbolTable,
    graph: &OriginalSchemaGraph,
    positioned: &crate::history::data::PositionedCanonicalCommit,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), RelationalRecoveryStop> {
    let envelope = positioned.envelope();
    let inputs = envelope
        .recorded_original_inputs()
        .ok_or_else(|| corrupt("replayed source role missing"))?;
    let schema = graph.get(inputs.result_schema)?;
    let root = history
        .branch_cell(&envelope.branch_context)
        .and_then(|cell| cell.root())
        .ok_or_else(|| corrupt("replayed source current root missing"))?;
    if Arc::ptr_eq(&root.retained_schema_authority(), &schema) {
        return Ok(());
    }
    let issuer = crate::branch::root::RelationalBranchRootIdentityIssuer::for_request(request)
        .map_err(root_stop)?;
    let descriptor = root
        .descriptor()
        .cloned()
        .ok_or_else(|| corrupt("replayed source root descriptor missing"))?;
    let positions = root.output_publication_positions.clone();
    // Source synchronization already chose this exact canonical owner. Schema
    // binding must not replace it with an equivalent, separately owned body.
    let original = root
        .canonical_payload()
        .ok_or_else(|| corrupt("replayed source root Original carrier missing"))?;
    let replacement = crate::branch::root::RelationalBranchRoot::readmit_unchanged_for_request(
        lane,
        &issuer,
        &root,
        original,
        descriptor.clone(),
        schema,
        positions,
        request,
    )
    .map_err(root_stop)?;
    crate::runtime::state::subsystems::history::history_catalog_recovery::readmit_replayed_root_descriptor(
            history,
            request,
            envelope.commit.commit_id,
            descriptor,
            replacement,
            &symbols.interner_snapshot(),
        )
        .map_err(RelationalRecoveryStop::from)?;
    Ok(())
}
pub(super) fn root_stop(
    cause: crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial,
) -> RelationalRecoveryStop {
    match cause {
        crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Request(cause)
        | crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Execution(cause) => {
            RelationalRecoveryStop::Request(cause)
        }
        crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial::Content(cause) => cause.into(),
        crate::branch::root_capture_denial::RelationalBranchRootCaptureDenial::StorageUnavailable(cause) => cause.into(),
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
fn capture(
    cause: crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputsDenial,
) -> RelationalRecoveryStop {
    match cause {
        crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputsDenial::Request(cause) => {
            RelationalRecoveryStop::Request(cause.into())
        }
        crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputsDenial::SizeOverflow => {
            RelationalRecoveryStop::Request(
                crate::execution::RelationalExecutionDenialCause::ChargedBytesOverflow,
            )
        }
        crate::durability::source_initial_inputs::producer_roster::OriginalBranchSchemaInputsDenial::OriginalPrefixMismatch => {
            corrupt("reconstructed original stream prefix mismatch").into()
        }
    }
}
