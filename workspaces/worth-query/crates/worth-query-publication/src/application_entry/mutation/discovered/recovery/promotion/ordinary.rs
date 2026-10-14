//! Restricted discovered delegation until its owning adapter lands.
use super::*;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    crate::application_entry::WorthQueryPendingApplicationLane<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub fn promote_recovered_discovered_outputs<Program, Root>(
        self,
        recovery: WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        WorthQueryRecoveredDiscoveredOutputs<Schema, Program, Root>,
        (WorthQueryApplicationRecoveryRequestDenial, WorthQueryUnpublishedDiscoveredApplicationMutation<Schema, Intent, Program, Root>),
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Outputs: ApplicationProgramOutputsShape<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema, Source=Intent::Binding>,
        Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
        RootDemand<Schema, Root>: Clone,
        DiscoveryValue<Schema, Root>: WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
        <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
        RootSourceValue<Schema, Root>: WorthQueryApplicationProjection<Schema, RootSourceQuery<Schema, Root>> + Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding=RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<Schema, <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    {
        self.native
            .promote_recovered_discovered_outputs(recovery, application, controls)
    }
}
