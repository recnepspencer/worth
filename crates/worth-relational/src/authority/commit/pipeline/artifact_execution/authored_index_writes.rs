//! Authored equality-membership writes from the actual selected/working pair.
mod funding;
mod projection;
mod values;
use crate::branch::SelectedRelationalBranchState;
use crate::history::data::{
    CanonicalAuthoredIndexWrite as Entry, CanonicalAuthoredIndexWrites as Evidence,
};
use crate::history::{
    RelationalAuthoredEntityIndexDefinition as Definition, RelationalAuthoredWriteDenial as Denial,
};
use crate::runtime::{RelationalPreparationRuntime, WorkingState};
use crate::transactions::data::{MergedCommitPlan, RecordRef};
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{AspectKey, FieldKey};

pub(super) fn prepare(
    runtime: &RelationalPreparationRuntime,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: &MergedCommitPlan,
    allocations: &[crate::history::data::CanonicalRecordAllocation],
    supported: bool,
    execution: ExecutionRequest<'_, '_>,
) -> Result<Evidence, crate::transactions::data::TransactionCommitError> {
    if !supported {
        return Ok(Evidence::default());
    }
    let prepare = |execution: ExecutionRequest<'_, '_>| {
        prepare_evidence(runtime, selected, working, plan, allocations, execution)
    };
    let evidence = match (
        super::super::request_lease(&execution),
        runtime.commit_work_budget.as_ref(),
    ) {
        (Some(lease), Some(budget)) => budget
            .run_stage(lease, prepare)
            .map_err(Denial::Request)
            .and_then(std::convert::identity),
        _ => prepare(execution),
    };
    evidence.map_err(crate::runtime::authored_write_commit_denial)
}

fn prepare_evidence(
    runtime: &RelationalPreparationRuntime,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: &MergedCommitPlan,
    allocations: &[crate::history::data::CanonicalRecordAllocation],
    execution: ExecutionRequest<'_, '_>,
) -> Result<Evidence, Denial> {
    crate::history::run_authored_write_stage(execution, |context| {
        let catalog = runtime.index_definitions.snapshot();
        let output = funding::Output::new(execution, context)?;
        let mut builder = projection::Projection {
            execution,
            runtime,
            catalog: &catalog,
            unavailable: false,
            selected,
            working,
            context,
            output,
        };
        match builder.project(plan, allocations) {
            Ok(()) => {}
            // An unavailable authoritative image cannot issue complete index
            // evidence. Keep the existing unavailable packet, not guessed keys.
            Err(Denial::SourceUnavailable) => return Ok(Evidence::default()),
            Err(cause) => return Err(cause),
        }
        if builder.unavailable {
            Ok(Evidence::default())
        } else {
            Ok(builder.output.finish(catalog.coverage()))
        }
    })
}

fn work(context: &mut MapKernelContext<'_, '_>, units: u64) -> Result<(), Denial> {
    if context.has_accounting() {
        context.checkpoint(units)?;
    } else {
        context.poll()?;
    }
    Ok(())
}

fn overflow() -> Denial {
    Denial::Interrupted(worth_execution::MapKernelStop::WorkCounterOverflow)
}

struct Locus<'a> {
    aspect: &'a AspectKey,
    field: &'a FieldKey,
}
