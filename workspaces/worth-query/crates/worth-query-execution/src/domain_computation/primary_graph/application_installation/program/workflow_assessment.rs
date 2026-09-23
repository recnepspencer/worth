use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;

use super::WorthQueryWorkflowApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationOutputDemandSource,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryProducerOutputFamily,
};

type SourceBinding<Schema, Family> = <Family as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Family> = <SourceBinding<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Family> = <<SourceBinding<Schema, Family> as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::ResultBinding as worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding>::Value;

impl<Schema, Spec, Program> WorthQueryWorkflowApplicationRuntime<Schema, Spec, Program>
where
    Schema: ApplicationSchema + 'static,
    Spec: ApplicationWorkflowSpec<Schema = Schema>,
    Program: ApplicationProgramDefinition<Schema>,
{
    pub fn admit_workflow_assessment_output<Family>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        source: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Family>,
            SourceValue<Schema, Family>,
        >,
        maximum_work: usize,
        maximum_retained_bytes: usize,
    ) -> Result<WorthQueryAdmittedOutputDemand<Schema, Family>, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        self.program_runtime()
            .runtime()
            .admit_output_demand::<Family>(source, maximum_work, maximum_retained_bytes)
    }

    pub fn advance_workflow_assessment_output<Family>(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        demand: &mut WorthQueryAdmittedOutputDemand<Schema, Family>,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request_scope: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
        delivery_branch: crate::basis::WorthQueryProductBranch,
        disclosure: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Family>,
            SourceValue<Schema, Family>,
        >,
    ) -> Result<WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
        SourceValue<Schema, Family>: 'static,
        SourceQuery<Schema, Family>: 'static,
    {
        self.program_runtime()
            .runtime()
            .advance_program_output_demand(
                demand,
                principal,
                request_scope,
                delivery_branch,
                disclosure,
            )
    }
}
