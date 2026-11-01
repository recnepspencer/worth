//! The stopped World and Serving Native remain owned through coupled selection.
use crate::{ApplicationHomeCloseDenial as Denial, WorldReadmittedApplicationHome};
use worth_execution::ExecutionRequest;

pub(super) fn settle_before_world_drop<D, I, E, Ctx, T>(
    home: &mut WorldReadmittedApplicationHome<D, I, E, Ctx, T>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Denial>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    E: Send + Sync + 'static,
    Ctx: Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    if home.home.context.world_close_checkpoint.is_none() {
        let prior = crate::checkpoint::close_settlement::settle(
            &home.home.context.serving,
            &home.home.context.checkpoint,
            request,
        )
        .map_err(crate::close::checkpoint_settlement_denial)?;
        let result = prior.unwrap_or_else(|| home.checkpoint(request));
        if let Some(pending) = home.home.checkpoint_observation() {
            return Err(Denial::CheckpointRecoveryRequired(pending));
        }
        home.home.context.world_close_checkpoint = Some(result);
    }
    // A settled failure does not veto otherwise lawful release. Actual
    // cancellation/deadline still returns this SAME stopped, owning World.
    crate::close::posture::interrupted(request).map_err(Denial::Execution)
}
