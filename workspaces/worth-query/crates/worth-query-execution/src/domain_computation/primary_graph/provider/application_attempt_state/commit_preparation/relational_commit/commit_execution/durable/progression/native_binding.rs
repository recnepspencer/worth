//! Bind the same admitted proposal to its actual World attempt.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    guard: &mut QueryDurablePreparationGuard,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Option<Progress>, worth_execution::WorkCeilingDenial> {
    context
        .checkpoint(1)
        .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
    let World::Opened(opened) = take_world(guard, Call::NativeBinding) else {
        unreachable!()
    };
    let Native::Proposal(proposal) = std::mem::replace(
        &mut guard.preparation_mut().query.native,
        Native::OwnerCallInProgress,
    ) else {
        unreachable!()
    };
    match bind_query_proposal(opened, proposal) {
        Ok((preparation, proposal)) => {
            guard.preparation_mut().world = World::NativePreparation(preparation);
            guard.preparation_mut().query.native = Native::Proposal(proposal);
        }
        Err(cause) => {
            guard.preparation_mut().world = World::NativeRefused(cause);
            guard.preparation_mut().stage = Stage::BindingNativeProposal;
            return Ok(Some(Progress::Pending(Stage::BindingNativeProposal)));
        }
    }
    Ok(None)
}
