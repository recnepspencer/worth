//! One ordinary program-root admission, with optional recording-source correspondence.
use super::*;
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
    pub(in crate::application_entry) fn start_for_program<Program, Root>(
        self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        (
            super::super::WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>,
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        self.start_program_root::<Program, Root>(phase, application, None)
    }

    pub(in crate::application_entry) fn start_for_recording<Program, Root>(
        self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        origin: &worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin,
        index: usize,
    ) -> Result<
        (
            super::super::WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>,
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        self.start_program_root::<Program, Root>(phase, application, Some((origin, index)))
    }

    fn start_program_root<Program, Root>(
        self,
        phase: &AdvancementPhase<'_>,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        recording: Option<(&worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin, usize)>,
    ) -> Result<
        (
            super::super::WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand>,
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, application.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let observation = self
            .observation
            .as_ref()
            .cloned()
            .ok_or(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch)?;
        let source_result = self.query_source(phase)?.into_output_demand_source();
        if let Some((origin, index)) = recording {
            origin
                .require_recording_root_source::<Schema, Program, Root>(
                    phase,
                    &worth_query_execution::publication_boundary::program_publication_access(),
                    application,
                    index,
                    &observation,
                    &source_result,
                )
                .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        }
        let limits = self.resolved_limits();
        let admitted = application
            .admit_program_root_output::<Root>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result,
                limits,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok((
            super::super::WorthQueryApplicationProgramDemandHandle::new(
                admitted,
                self.demand,
                None,
            ),
            observation,
        ))
    }
}
