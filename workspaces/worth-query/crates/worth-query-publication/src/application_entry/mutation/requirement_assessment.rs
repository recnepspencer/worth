//! Authorized read-only assessment of an installed body's own requirements.
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationInvariantProjectionAuthority, WorthQueryOperationBodyAssessmentDenial,
};
use worth_query_installation::facade::ApplicationSchema;

use super::selected_program::map_selected_program_owner_denial;
use super::WorthQueryApplicationMutationRequest;
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

pub use worth_query_execution::facade::primary_graph::WorthQueryCurrentBodyRequirementAssessment;

/// A read-only assessment may fail before or during its authorized native observation.
#[derive(Debug)]
pub enum WorthQueryCurrentBodyRequirementAssessmentDenial {
    Request(WorthQueryApplicationRequestMutationDenial),
    Native(WorthQueryOperationBodyAssessmentDenial),
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
                let findings = application
                    .assess_admitted_operation_body_requirements(
                        &phase,
                        &admission,
                        &installed,
                        projection,
                        ExecutionAllocationPolicy::SystemAllocation,
                    )
                    .map_err(WorthQueryCurrentBodyRequirementAssessmentDenial::Native)?;
                Ok(findings)
            })
            .map_err(|cause| {
                WorthQueryCurrentBodyRequirementAssessmentDenial::Request(
                    WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
                )
            })?
    }
}
