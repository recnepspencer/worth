//! Advance the same actual World phases under the original request ceiling.
use super::super::durable_custody::{QueryDurableOwnerCall as Call, QueryNativeCustody as Native};
use super::*;
use worth_runtime_world::facade::{WorldDurablePublicationOutcome, WorldOpenAttemptOutcome};

pub(super) enum Progress {
    Terminal,
    Pending(Stage),
}
fn take_world(guard: &mut QueryDurablePreparationGuard, call: Call) -> World {
    std::mem::replace(
        &mut guard.preparation_mut().world,
        World::OwnerCallInProgress(call),
    )
}
pub(super) fn run(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<Progress, worth_execution::WorkCeilingDenial> {
    // The concrete guard lives outside both scopes, including unwind containment.
    execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    advance(execution, provider, guard, allocation, context)
                })
        })?
        .map(|(outcome, _)| outcome)?
}
fn advance(
    execution: worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    guard: &mut QueryDurablePreparationGuard,
    allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<Progress, worth_execution::WorkCeilingDenial> {
    use worth_execution::WorkCeilingDenial::Stopped;
    loop {
        context.checkpoint(1).map_err(Stopped)?;
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
                return Ok(Progress::Pending(Stage::OpenUnresolved));
            }
            WorldOpenAttemptOutcome::EvidenceRequired(open) => {
                guard.preparation_mut().world = World::PendingOpen(open);
                guard.preparation_mut().stage = Stage::OpenEvidenceRequired;
                return Ok(Progress::Pending(Stage::OpenEvidenceRequired));
            }
            WorldOpenAttemptOutcome::Refused(cause) => {
                guard.preparation_mut().world = World::OpenRefused(cause);
                return Ok(Progress::Terminal);
            }
        }
    }
    context.checkpoint(1).map_err(Stopped)?;
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
            return Ok(Progress::Pending(Stage::BindingNativeProposal));
        }
    }
    // Keep WorldQueryNativePreparation outside the one consuming Native call.
    context.checkpoint(1).map_err(Stopped)?;
    if let Err(cause) = native_work::prepare_native_candidate(
        &execution,
        provider,
        &mut guard.preparation_mut().query,
    ) {
        guard.preparation_mut().query_denial = Some(Cause::Candidate(cause));
        guard.preparation_mut().stage = Stage::PreparingNative;
        return Ok(Progress::Pending(Stage::PreparingNative));
    }
    if let Err(cause) = native_work::prepare_candidate_support(
        &execution,
        provider,
        &mut guard.preparation_mut().query,
        allocation,
    ) {
        guard.preparation_mut().query_denial = Some(Cause::Candidate(cause));
        guard.preparation_mut().stage = Stage::PreparingQueryCandidate;
        return Ok(Progress::Pending(Stage::PreparingQueryCandidate));
    }
    context.checkpoint(1).map_err(Stopped)?;
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
            return Ok(Progress::Pending(Stage::CompletingNative));
        }
    }
    let product = guard
        .preparation_mut()
        .query
        .attempt
        .affinity()
        .product_publication()
        .clone();
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
            return Ok(Progress::Pending(Stage::PreparingQueryCandidate));
        }
        Err(Stop::Denied(cause)) => {
            guard.preparation_mut().query_denial = Some(Cause::QueryEvidence(cause));
            guard.preparation_mut().stage = Stage::PreparingQueryCandidate;
            return Ok(Progress::Pending(Stage::PreparingQueryCandidate));
        }
        Err(_) => unreachable!("lineage only refuses prerequisites or evidence"),
    }
    loop {
        context.checkpoint(1).map_err(Stopped)?;
        let outcome = if matches!(guard.preparation_mut().world, World::Prepared(_)) {
            let World::Prepared(prepared) = take_world(guard, Call::Publication) else {
                unreachable!()
            };
            product.publication().execute_durable_without_signal(
                execution,
                prepared,
                guard.preparation_mut().control.cancellation(),
            )
        } else {
            let World::PendingPublication(pending) = take_world(guard, Call::PublicationAdvance)
            else {
                unreachable!()
            };
            pending.advance(&execution)
        };
        match outcome {
            WorldDurablePublicationOutcome::Performed(performed) => {
                guard.preparation_mut().world = World::Performed(performed);
                return Ok(Progress::Terminal);
            }
            WorldDurablePublicationOutcome::Pending(pending) => {
                guard.preparation_mut().world = World::PendingPublication(pending);
                guard.preparation_mut().stage = Stage::PublicationPending;
                std::thread::yield_now();
            }
            WorldDurablePublicationOutcome::ProductUnpublished(effects) => {
                guard.preparation_mut().world = World::Unpublished(effects);
                return Ok(Progress::Terminal);
            }
            WorldDurablePublicationOutcome::RecoveryRequired { cause, custody } => {
                guard.preparation_mut().world =
                    World::PublicationRecoveryRequired { cause, custody };
                guard.preparation_mut().stage = Stage::PublicationRecoveryRequired;
                return Ok(Progress::Pending(Stage::PublicationRecoveryRequired));
            }
        }
    }
}
