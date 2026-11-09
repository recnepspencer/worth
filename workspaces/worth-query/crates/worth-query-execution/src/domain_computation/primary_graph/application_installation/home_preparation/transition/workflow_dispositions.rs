//! Fresh Home dispositions use the ordinary workflow inventory and staging owner.
use super::*;
use crate::domain_computation::primary_graph::workflow::adoption::{
    inventory_workflows_from_records, prepare_workflow_dispositions,
    workflow_record_population_is_empty, WorkflowAdoptionInventoryRequest,
    WorkflowAdoptionReadDenial, WorkflowDispositionPreparationDenial,
    WorkflowVocabularyCoverageRegistry, WorthQueryWorkflowDispositions,
};
use worth_query_declaration::facade::application_program::ApplicationProgramRevision;
use worth_query_installation::facade::{
    WorthQueryProgramAdoptionRequirements, WorthQueryProgramAdoptionRequirementsDenial,
};
use worth_relational::facade::transactions::WorkerIntentBatch;

#[derive(Debug)]
pub enum QueryHomeWorkflowDispositionDenial {
    Requirements(WorthQueryProgramAdoptionRequirementsDenial),
    Inventory(WorkflowAdoptionReadDenial),
    Required,
    Rejected,
}

impl<Schema: ApplicationSchemaComposition, Program>
    PreparedHomeProgramTransitionTarget<Schema, Program>
{
    /// Stage only framework-owned workflow effects; the Host appends them after
    /// the authored-body fence and before the canonical plan digest.
    pub fn prepare_home_workflow_dispositions<
        Before: ApplicationSchemaComposition,
        BeforeProgram,
    >(
        &self,
        predecessor: &ReadmittedExistingHomeNativeMeaning<Before, BeforeProgram>,
        source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
        choices: Option<&WorthQueryWorkflowDispositions>,
        maximum_work_units: usize,
    ) -> Result<Option<WorkerIntentBatch>, QueryHomeWorkflowDispositionDenial> {
        let target = &self._pending._state;
        let (source_schema, source_roster) = predecessor.predecessor_program_support();
        let source_revision = predecessor.predecessor_activation().0;
        let target_revision = &target.support.initial_revision;
        let mut compare = || {
            target
                .support
                .roster
                .adoption_requirements_from_installed_predecessor(
                    &target.installed,
                    source_roster,
                    source_schema,
                    source_revision,
                    target_revision,
                    maximum_work_units,
                )
        };
        prepare(
            source,
            source_revision,
            target_revision,
            choices,
            maximum_work_units,
            &mut compare,
        )
    }
}

fn prepare(
    source: &WorthQueryHomeProgramTransitionReadSource<'_, '_>,
    before: &ApplicationProgramRevision,
    target: &ApplicationProgramRevision,
    choices: Option<&WorthQueryWorkflowDispositions>,
    maximum_work_units: usize,
    compare: &mut dyn FnMut() -> Result<
        WorthQueryProgramAdoptionRequirements,
        WorthQueryProgramAdoptionRequirementsDenial,
    >,
) -> Result<Option<WorkerIntentBatch>, QueryHomeWorkflowDispositionDenial> {
    use QueryHomeWorkflowDispositionDenial as Denial;
    let execution = source.request();
    let layout = source.layout.workflow();
    let (empty, selection_work) =
        workflow_record_population_is_empty(&execution, &source.view, layout, maximum_work_units)
            .map_err(Denial::Inventory)?;
    if empty {
        if let Some(choices) = choices {
            choices
                .admit_empty_population()
                .map_err(|()| Denial::Rejected)?;
        }
        return Ok(None);
    }
    let requirements = compare().map_err(Denial::Requirements)?;
    // Home has installed no executable workflow vocabulary. That absence cannot
    // license carriage; retirement/cancellation needs no executable target.
    let coverage = WorkflowVocabularyCoverageRegistry::default();
    let request = WorkflowAdoptionInventoryRequest {
        layout,
        branch_occurrence: source.branch.occurrence_ordinal(),
        coverage: &coverage,
        requirements: &requirements,
        source: before,
        target,
        maximum_work_units: maximum_work_units.saturating_sub(selection_work),
    };
    let inventory = inventory_workflows_from_records(&execution, source.view.clone(), &request)
        .map_err(Denial::Inventory)?;
    let intents = prepare_workflow_dispositions(
        layout,
        &inventory,
        choices,
        source.branch,
        target,
        &source.actor,
    )
    .map_err(|cause| match cause {
        WorkflowDispositionPreparationDenial::Required => Denial::Required,
        WorkflowDispositionPreparationDenial::Rejected => Denial::Rejected,
    })?;
    if intents.is_empty() {
        return Ok(None);
    }
    let mut batch = WorkerIntentBatch::new("home-workflow-dispositions");
    for intent in intents {
        batch = batch.push(intent);
    }
    Ok(Some(batch))
}
