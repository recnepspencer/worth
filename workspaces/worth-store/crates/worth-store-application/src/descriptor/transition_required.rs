//! Admit the detailed refusal's dynamic metadata before decoding or allocating it.
use super::{binding, ApplicationHomeDescriptorDenial as Denial};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ApplicationHomeBinding;

pub(super) fn refuse(
    recorded: &[u8],
    requested: ApplicationHomeBinding,
    request: &ExecutionRequest<'_, '_>,
) -> Denial {
    let result = (|| {
        let work = u64::try_from(recorded.len())
            .ok()
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or(Denial::SizeLimit)?;
        request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(work)
                } else {
                    context.poll()
                }
            })
            .map_err(Denial::Execution)?
            .0
            .map_err(|cause| {
                Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause))
            })?;
        let memory = request
            .reserve_retained_memory(binding::allocation_bytes(recorded)?)
            .map_err(Denial::Execution)?;
        let recorded = binding::decode(recorded)?;
        Ok(Denial::TransitionRequired {
            recorded: Box::new(recorded),
            requested: Box::new(requested),
            _funding: memory,
        })
    })();
    match result {
        Ok(refusal) | Err(refusal) => refusal,
    }
}
