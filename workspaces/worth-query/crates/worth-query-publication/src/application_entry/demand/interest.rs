use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationProgramRootConnection,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationDependentOutputConnection,
    WorthQueryApplicationOutputDemandSource, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection, WorthQueryOutputDemandAdvance,
    WorthQueryPrimaryGraphApplicationRuntime,
};

use super::request::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandRequest,
    WorthQueryOutputDemandControls,
};
use super::settlement::WorthQueryApplicationOutputDemandSettlement;

mod lifecycle;

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type SourceBinding<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> = <<SourceBinding<Schema, Demand> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootConnection<Schema, Program> = ApplicationProgramRootConnection<Schema, Program>;
type ConnectionBinding<Schema, Connection> =
    <Connection as ApplicationConnectionShape<Schema>>::Binding;

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

    pub(in crate::application_entry) fn start_for_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, application.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let basis = self.program_basis();
        let source_result = self.query_source()?;
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .admit_program_root_output(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result.into_output_demand_source(),
                maximum_work,
                maximum_retained_bytes,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
            basis,
        ))
    }

    pub(in crate::application_entry) fn start_recovery<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    ) -> Result<
        super::WorthQueryApplicationProgramDemandHandle<'application, Schema, Program, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let basis = self.program_basis();
        let source_result = self.query_source()?;
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .recover_program_root_output(
                &worth_query_execution::publication_boundary::program_publication_access(),
                source_result.into_output_demand_source(),
                maximum_work,
                maximum_retained_bytes,
                source_receipt,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
            basis,
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
        parent_basis: &crate::application_entry::WorthQueryApplicationReadObservation,
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
        let basis = self.program_basis();
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
                &parent_basis.retained,
                source_result.into_output_demand_source(),
                maximum_work,
                maximum_retained_bytes,
            )
            .map_err(WorthQueryApplicationOutputDemandDenial::Demand)?;
        Ok(super::WorthQueryApplicationProgramDemandHandle::new(
            application,
            admitted,
            self.demand,
            basis,
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

    fn program_basis(&self) -> crate::application_entry::WorthQueryApplicationReadObservation {
        crate::application_entry::WorthQueryApplicationReadObservation::new(std::sync::Arc::clone(
            self.observation
                .as_ref()
                .expect("program output admission is pinned to one exact read basis"),
        ))
    }

    pub(in crate::application_entry) fn start_performed<Program>(
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
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        let basis = self.program_basis();
        let maximum_work = self
            .controls
            .map_or(1, |controls| controls.maximum_work().get());
        let maximum_retained_bytes = self
            .controls
            .map_or(1, |controls| controls.maximum_retained_bytes().get());
        let admitted = application
            .admit_performed_program_root_output(
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
            basis,
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
            closed: false,
        }
    }
}

impl<Schema, Demand> WorthQueryApplicationOutputDemandHandle<'_, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>: 'static,
    SourceQuery<Schema, Demand>: 'static,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
{
    pub fn advance(
        &mut self,
        fresh_request: &crate::application_entry::WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryApplicationOutputDemandProgress<SourceQuery<Schema, Demand>>,
        WorthQueryApplicationOutputDemandDenial,
    > {
        if self.closed {
            return Err(WorthQueryApplicationOutputDemandDenial::Closed);
        }
        if !std::ptr::eq(self.application, fresh_request.application) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        let disclosure = fresh_request
            .query(self.demand.source_intent())
            .execute()
            .map_err(WorthQueryApplicationOutputDemandDenial::Source)?;
        let _controls = self.controls;
        let progress = self
            .application
            .advance_output_demand(
                &self.admitted,
                fresh_request.principal,
                fresh_request.scope,
                fresh_request.branch,
                disclosure.into_output_demand_disclosure(),
            )
            .map_err(|denial| {
                if denial.kind()
                    == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded
                {
                    WorthQueryApplicationOutputDemandDenial::Superseded
                } else {
                    WorthQueryApplicationOutputDemandDenial::Demand(denial)
                }
            })?;
        match progress {
            WorthQueryOutputDemandAdvance::Pending => {
                Ok(WorthQueryApplicationOutputDemandProgress::Pending)
            }
            WorthQueryOutputDemandAdvance::Settled(receipt) => {
                Ok(WorthQueryApplicationOutputDemandProgress::Settled(
                    WorthQueryApplicationOutputDemandSettlement::new(
                        receipt,
                        self.admitted.observed_source().clone(),
                    ),
                ))
            }
        }
    }
}
