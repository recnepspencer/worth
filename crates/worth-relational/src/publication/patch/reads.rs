use crate::capabilities::PatchStreamSource;
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
    crate::execution::check_request(execution).map_err(read_refused)?;
    if request.max_commits == 0 {
        return Err(PatchStreamReadError {
            class: PatchStreamReadErrorClass::InvalidBatchSize,
            detail: "patch stream request must ask for at least one commit".to_string(),
        });
    }
    let latest_position = runtime.history().latest_patch_stream_position();
    let latest_commit_id = runtime.history().latest_canonical_commit_id();
    if let Some(after_position) = request.after_position
        && !runtime
            .history()
            .contains_patch_stream_position(after_position)
    {
        return Err(PatchStreamReadError {
            class: PatchStreamReadErrorClass::UnknownResumePosition,
            detail: format!("unknown patch stream resume position {}", after_position.0),
        });
    }
    let references =
        runtime.patch_stream_commits_after(request.after_position, request.max_commits);
    let mut patches = Vec::with_capacity(references.len());
    for reference in references {
        let envelope = match runtime.history.commit_artifact(reference.commit_id) {
            Some(artifact) => artifact.demand_envelope(execution, 1 << 30)
                .map_err(|failure| read_refused(failure.into()))?.clone(),
            None => crate::publication::retained_canonical_envelope_at_position(runtime, reference.position, None)
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
    crate::execution::check_request(execution).map_err(read_refused)?;
    Ok(PatchStreamBatch {
        resumed_after: request.after_position,
        next_position: patches.last().map(|patch| patch.position),
        latest_position,
        latest_commit_id,
        patches,
    })
}

fn read_refused(cause: crate::execution::RelationalExecutionDenialCause) -> PatchStreamReadError {
    PatchStreamReadError {
        class: PatchStreamReadErrorClass::ReadRequest(cause),
        detail: format!("canonical patch read refused: {cause:?}"),
    }
}
