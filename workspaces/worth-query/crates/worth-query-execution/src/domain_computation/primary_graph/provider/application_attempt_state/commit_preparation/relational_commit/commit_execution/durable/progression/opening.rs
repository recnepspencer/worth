//! Advance the exact retained World open before any Native preparation.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    guard: &mut QueryDurablePreparationGuard,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Option<Progress>, worth_execution::WorkCeilingDenial> {
    loop {
        context
            .checkpoint(1)
            .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
        let World::PendingOpen(open) = take_world(guard, Call::Open) else {
            unreachable!()
        };
        // Store every actual outcome before the next checkpoint or yield.
        match open.advance(&execution) {
            WorldOpenAttemptOutcome::Opened(open) => {
                guard.preparation_mut().world = World::Opened(open);
                break;
            }
            WorldOpenAttemptOutcome::Pending(open) => {
                guard.preparation_mut().world = World::PendingOpen(open);
                guard.preparation_mut().stage = Stage::OpenPending;
                std::thread::yield_now();
            }
            WorldOpenAttemptOutcome::Unresolved(open) => {
                guard.preparation_mut().world = World::PendingOpen(open);
                guard.preparation_mut().stage = Stage::OpenUnresolved;
                return Ok(Some(Progress::Pending(Stage::OpenUnresolved)));
            }
            WorldOpenAttemptOutcome::EvidenceRequired(open) => {
                guard.preparation_mut().world = World::PendingOpen(open);
                guard.preparation_mut().stage = Stage::OpenEvidenceRequired;
                return Ok(Some(Progress::Pending(Stage::OpenEvidenceRequired)));
            }
            WorldOpenAttemptOutcome::Refused(cause) => {
                guard.preparation_mut().world = World::OpenRefused(cause);
                return Ok(Some(Progress::Terminal));
            }
        }
    }
    Ok(None)
}
