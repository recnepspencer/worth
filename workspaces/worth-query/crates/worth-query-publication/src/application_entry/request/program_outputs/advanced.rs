//! Explicit native fixed-output entrances, delegated without another phase.
use super::*;

impl<'application, 'principal, 'scope, Schema>
    crate::application_entry::advanced::WorthQueryAdvancedApplicationRequest<
        'application,
        'principal,
        'scope,
        Schema,
    >
where
    Schema: ApplicationSchema + 'static,
{
    pub fn start_program_outputs<Program, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Root::Dependents:
            crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>,
            >,
        RootDemand<Schema, Root>: Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootQuery<Schema, Root>> + Clone,
    {
        self.request
            .start_program_outputs(application, root_demand, controls)
    }
    pub fn recover_required_outputs<Program, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<Schema,
            Program,
            Root,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Root::Dependents:
            crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<Schema,
                Program,
                RootDemand<Schema, Root>,
            >,
        RootDemand<Schema, Root>: Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootQuery<Schema, Root>> + Clone,
    {
        self.request
            .recover_required_outputs(application, source_receipt, root_demand, controls)
    }
}
