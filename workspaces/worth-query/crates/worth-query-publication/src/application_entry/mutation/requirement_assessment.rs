//! Authorized read-only assessment of an installed body's own requirements.
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
};
use worth_query_execution::facade::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryOperationBodyRequirementFinding, WorthQueryOperationProjectionDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::selected_program::map_selected_program_owner_denial;
use super::WorthQueryApplicationMutationRequest;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

/// A descriptive observation, never an admission permit or retained snapshot.
pub struct WorthQueryCurrentBodyRequirementAssessment {
    findings: Vec<WorthQueryOperationBodyRequirementFinding>,
}

impl WorthQueryCurrentBodyRequirementAssessment {
    pub fn findings(&self) -> &[WorthQueryOperationBodyRequirementFinding] {
        &self.findings
    }
}

/// A read-only assessment may fail before or during its authorized native observation.
#[derive(Debug)]
pub enum WorthQueryCurrentBodyRequirementAssessmentDenial {
    Request(WorthQueryApplicationRequestMutationDenial),
    Projection(WorthQueryOperationProjectionDenial),
    Observation(HandlerExecutionDenial),
}

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequest<'_, '_, '_, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn assess_current_body_requirements<Program>(
        mut self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        projection: &WorthQueryApplicationInvariantProjectionAuthority<Schema>,
    ) -> Result<
        WorthQueryCurrentBodyRequirementAssessment,
        WorthQueryCurrentBodyRequirementAssessmentDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
        let application = self.application;
        let scope = self.scope.clone();
        application
            .with_application_advancement(&scope, |phase| {
                if !std::ptr::eq(program.runtime(), application) {
                    return Err(WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch,
                    ));
                }
                let selected =
                    application
                        .on_branch(self.branch)
                        .select(&phase)
                        .map_err(|denial| {
                            WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                                WorthQueryApplicationRequestMutationDenial::ProductSelection(
                                    denial,
                                ),
                            )
                        })?;
                let owner = program
                    .selected_program_owner(&selected)
                    .map_err(|denial| {
                        WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                            map_selected_program_owner_denial(denial),
                        )
                    })?;
                if !owner.contains_action::<Intent::Binding>() {
                    return Err(WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
                    ));
                }
                let installed = application
                    .installed_schema()
                    .installed_mutation_binding::<Intent::Binding>()
                    .map_err(|denial| {
                        WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                            WorthQueryApplicationRequestMutationDenial::BindingInstallation(denial),
                        )
                    })?;
                let admission = super::authorization::admit_for_assessment(&phase, &mut self)
                    .map_err(WorthQueryCurrentBodyRequirementAssessmentDenial::Request)?;
                let projected = projection
                    .project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, identity| reader.inspect_body_requirements(identity, &installed),
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .map_err(WorthQueryCurrentBodyRequirementAssessmentDenial::Projection)?;
                let (findings, snapshot, _) = projected.into_parts();
                drop(snapshot);
                Ok(WorthQueryCurrentBodyRequirementAssessment {
                    findings: findings
                        .map_err(WorthQueryCurrentBodyRequirementAssessmentDenial::Observation)?,
                })
            })
            .map_err(|cause| {
                WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?
    }
}
