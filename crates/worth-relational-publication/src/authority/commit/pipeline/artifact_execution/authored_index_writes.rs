//! Authored equality-membership writes from the actual selected/working pair.
use worth_relational_contracts::internal_port as contracts;
pub mod funding;
pub mod projection;
pub mod values;
use worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState;
use worth_relational_branch::internal_port::history::access::relevant_writes::selection::RelationalAuthoredEntityIndexDefinition as Definition;
use worth_relational_branch::internal_port::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::authored_index_writes::{CanonicalAuthoredIndexWrite as Entry, CanonicalAuthoredIndexWrites as Evidence};
#[derive(Clone, Copy)]
pub struct AuthoredIndexWriteInputs<'a> {
    pub schema_authority_digest: [u8; 32],
    pub original: Option<&'a Evidence>,
    pub work: Option<&'a contracts::execution::request_work::RequestWorkBudget>,
}
use contracts::transactions::data::primitives::RecordRef;
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub(super) fn prepare(
    inputs: AuthoredIndexWriteInputs<'_>,
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: RelationalSemanticPlan<'_>,
    allocations: &[contracts::history::data::canonical_record_allocation::CanonicalRecordAllocation],
    supported: bool,
    execution: ExecutionRequest<'_, '_>,
) -> Result<
    Evidence,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    if !supported || inputs.original.is_some_and(Evidence::is_unavailable) {
        return Ok(Evidence::default());
    }
    contracts::execution::phase_observation::run(
        &execution,
        "authored_index_writes",
        || {
            let prepare = |execution: ExecutionRequest<'_, '_>| {
                prepare_evidence(
                    inputs,
                    catalog,
                    selected,
                    working,
                    plan,
                    allocations,
                    execution,
                )
            };
            let evidence = match (
                contracts::execution::request_custody::request_lease(&execution),
                inputs.work,
            ) {
                (Some(_), Some(budget)) => budget
                    .run_stage(execution, || prepare(execution))
                    .map_err(Denial::Request)
                    .and_then(std::convert::identity),
                _ => prepare(execution),
            };
            let evidence = evidence.map_err(crate::runtime::state::subsystems::history::canonical_publication_routes::authored_write_commit_denial)?;
            if inputs
                .original
                .is_some_and(|original| original != &evidence)
            {
                return Err(
                    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::publication(
                        contracts::publication::data::publication_error::PublicationError::new(
                            contracts::publication::bundle::publication_status::PublicationStage::BundleAssembly,
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
    inputs: AuthoredIndexWriteInputs<'_>,
    catalog: &worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::catalog_publication::CapturedIndexDefinitionCatalog,
    selected: &SelectedRelationalBranchState,
    working: &WorkingState,
    plan: RelationalSemanticPlan<'_>,
    allocations: &[contracts::history::data::canonical_record_allocation::CanonicalRecordAllocation],
    execution: ExecutionRequest<'_, '_>,
) -> Result<Evidence, Denial> {
    worth_relational_branch::internal_port::history::access::relevant_writes::run(
        execution,
        |context| {
            let output = funding::Output::new(execution, context)?;
            if !catalog.catalog.has_entity_field_indexes() {
                // The actual installed catalog proves complete-empty membership
                // independently of record residency. Missing data never makes this proof.
                return Ok(output.finish(catalog.coverage));
            }
            let mut builder = projection::Projection {
                execution,
                schema_authority_digest: inputs.schema_authority_digest,
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
        },
    )
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
