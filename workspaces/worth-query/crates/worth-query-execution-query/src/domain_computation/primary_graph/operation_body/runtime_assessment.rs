//! Runtime-owned body requirement assessment and candidate write application.

use worth_query_declaration::facade::{
    application_operation::{ApplicationMutationBinding, ApplicationMutationScopeBinding},
    application_schema::{ApplicationOperationBodyProgram, ApplicationSchema},
};
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_query_installation::facade::WorthQueryInstalledApplicationMutationBinding;

use crate::domain_computation::primary_graph::{
    CandidateWriter, HandlerExecutionDenial, HandlerResult, WorthQueryAdmittedApplicationOperation,
    WorthQueryAdvancementPhase, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial,
    WorthQueryOperationBodyDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

use super::{PreparedBodyPathWrites, PreparedBodyRetirements};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Inspects the installed body's requirements using an already authorized
    /// operation. The result retains its owned bytes; no snapshot or mutation target escapes.
    pub fn assess_admitted_operation_body_requirements<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        admission: &WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        installed: &WorthQueryInstalledApplicationMutationBinding<Schema, Binding>,
        projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
        allocation: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryCurrentBodyRequirementAssessment, WorthQueryOperationBodyAssessmentDenial>
    where
        Binding: ApplicationMutationBinding<Schema>,
    {
        let execution = phase
            .execution_for(&self.runtime.product_runtime)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Phase)?;
        let request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Phase)?;
        self.validate_projected_source_facts(&request, admission)
            .map_err(WorthQueryOperationBodyAssessmentDenial::Source)?;
        let projected = projection
            .project_admitted_operation(
                phase,
                admission,
                |reader, identity| reader.inspect_body_requirements(identity, installed, execution),
                allocation,
            )
            .map_err(WorthQueryOperationBodyAssessmentDenial::Projection)?;
        let (findings, snapshot, _) = projected.into_parts();
        drop(snapshot);
        findings.map_err(WorthQueryOperationBodyAssessmentDenial::Observation)
    }
}

pub fn apply_retirements<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    prepared: PreparedBodyRetirements<S>,
    body: &ApplicationOperationBodyProgram,
    writer: &mut CandidateWriter<'_, S, B>,
) -> HandlerResult<(), WorthQueryOperationBodyDenial> {
    for batch in prepared.batches {
        let rule = &body.retirements()[batch.rule];
        for (owner, selected) in batch.targets {
            if let Err(cause) = writer.body_retire_owned_selection(&owner, &selected, rule) {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause));
            }
        }
    }
    HandlerResult::Completed(())
}

pub fn apply_path_writes<S: ApplicationSchema, B: ApplicationMutationBinding<S>>(
    prepared: PreparedBodyPathWrites<S>,
    body: &ApplicationOperationBodyProgram,
    values: Option<&super::value::PreparedBodyValues>,
    writer: &mut CandidateWriter<'_, S, B>,
) -> Result<(), HandlerExecutionDenial> {
    for (step, target) in body.path_value_assignments().iter().zip(prepared.targets) {
        let target = writer
            .projected_entity(&target)
            .map_err(HandlerExecutionDenial::new)?;
        writer
            .body_field(
                &target,
                &step.target.field,
                values
                    .expect("installed computed path value")
                    .value(&step.value),
            )
            .map_err(HandlerExecutionDenial::new)?;
    }
    Ok(())
}
