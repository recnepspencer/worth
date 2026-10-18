//! Explicit current native registered-custody recovery; ordinary operations retain their descriptor.
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
    pub fn recover_discovered_required_outputs<Program, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        receipt: &WorthQueryApplicationCommitReceipt,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>,
        WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>,
        >,
        RootDemand<Schema, Root>: Clone,
        DiscoveryValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
        <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootSourceValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
    {
        self.request
            .recover_discovered_required_outputs(application, receipt, controls)
    }
}
