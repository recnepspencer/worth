//! Prepare Native and Query support while the existing guard owns their inputs.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Option<Progress>, worth_execution::WorkCeilingDenial> {
    // Keep WorldQueryNativePreparation outside the one consuming Native call.
    context
        .checkpoint(1)
        .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
    if let Err(cause) = native_work::prepare_native_candidate(
        &execution,
        provider,
        &mut guard.preparation_mut().query,
    ) {
        guard.preparation_mut().query_denial = Some(Cause::Candidate(cause));
        guard.preparation_mut().stage = Stage::PreparingNative;
        return Ok(Some(Progress::Pending(Stage::PreparingNative)));
    }
    if let Err(cause) = native_work::prepare_candidate_support(
        &execution,
        provider,
        &mut guard.preparation_mut().query,
        allocation,
    ) {
        guard.preparation_mut().query_denial = Some(Cause::Candidate(cause));
        guard.preparation_mut().stage = Stage::PreparingQueryCandidate;
        return Ok(Some(Progress::Pending(Stage::PreparingQueryCandidate)));
    }
    Ok(None)
}
