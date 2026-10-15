use crate::domain_computation::execution_runtime::product_world::publication_denial::WorthQueryProductPublicationDenial as PublicationDenial;
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use std::collections::{BTreeMap, BTreeSet};

use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, RevalidateEntityIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::workflow::{admit_workflow_dispositions, workflow_read_denial};
use super::{
    dispositions, selection, WorthQueryBranchAdoptionPreparationDenial,
    WorthQueryPreparedBranchAdoption, WorthQueryPreparedProgramMigration,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;
use crate::domain_computation::primary_graph::workflow::adoption::{
    inventory_workflows, stage_workflow_dispositions, WorkflowAdoptionInventoryRequest,
};
use crate::domain_computation::primary_graph::workflow::{
    WorthQueryWorkflowAdoptionInventory, WorthQueryWorkflowDispositions,
};

fn map_relational_preparation_denial(
    denial: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryBranchAdoptionPreparationDenial {
    // Execution control remains typed, but native partition context and the
    // private workflow candidate's commit log never cross the disclosure boundary.
    use worth_relational::facade::transactions::{
        CommitExecutionDenialKind, RelationalExecutionDenialCause as Cause, TransactionCommitError,
    };
    let execution_cause = match &denial {
        TransactionCommitError::Execution { denial, .. } => {
            let CommitExecutionDenialKind::Cause(cause) = denial.kind;
            Some(cause)
        }
        TransactionCommitError::Interrupted { interruption, .. } => {
            Some(match interruption.interruption() {
                worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
                    Cause::Cancelled
                }
                worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
                    Cause::DeadlineElapsed
                }
            })
        }
        _ => None,
    };
    if let Some(cause) = execution_cause {
        return WorthQueryBranchAdoptionPreparationDenial::ReadExecution(
            crate::facade::primary_graph::relational_execution_kind(cause, None),
        );
    }
    if let worth_relational::facade::transactions::TransactionCommitError::Conflict {
        error, ..
    } = &denial
        && let worth_relational::facade::transactions::ConflictClass::InvariantViolation {
            fields:
                worth_relational::facade::transactions::InvariantViolationFields::CustomInvariantViolation {
                    identity,
                },
            ..
        } = &error.class
        {
            return WorthQueryBranchAdoptionPreparationDenial::TargetRuleRejected {
                identity: identity.clone(),
            };
        }
    WorthQueryBranchAdoptionPreparationDenial::RelationalPreparation
}

pub(super) fn prepare<Schema: ApplicationSchema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    admission: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    _phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
    expected_requirements: &WorthQueryProgramAdoptionRequirements,
    migration: Option<WorthQueryPreparedProgramMigration>,
    workflow: Option<WorthQueryWorkflowDispositions>,
    maximum_selection_work: usize,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
    admission.validate_current_authority().map_err(|cause| WorthQueryBranchAdoptionPreparationDenial::Authorization(super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial::Authorization(cause)))?;
    let (source, requirements) = requirements(selected, target)?;
    if &requirements != expected_requirements {
        return Err(WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged);
    }
    if migration.is_none()
        && let Some(requirement) = requirements.migration_assessment_requirements().first()
    {
        return Err(
            WorthQueryBranchAdoptionPreparationDenial::MigrationAssessmentRequired(
                requirement.clone(),
            ),
        );
    }
    let custody = dispositions::derive(&requirements)
        .map_err(WorthQueryBranchAdoptionPreparationDenial::CustodyDispositionUnsupported)?;
    let application = selected.application();
    if let Some(candidate) = migration.as_ref() {
        if candidate.target() != target {
            return Err(WorthQueryBranchAdoptionPreparationDenial::MigrationTargetMismatch);
        }
        if candidate.source_product() != selected.product().observation() {
            return Err(WorthQueryBranchAdoptionPreparationDenial::MigrationSourceChanged);
        }
    }
    let (migration_description, migration_batch) = match migration {
        Some(candidate) => {
            let (_, _, description, batch) = candidate.into_parts();
            (Some(description), Some(batch))
        }
        None => (None, None),
    };
    let migrated_entities = migration_batch
        .as_ref()
        .map_or_else(BTreeSet::new, |batch| {
            batch
                .intents
                .iter()
                .filter_map(existing_entity_target)
                .collect()
        });
    let support = application
        .installed_program_support()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable)?;
    let support_custody = support
        .retain_custody(&source, target)
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportRetirementInProgress)?;
    let activation = support
        .activation()
        .published()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnavailable)?;
    let graph = &application.primary_provider.graph;
    let layout = graph.layout.program_activation().clone();
    let basis = selected.product().relational_basis();

    // One view of the branch's own root serves both reads.
    let (selection, workflow_inventory) = graph.with_runtime(execution, |runtime| {
        let view = selection::branch_view(execution, runtime, basis)?;
        let selection = selection::select(
            execution,
            &view,
            &graph.layout,
            &requirements,
            maximum_selection_work,
        )?;
        if source == *target {
            return Ok((selection, None));
        }
        let workflow_request = WorkflowAdoptionInventoryRequest {
            layout: graph.layout.workflow(),
            branch_occurrence: selected.product().product_branch().occurrence_ordinal(),
            coverage: &application.workflow_coverage,
            requirements: &requirements,
            source: &source,
            target,
            maximum_work_units: maximum_selection_work.saturating_sub(selection.work_units),
        };
        let inventory = inventory_workflows(execution, view, &workflow_request)
            .map_err(|denial| workflow_read_denial(denial))?;
        Ok((selection, Some(inventory)))
    })?;
    let selected_entity_count = selection.entities.len();
    let workflow_intents = match workflow_inventory.as_ref() {
        Some(inventory) => admit_workflow_dispositions(inventory, workflow.as_ref(), selected.product().product_branch())?
            .map(|choices| {
                stage_workflow_dispositions(graph.layout.workflow(), inventory, &choices, target,
                    crate::domain_computation::primary_graph::workflow::actor::record::WorkflowAdmittedActor::from_admission(&admission))
            })
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let selection_work_units = selection.work_units.saturating_add(
        workflow_inventory
            .as_ref()
            .map_or(0, WorthQueryWorkflowAdoptionInventory::work_units),
    );
    let target_rendering = program_revision_rendering(target);
    admission.validate_current_authority().map_err(|cause| WorthQueryBranchAdoptionPreparationDenial::Authorization(super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial::Authorization(cause)))?;
    let mut candidate = graph.with_runtime_mut(execution, |runtime| {
        let mut batch =
            WorkerIntentBatch::new("application-program-adoption").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: activation,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        layout.program_revision_locator.clone(),
                        target_rendering,
                    )])),
                }),
            ));
        for entity_id in selection.entities {
            if migrated_entities.contains(&entity_id) {
                continue;
            }
            batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                RevalidateEntityIntent { entity_id },
            )));
        }
        for intent in workflow_intents {
            batch = batch.push(intent);
        }
        let mut transaction = runtime
            .begin_branch_transaction(
                selected.product().relational_basis(),
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .map_err(WorthQueryBranchAdoptionPreparationDenial::TransactionAdmission)?;
        if let Some(migration_batch) = migration_batch {
            transaction
                .push_batch(
                    migration_batch,
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .map_err(WorthQueryBranchAdoptionPreparationDenial::TransactionStaging)?;
        }
        transaction
            .push_batch(
                batch,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .map_err(WorthQueryBranchAdoptionPreparationDenial::TransactionStaging)?;
        runtime
            .prepare_branch_transaction(execution, transaction)
            .map_err(map_relational_preparation_denial)
    })?;
    super::selector_indexes::prepare(execution, selected, &mut candidate)?;
    let successor_observation_requested = false;
    let recovery = selected.product().publication_binding().recovery();
    let disposition = selected
        .application()
        .primary_provider
        .unpublished_idempotency_disposition();
    admission.validate_current_authority().map_err(|cause| WorthQueryBranchAdoptionPreparationDenial::Authorization(super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial::Authorization(cause)))?;
    let publication = selected
        .product()
        .publication_binding()
        .prepare_relational_candidate(candidate, request, successor_observation_requested)
        .map_err(|denial| match denial {
            PublicationDenial::Stopped(stop) => {
                WorthQueryBranchAdoptionPreparationDenial::ExecutionDenied(advancement_denial(stop))
            }
            PublicationDenial::Refused(no_effect) => {
                WorthQueryBranchAdoptionPreparationDenial::WorldPreparation(no_effect)
            }
        })?;
    Ok(WorthQueryPreparedBranchAdoption {
        branch: selected.product().product_branch(),
        owner_identity: selected
            .application()
            .product_runtime
            .owner
            .owner_identity(),
        source,
        target: *target,
        requirements,
        selected_entity_count,
        selection_work_units,
        migration: migration_description,
        custody,
        publication,
        recovery,
        disposition,
        support_custody,
    })
}

fn existing_entity_target(
    intent: &MutationIntent,
) -> Option<worth_relational::facade::identity::EntityId> {
    let MutationIntent::Entity(intent) = intent else {
        return None;
    };
    Some(match intent {
        EntityMutationIntent::UpdateFields(intent) => intent.entity_id,
        EntityMutationIntent::ApplyAspectPatch(intent) => intent.entity_id,
        EntityMutationIntent::Replace(intent) => intent.entity_id,
        EntityMutationIntent::Delete(intent) => intent.entity_id,
        EntityMutationIntent::Revalidate(intent) => intent.entity_id,
    })
}

pub(super) fn requirements<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
) -> Result<
    (
        ApplicationProgramRevision,
        WorthQueryProgramAdoptionRequirements,
    ),
    WorthQueryBranchAdoptionPreparationDenial,
> {
    let application = selected.application();
    let source = *selected
        .inspect_selected_program()
        .map_err(|denial| match denial {
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial::ReadExecution(cause) => WorthQueryBranchAdoptionPreparationDenial::ReadExecution(cause),
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial::ProgramSupportUnavailable => {
                WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable
            }
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnavailable => {
                WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnavailable
            }
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnreadable => {
                WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnreadable
            }
            crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial::ProgramActivationUnrostered => {
                WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnrostered
            }
        })?
        .revision();
    let support = application
        .installed_program_support()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable)?;
    let requirements = support
        .adoption_requirements(application.installed_schema(), &source, target)
        .map_err(WorthQueryBranchAdoptionPreparationDenial::Requirements)?;
    Ok((source, requirements))
}
