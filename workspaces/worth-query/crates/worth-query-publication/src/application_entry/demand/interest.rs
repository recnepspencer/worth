mod advance;
mod types;
mod workflow;

use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramIdentity, ApplicationProgramRevision,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationOutputDemandSource, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandAdvance,
    WorthQueryOutputDemandNotifications, WorthQueryPrimaryGraphApplicationRuntime,
};

use super::request::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandRequest,
    WorthQueryOutputDemandControls,
};
use super::settlement::WorthQueryApplicationOutputDemandSettlement;

use types::{ConnectionBinding, Family, RootConnection, SourceBinding, SourceQuery, SourceValue};

pub enum WorthQueryApplicationOutputDemandProgress<Query> {
    Pending,
    Settled(WorthQueryApplicationOutputDemandSettlement<Query>),
}

pub struct WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    application: &'application WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    demand: Demand,
    controls: WorthQueryOutputDemandControls,
    selected_program: Option<(ApplicationProgramIdentity, ApplicationProgramRevision)>,
    closed: bool,
}

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
    pub fn start(
        self,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let source_result = self.query_source()?;
        self.start_ordinary(source_result.into_output_demand_source())
    }

    /// Explicitly binds a direct producer demand to the program selected by this host.
    /// The exact revision is checked again at the producer's commit boundary.
    pub fn start_in_program<Program, Root>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, program.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        if !program.contains_output_root::<Root>() {
            return Err(WorthQueryApplicationOutputDemandDenial::ProgramOutputUndeclared);
        }
        self.start_with_program_selection(program)
    }

    /// Advanced direct demand for one output connection declared by the selected program.
    pub fn start_dependent_in_program<Program, Connection>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>:
            WorthQueryApplicationDependentOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, program.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        if !program.contains_output_connection::<Connection>(
            &worth_query_execution::publication_boundary::program_publication_access(),
        ) {
            return Err(WorthQueryApplicationOutputDemandDenial::ProgramOutputUndeclared);
        }
        self.start_with_program_selection(program)
    }

    fn start_with_program_selection<Program>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let selection = (
            program.installed_program().identity().clone(),
            program.installed_program().revision().clone(),
        );
        let mut handle = self.start()?;
        handle.selected_program = Some(selection);
        Ok(handle)
    }

    pub(in crate::application_entry) fn start_for_program<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        (
            super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
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
        let source_result = self.query_source()?;
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .admit_program_root_output::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result.into_output_demand_source(),
                maximum_work,
                maximum_retained_bytes,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok((
            super::WorthQueryApplicationProgramDemandHandle::new(
                application,
                admitted,
                self.demand,
            ),
            observation,
        ))
    }

    pub(in crate::application_entry) fn start_recovery<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        (
            super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
            std::sync::Arc<
                worth_query_execution::facade::primary_graph::WorthQueryApplicationReadObservation,
            >,
        ),
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let retained = application
            .recover_prepared_program_root_source::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_receipt,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        let (observation, prepared) = retained;
        let source_result = {
            let request = crate::application_entry::WorthQueryApplicationRequest {
                application: self.application,
                principal: self.principal,
                scope: self.scope,
                branch: self.branch,
            };
            request
                .at(
                    &crate::application_entry::WorthQueryApplicationReadObservation::new(
                        std::sync::Arc::clone(&observation),
                    ),
                )
                .query(self.demand.source_intent())
                .execute()
                .map_err(WorthQueryApplicationOutputDemandDenial::Source)?
        };
        let source_result = source_result.into_output_demand_source();
        let current = self.query_source()?.into_output_demand_source();
        application
            .validate_recovered_program_root_currentness::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &prepared,
                &source_result,
                &current,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        application
            .ensure_recovered_program_root_source_bound::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                &prepared,
                &source_result,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .recover_program_root_output::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result,
                maximum_work,
                maximum_retained_bytes,
                source_receipt,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok((
            super::WorthQueryApplicationProgramDemandHandle::new(
                application,
                admitted,
                self.demand,
            ),
            observation,
        ))
    }

    pub(in crate::application_entry) fn start_dependent<Program, ParentDemand, Connection>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent: &worth_query_execution::facade::application_installation::WorthQuerySettledProgramOutput<
            Schema,
            Program,
            ParentDemand,
        >,
        basis: &crate::application_entry::WorthQueryApplicationReadObservation,
        minimum_observation: &crate::application_entry::WorthQueryApplicationReadObservation,
    ) -> Result<
        super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        ParentDemand: WorthQueryApplicationOutputDemand<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>: WorthQueryApplicationDependentOutputConnection<
            Schema,
            RootDemand = ParentDemand,
            Demand = Demand,
        >,
    {
        let source_result = self.query_source()?;
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .admit_program_dependent_output::<ParentDemand, Connection>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                parent,
                &basis.retained,
                &minimum_observation.retained,
                source_result.into_output_demand_source(),
                maximum_work,
                maximum_retained_bytes,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
        ))
    }

    fn query_source(
        &self,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let request = crate::application_entry::WorthQueryApplicationRequest {
            application: self.application,
            principal: self.principal,
            scope: self.scope,
            branch: self.branch,
        };
        if let Some(observation) = &self.observation {
            request
                .at(
                    &crate::application_entry::WorthQueryApplicationReadObservation::new(
                        std::sync::Arc::clone(observation),
                    ),
                )
                .query(self.demand.source_intent())
                .execute()
        } else {
            request.query(self.demand.source_intent()).execute()
        }
        .map_err(WorthQueryApplicationOutputDemandDenial::Source)
    }

    pub(in crate::application_entry) fn start_performed<Program, Root>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        prepared: &worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source_result: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
    ) -> Result<
        super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>
            + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .admit_performed_program_root_output::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result,
                maximum_work,
                maximum_retained_bytes,
                prepared,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
        ))
    }

    fn start_ordinary(
        self,
        source_result: WorthQueryApplicationOutputDemandSource<
            SourceQuery<Schema, Demand>,
            SourceValue<Schema, Demand>,
        >,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = self
            .application
            .admit_output_demand::<Family<Schema, Demand>>(
                source_result,
                maximum_work,
                maximum_retained_bytes,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(self.handle(admitted))
    }

    fn handle(
        self,
        admitted: WorthQueryAdmittedOutputDemand<Schema, Family<Schema, Demand>>,
    ) -> WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand> {
        WorthQueryApplicationOutputDemandHandle {
            application: self.application,
            admitted,
            demand: self.demand,
            controls: self.controls.unwrap_or_else(|| {
                WorthQueryOutputDemandControls::new(
                    std::num::NonZeroUsize::new(1).unwrap(),
                    std::num::NonZeroUsize::new(1).unwrap(),
                )
            }),
            selected_program: None,
            closed: false,
        }
    }
}

impl<Schema, Demand> Drop for WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
{
    fn drop(&mut self) {
        if !self.closed {
            self.admitted.close();
            self.closed = true;
        }
    }
}
