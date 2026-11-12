//! Advance the exact retained World open before any Native preparation.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
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
    let preparation = guard.preparation_mut();
    let World::Opened(opened) = &preparation.world else {
        unreachable!()
    };
    preparation.stage = Stage::RecordingOriginalMeaning;
    if let Err(cause) = provider.acknowledge_original_meaning(
        preparation
            .query
            .attempt
            .original_authority()
            .original_program(),
        opened,
        &execution,
    ) {
        preparation.query_denial = Some(Cause::OriginalMeaning(cause));
        return Ok(Some(Progress::Pending(Stage::RecordingOriginalMeaning)));
    }
    preparation.stage = Stage::RecordingCorrectionMaterial;
    if let Some(causality) = preparation.query.attempt.aftermath_causality()
        && let Err(cause) = provider.transfer_correction_preparation(
            causality.prepared_correction(),
            preparation.query.attempt.original_authority(),
            opened,
            &execution,
        )
    {
        preparation.query_denial = Some(Cause::CorrectionMaterial(cause));
        return Ok(Some(Progress::Pending(Stage::RecordingCorrectionMaterial)));
    }
    if let Err(cause) = provider.capture_correction_material(
        preparation.query.attempt.original_authority(),
        preparation.query.retained_preimage.as_ref(),
        opened,
        &execution,
    ) {
        preparation.query_denial = Some(Cause::CorrectionMaterial(cause));
        return Ok(Some(Progress::Pending(Stage::RecordingCorrectionMaterial)));
    }
    preparation.stage = Stage::BindingNativeProposal;
    Ok(None)
}
