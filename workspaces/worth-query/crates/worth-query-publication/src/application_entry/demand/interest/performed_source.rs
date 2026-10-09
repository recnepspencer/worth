use super::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;

impl<'application, 'principal, 'scope, Schema, Demand>
    WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    pub(in crate::application_entry) fn start_performed<Program, Root>(
        self,
        _phase: &AdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        prepared: &worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source_result: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
    ) -> Result<
        super::super::WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            Demand,
        >,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let limits = self.resolved_limits();
        let admitted = application
            .admit_performed_program_root_output::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result,
                limits,
                prepared,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
            None,
        ))
    }
}
