use crate::clock::RuntimeInstant;
use crate::data::comparator::ComparatorPolicyResolver;
use crate::data::error::SignalError;
use crate::data::request_preparation::SignalPreparationBudget;
use crate::logic::planner::precompute::callback::SignalPrecompute;

use super::super::precompute_reporting::record_stage_precompute_report;
use super::super::semantic::reserve_stage_identities;
use super::super::stage_apply::apply_stage;
use super::super::stage_precompute::perform_stage_precompute;
use super::super::stage_recording::begin_stage_record;
use super::context::ExecutionContext;
use super::reporting::record_stage_execution_completion;
use super::AdmittedEpoch;

struct StagePreparedPass<'tasks, 'lease, 'authority> {
    precomputed:
        crate::logic::planner::precompute::stage::StagePrecomputeResult<'tasks, 'lease, 'authority>,
    snapshot_nanos: u128,
    precompute_nanos: u128,
}

struct StageAppliedPass {
    stage_record: crate::logic::planner::types::StageExecutionRecord,
    apply_elapsed_nanos: u128,
    stage_elapsed_nanos: u128,
}

pub(crate) fn execute_stage<'tasks, 'lease, 'authority, P, R>(
    ctx: &mut ExecutionContext<'_, '_, '_, 'authority, P, R>,
    stage: AdmittedEpoch<'tasks, 'lease, 'authority>,
    progress: &mut crate::data::error::SignalPublicationProgress,
) -> Result<(), SignalError>
where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    let stage_start = RuntimeInstant::now();
    let prepared_pass = run_stage_precompute_pass(ctx, stage)?;
    progress.prepared_epoch();
    let applied_pass = run_stage_apply_pass(ctx, prepared_pass, stage_start)?;
    complete_stage_reporting_pass(ctx, applied_pass);
    Ok(())
}

fn run_stage_precompute_pass<'tasks, 'lease, 'authority, P, R>(
    ctx: &mut ExecutionContext<'_, '_, '_, 'authority, P, R>,
    stage: AdmittedEpoch<'tasks, 'lease, 'authority>,
) -> Result<StagePreparedPass<'tasks, 'lease, 'authority>, SignalError>
where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    let precomputed = perform_stage_precompute(
        ctx.graph,
        ctx.summary,
        stage,
        ctx.precompute,
        ctx.comparator_resolver,
        &ctx.temporal_lowering,
        &ctx.policy,
        ctx.request_work.as_deref_mut(),
        ctx.preparation.as_deref_mut(),
    )?;
    record_stage_precompute_report(
        &mut ctx.report,
        precomputed.prepared.len(),
        precomputed.snapshot_nanos,
        precomputed.precompute_nanos,
    );
    Ok(StagePreparedPass {
        snapshot_nanos: precomputed.snapshot_nanos,
        precompute_nanos: precomputed.precompute_nanos,
        precomputed,
    })
}

fn run_stage_apply_pass<P, R>(
    ctx: &mut ExecutionContext<'_, '_, '_, '_, P, R>,
    prepared_pass: StagePreparedPass<'_, '_, '_>,
    stage_start: RuntimeInstant,
) -> Result<StageAppliedPass, SignalError>
where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    let apply_start = RuntimeInstant::now();
    reserve_retained(&mut ctx.report.stages, 1, ctx.preparation.as_deref_mut())?;
    let mut stage_record = begin_stage_record(
        prepared_pass.precomputed.prepared.metadata().index(),
        prepared_pass.snapshot_nanos,
        prepared_pass.precompute_nanos,
        prepared_pass.precomputed.prepared.reports(),
    );
    // The grouped apply pass and the enclosing request can each append one
    // more physical report without growing this Vec after publication.
    reserve_retained(
        &mut ctx.report.execution,
        prepared_pass
            .precomputed
            .prepared
            .reports()
            .len()
            .saturating_add(2),
        ctx.preparation.as_deref_mut(),
    )?;
    ctx.report
        .execution
        .extend(prepared_pass.precomputed.prepared.reports().iter().copied());
    if let Some(preparation) = ctx.preparation.as_deref_mut() {
        // Finalization retains one task record per stage task in the returned report.
        preparation.claim_retained_vec::<crate::logic::planner::TaskExecutionRecord>(
            prepared_pass.precomputed.prepared.metadata().tasks().len(),
        )?;
        if !ctx.reuse_origin_storage_claimed {
            // ReuseOrigin has seven variants. The BTreeMap's node allocations are
            // retained with the report, so reserve a conservative node allowance.
            preparation.claim_retained(7 * 256)?;
            ctx.reuse_origin_storage_claimed = true;
        }
    }
    crate::data::request_preparation::claim_vec::<super::super::semantic::StageSemanticIdentity>(
        ctx.preparation.as_deref_mut(),
        prepared_pass.precomputed.prepared.metadata().tasks().len(),
    )?;
    let stage_identities = reserve_stage_identities(
        &mut ctx.next_record_id,
        &mut ctx.next_segment_id,
        prepared_pass.precomputed.prepared.metadata().tasks().len(),
    );

    apply_stage::<R>(
        ctx.graph,
        ctx.summary,
        prepared_pass.precomputed,
        ctx.comparator_resolver,
        &ctx.policy,
        &stage_identities,
        &mut ctx.report,
        &mut stage_record,
        ctx.request_work.as_deref_mut(),
        ctx.preparation.as_deref_mut(),
    )?;
    Ok(StageAppliedPass {
        stage_record,
        apply_elapsed_nanos: apply_start.elapsed().as_nanos(),
        stage_elapsed_nanos: stage_start.elapsed().as_nanos(),
    })
}

fn reserve_retained<T>(
    values: &mut Vec<T>,
    additional: usize,
    preparation: Option<&mut SignalPreparationBudget>,
) -> Result<(), SignalError> {
    let required = values
        .len()
        .checked_add(additional)
        .ok_or_else(|| SignalError::invalid_input("Signal report capacity overflow"))?;
    if required <= values.capacity() {
        return Ok(());
    }
    let capacity = values.capacity().saturating_mul(2).max(4).max(required);
    if let Some(preparation) = preparation {
        preparation.claim_retained_vec::<T>(capacity)?;
    }
    values.reserve_exact(capacity - values.len());
    Ok(())
}

fn complete_stage_reporting_pass<P, R>(
    ctx: &mut ExecutionContext<'_, '_, '_, '_, P, R>,
    applied_pass: StageAppliedPass,
) where
    P: SignalPrecompute,
    R: ComparatorPolicyResolver,
{
    record_stage_execution_completion(
        ctx.graph,
        &mut ctx.report,
        applied_pass.stage_record,
        applied_pass.apply_elapsed_nanos,
        applied_pass.stage_elapsed_nanos,
    );
}
