use crate::publication::patch::data::{
    PatchStreamBatch, PatchStreamReadError, PatchStreamReadErrorClass, PatchStreamRequest,
};
use crate::publication::{retained_canonical_envelopes_after, RetainedCanonicalEnvelopeGap};
use crate::runtime::RelationalRuntime;

pub(crate) fn read_patch_stream(
    runtime: &RelationalRuntime,
    request: PatchStreamRequest,
) -> Result<PatchStreamBatch, PatchStreamReadError> {
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

    let patches: Vec<_> =
        retained_canonical_envelopes_after(runtime, request.after_position, request.max_commits)
            .map_err(retained_history_gap)?
            .into_iter()
            .map(|commit| commit.published_patch())
            .collect();

    Ok(PatchStreamBatch {
        resumed_after: request.after_position,
        next_position: patches.last().map(|patch| patch.position),
        latest_position,
        latest_commit_id,
        patches,
    })
}

fn retained_history_gap(gap: RetainedCanonicalEnvelopeGap) -> PatchStreamReadError {
    PatchStreamReadError {
        class: PatchStreamReadErrorClass::RetainedHistoryGap,
        detail: format!(
            "patch stream position {} is retained for commit {} but has no retained canonical envelope or durable recovery coverage",
            gap.position.0, gap.commit_id.0
        ),
    }
}
