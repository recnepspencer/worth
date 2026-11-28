use worth_relational_contracts::internal_port::publication::patch::data::authoritative_patch_envelope::PublishedAuthoritativePatchEnvelopeOwnerPort as _;
use crate::publication::patch::data::{
    PatchStreamBatch, PatchStreamReadError, PatchStreamReadErrorClass, PatchStreamRequest,
    PublishedAuthoritativePatchEnvelope,
};
use crate::runtime::RelationalRuntime;

pub(crate) fn read_patch_stream(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    request: PatchStreamRequest,
) -> Result<PatchStreamBatch, PatchStreamReadError> {
    crate::execution::request_custody::check_request(execution).map_err(read_refused)?;
    if request.max_commits == 0 {
        return Err(PatchStreamReadError {
            class: PatchStreamReadErrorClass::InvalidBatchSize,
            detail: "patch stream request must ask for at least one commit".to_string(),
        });
    }
    let latest_position = runtime.history().latest_patch_stream_position();
    let latest_commit_id = runtime.history().latest_canonical_commit_id();
    if let Some(after_position) = request.after_position {
        let recorded = runtime
            .history
            .try_recorded_commit_at_patch_position(after_position)
            .map_err(|cause| storage_refused(after_position, cause))?;
        let exists = recorded.is_some()
            || runtime
                .history
                .canonical_envelope_at_patch(after_position)
                .is_some();
        if !exists {
            return Err(PatchStreamReadError {
                class: PatchStreamReadErrorClass::UnknownResumePosition,
                detail: format!("unknown patch stream resume position {}", after_position.0),
            });
        }
    }
    let references = crate::publication::canonical_history::try_patch_stream_commits_after(
        &runtime.history,
        request.after_position,
        request.max_commits,
    )
    .map_err(|cause| {
        storage_refused(
            request
                .after_position
                .unwrap_or(crate::publication::patch::data::PatchStreamPosition(0)),
            cause,
        )
    })?;
    let mut patches = Vec::with_capacity(references.len());
    for reference in references {
        let envelope = match runtime
            .history
            .try_commit_artifact(reference.commit_id)
            .map_err(|cause| storage_refused(reference.position, cause))?
        {
            Some(artifact) => artifact.demand_envelope(execution, 1 << 30)
                .map_err(|failure| read_refused(failure.into()))?.clone(),
            None => crate::publication::retained_canonical_envelope_at_position(&runtime.history, runtime.durability.canonical_coverage(), reference.position, None)
                .map(|commit| commit.canonical_arc().clone())
                .ok_or_else(|| PatchStreamReadError {
                    class: PatchStreamReadErrorClass::RetainedHistoryGap,
                    detail: format!("patch stream position {} is retained for commit {} but has no canonical envelope or durable recovery coverage", reference.position.0, reference.commit_id.0),
                })?,
        };
        patches.push(PublishedAuthoritativePatchEnvelope::from_canonical(
            reference.position,
            &envelope.patch,
        ));
    }
    crate::execution::request_custody::check_request(execution).map_err(read_refused)?;
    Ok(PatchStreamBatch {
        resumed_after: request.after_position,
        next_position: patches.last().map(|patch| patch.position),
        latest_position,
        latest_commit_id,
        patches,
    })
}

fn storage_refused(
    position: crate::publication::patch::data::PatchStreamPosition,
    cause: crate::storage::data::RelationalStorageUnavailableCause,
) -> PatchStreamReadError {
    read_refused(
        crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
            worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageUnavailable {
                subject: worth_relational_contracts::internal_port::storage::data::storage_unavailable::RelationalStorageSubject::Envelope(position),
                cause,
            },
        ),
    )
}

fn read_refused(
    cause: crate::execution::denial_cause::RelationalExecutionDenialCause,
) -> PatchStreamReadError {
    PatchStreamReadError {
        class: PatchStreamReadErrorClass::ReadRequest(cause),
        detail: format!("canonical patch read refused: {cause:?}"),
    }
}
