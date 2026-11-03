//! Complete the same Native candidate and prepare its lineage slot.
use super::*;

#[inline(never)]
pub(super) fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Option<Progress>, worth_execution::WorkCeilingDenial> {
    context
        .checkpoint(1)
        .map_err(worth_execution::WorkCeilingDenial::Stopped)?;
    let World::NativePreparation(preparation) = take_world(guard, Call::NativeCompletion) else {
        unreachable!()
    };
    let Native::Candidate(candidate) = std::mem::replace(
        &mut guard.preparation_mut().query.native,
        Native::OwnerCallInProgress,
    ) else {
        unreachable!()
    };
    match complete_query_candidate(preparation, candidate, &execution) {
        Ok(prepared) => guard.preparation_mut().world = World::Prepared(prepared),
        Err(cause) => {
            guard.preparation_mut().world = World::NativeRefused(cause);
            guard.preparation_mut().stage = Stage::CompletingNative;
            return Ok(Some(Progress::Pending(Stage::CompletingNative)));
        }
    }
    let lineage = {
        let retained = guard.preparation_mut();
        let World::Prepared(prepared) = &retained.world else {
            unreachable!()
        };
        product_publication::prepare_lineage_slot(
            provider,
            &retained.query.attempt,
            prepared.planned_successor(),
            &mut retained.query.required_prerequisites,
        )
    };
    match lineage {
        Ok(lineage) => guard.preparation_mut().prepared_lineage_slot = lineage,
        Err(Stop::Deferred(deferred)) => {
            guard.preparation_mut().query_denial = Some(Cause::Prerequisite(
                deferred
                    .into_prerequisite_denial()
                    .expect("exact lineage prerequisite cause"),
            ));
            guard.preparation_mut().stage = Stage::PreparingQueryCandidate;
            return Ok(Some(Progress::Pending(Stage::PreparingQueryCandidate)));
        }
        Err(Stop::Denied(cause)) => {
            guard.preparation_mut().query_denial = Some(Cause::QueryEvidence(cause));
            guard.preparation_mut().stage = Stage::PreparingQueryCandidate;
            return Ok(Some(Progress::Pending(Stage::PreparingQueryCandidate)));
        }
        Err(
            Stop::ProductStale(_)
            | Stop::ProductUnpublished(_)
            | Stop::ProductPreparationPending(_)
            | Stop::ProductPreparationRefused(_)
            | Stop::NoEffect(_)
            | Stop::PreEffectDenied(_)
            | Stop::ControlStopped(_),
        ) => unreachable!("lineage only refuses prerequisites or evidence"),
    }
    Ok(None)
}
