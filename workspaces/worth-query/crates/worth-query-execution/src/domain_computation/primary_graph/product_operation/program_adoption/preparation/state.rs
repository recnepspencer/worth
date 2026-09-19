use std::collections::BTreeMap;

use worth_foundational::facade::ContractValidatedAspectValueView;
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{ApplicationSchema, WorthQueryProgramAdoptionRequirements};
use worth_relational::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, RevalidateEntityIntent,
    UpdateEntityFieldsIntent, WorkerIntentBatch,
};

use super::{
    selection, WorthQueryBranchAdoptionPreparationDenial, WorthQueryPreparedBranchAdoption,
};
use crate::domain_computation::primary_graph::product_operation::WorthQuerySelectedProductOperation;
use crate::domain_computation::primary_graph::program_occurrence::program_revision_rendering;

fn map_relational_preparation_denial(
    denial: worth_relational::facade::transactions::TransactionCommitError,
) -> WorthQueryBranchAdoptionPreparationDenial {
    if let worth_relational::facade::transactions::TransactionCommitError::Conflict {
        error, ..
    } = &denial
    {
        if let worth_relational::facade::transactions::ConflictClass::InvariantViolation {
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
    }
    WorthQueryBranchAdoptionPreparationDenial::RelationalPreparation(denial)
}

pub(super) fn prepare<Schema: ApplicationSchema>(
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    target: &ApplicationProgramRevision,
    expected_requirements: &WorthQueryProgramAdoptionRequirements,
    maximum_selection_work: usize,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Result<WorthQueryPreparedBranchAdoption, WorthQueryBranchAdoptionPreparationDenial> {
    let (source, requirements) = requirements(selected, target)?;
    if &requirements != expected_requirements {
        return Err(WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged);
    }
    let application = selected.application();
    let support = application
        .installed_program_support()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable)?;
    let activation = support
        .activation()
        .published()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnavailable)?;
    let graph = &application.primary_provider.graph;
    let layout = graph.layout.program_activation().clone();
    let version = selected
        .product()
        .relational_basis()
        .observation()
        .version_id();

    let selection = graph.with_runtime(|runtime| {
        selection::select(
            runtime,
            &graph.layout,
            version,
            &requirements,
            maximum_selection_work,
        )
    })?;
    let selected_entity_count = selection.entities.len();
    let selection_work_units = selection.work_units;
    let target_rendering = program_revision_rendering(target);
    let candidate = graph.with_runtime_mut(|runtime| {
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
            batch = batch.push(MutationIntent::Entity(EntityMutationIntent::Revalidate(
                RevalidateEntityIntent { entity_id },
            )));
        }
        let mut transaction = runtime
            .begin_branch_transaction(
                selected.product().relational_basis(),
                worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary(),
            )
            .map_err(WorthQueryBranchAdoptionPreparationDenial::TransactionAdmission)?;
        transaction
            .push_batch(batch)
            .map_err(WorthQueryBranchAdoptionPreparationDenial::TransactionStaging)?;
        runtime
            .prepare_branch_transaction(transaction)
            .map_err(map_relational_preparation_denial)
    })?;
    let successor_observation_requested = false;
    let recovery = selected.product().publication_binding().recovery();
    let disposition = selected
        .application()
        .primary_provider
        .unpublished_idempotency_disposition();
    let publication = selected
        .product()
        .publication_binding()
        .prepare_relational_candidate(candidate, request, successor_observation_requested)
        .map_err(WorthQueryBranchAdoptionPreparationDenial::WorldPreparation)?;
    Ok(WorthQueryPreparedBranchAdoption {
        source,
        target: target.clone(),
        requirements,
        selected_entity_count,
        selection_work_units,
        publication,
        recovery,
        disposition,
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
    let support = application
        .installed_program_support()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramSupportUnavailable)?;
    let activation = support
        .activation()
        .published()
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnavailable)?;
    let graph = &application.primary_provider.graph;
    let layout = graph.layout.program_activation().clone();
    let version = selected
        .product()
        .relational_basis()
        .observation()
        .version_id();

    let source_rendering = graph
        .with_runtime(|runtime| {
            let record = runtime
                .read_truth()
                .visible_entity_at_version(activation, version)?;
            let state = record.authoritative_aspect_state.as_ref()?;
            let value = state.get(layout.program_revision_locator.aspect().aspect_key())?;
            let ContractValidatedAspectValueView::Struct(fields) = value.view() else {
                return None;
            };
            let field = layout
                .program_revision_locator
                .field_path()
                .fields()
                .first()?;
            fields.get(field).cloned()
        })
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnreadable)?;
    let source = support
        .rostered_for_rendering(&source_rendering)
        .ok_or(WorthQueryBranchAdoptionPreparationDenial::ProgramActivationUnrostered)?
        .revision()
        .clone();
    let requirements = support
        .adoption_requirements(application.installed_schema(), &source, target)
        .map_err(WorthQueryBranchAdoptionPreparationDenial::Requirements)?;
    Ok((source, requirements))
}
