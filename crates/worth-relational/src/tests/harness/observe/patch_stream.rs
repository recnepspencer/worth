use crate::facade::publication::{
    PatchStreamPosition, PatchStreamRequest, PublishedAuthoritativePatchEnvelope,
};
use crate::facade::runtime::RelationalRuntime;

pub(crate) fn collect_patch_stream_from_head(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    collect_patch_stream_after(execution, runtime, None, window_size)
}

pub(crate) fn collect_patch_stream_after(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    after_position: Option<PatchStreamPosition>,
    window_size: usize,
) -> Vec<PublishedAuthoritativePatchEnvelope> {
    let mut collected = Vec::new();
    let mut after = after_position;

    loop {
        let batch = runtime
            .publication()
            .read_patch_stream(
                execution,
                PatchStreamRequest {
                    after_position: after,
                    max_commits: window_size,
                },
            )
            .unwrap();

        if batch.patches.is_empty() {
            break;
        }

        after = batch.patches.last().map(|patch| patch.position);
        collected.extend(batch.patches);
    }

    collected
}
