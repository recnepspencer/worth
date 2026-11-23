//! Authored equality-membership writes from the actual selected/working pair.
mod funding;
mod projection;
mod values;
use crate::authority::commit::semantic_plan::RelationalSemanticPlan;
use crate::branch::root_selection::SelectedRelationalBranchState;
use crate::history::access::relevant_writes::RelationalAuthoredEntityIndexDefinition as Definition;
use crate::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use crate::history::data::{
    CanonicalAuthoredIndexWrite as Entry, CanonicalAuthoredIndexWrites as Evidence,
};
use crate::runtime::RelationalPreparationRuntime;
use crate::storage::overlay::WorkingState;
use crate::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{AspectKey, FieldKey};

pub(super) fn prepare(
    runtime: &RelationalPreparationRuntime,
    catalog: &crate::runtime::state::subsystems::indexing::CapturedIndexDefinitionCatalog,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: RelationalSemanticPlan<'_>,
    allocations: &[crate::history::data::CanonicalRecordAllocation],
    supported: bool,
    execution: ExecutionRequest<'_, '_>,
) -> Result<
    Evidence,
    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    if !supported
        || runtime
            .recovered_authored_index_writes
            .as_ref()
            .is_some_and(Evidence::is_unavailable)
    {
        return Ok(Evidence::default());
    }
    crate::execution::phase_observation::run(
        &execution,
        "authored_index_writes",
        || {
            let prepare = |execution: ExecutionRequest<'_, '_>| {
                prepare_evidence(
                    runtime,
                    catalog,
                    selected,
                    working,
                    plan,
                    allocations,
                    execution,
                )
            };
            let evidence = match (
                super::super::request_lease(&execution),
                runtime.commit_work_budget.as_ref(),
            ) {
                (Some(_), Some(budget)) => budget
                    .run_stage(execution, || prepare(execution))
                    .map_err(Denial::Request)
                    .and_then(std::convert::identity),
                _ => prepare(execution),
            };
            let evidence = evidence.map_err(crate::runtime::state::subsystems::history::canonical_publication_routes::authored_write_commit_denial)?;
            if runtime
                .recovered_authored_index_writes
                .as_ref()
                .is_some_and(|original| original != &evidence)
            {
                return Err(
                    crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                        crate::publication::data::PublicationError::new(
                            crate::publication::bundle::PublicationStage::BundleAssembly,
                            "recovered authored index writes disagree with original source",
                        ),
                    ),
                );
            }
            Ok(evidence)
        },
        |cause| {
            crate::runtime::state::subsystems::history::canonical_publication_routes::authored_write_commit_denial(Denial::Execution(cause))
        },
    )
}

fn prepare_evidence(
    runtime: &RelationalPreparationRuntime,
    catalog: &crate::runtime::state::subsystems::indexing::CapturedIndexDefinitionCatalog,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: RelationalSemanticPlan<'_>,
    allocations: &[crate::history::data::CanonicalRecordAllocation],
    execution: ExecutionRequest<'_, '_>,
) -> Result<Evidence, Denial> {
    crate::history::access::relevant_writes::run(execution, |context| {
        let output = funding::Output::new(execution, context)?;
        if !catalog.catalog.has_entity_field_indexes() {
            // The actual installed catalog proves complete-empty membership
            // independently of record residency. Missing data never makes this proof.
            return Ok(output.finish(catalog.coverage));
        }
        let mut builder = projection::Projection {
            execution,
            runtime,
            catalog,
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
            Ok(builder.output.finish(catalog.coverage))
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
