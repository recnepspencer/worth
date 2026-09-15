use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramRootConnection,
    ApplicationProgramRootEdges,
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
    WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
};

use crate::application_entry::demand::{
    WorthQueryApplicationProgramDemandHandle, WorthQueryApplicationProgramDemandProgress,
};

use super::program_output_continuation::{
    ProgramOutputContinuation, ProgramOutputContinuationFactory, ProgramOutputContinuationProgress,
};
use super::program_output_settlement::{
    WorthQueryApplicationProgramOutputProgress, WorthQueryApplicationProgramOutputSettlement,
};

type RootConnection<Schema, Program> = ApplicationProgramRootConnection<Schema, Program>;
type RootDemand<Schema, Program> =
    <RootConnection<Schema, Program> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type Query<Schema, Demand> = <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type Value<Schema, Demand> = <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub struct WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    RootConnection<Schema, Program>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    root: Option<
        WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            RootDemand<Schema, Program>,
        >,
    >,
    root_demand: RootDemand<Schema, Program>,
    root_settlement: Option<
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, RootDemand<Schema, Program>>,
        >,
    >,
    continuation: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    complete: bool,
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    ApplicationProgramRootEdges<Schema, Program>: ProgramOutputContinuationFactory<
        'application,
        Schema,
        Program,
        RootDemand<Schema, Program>,
    >,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        root: WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            RootDemand<Schema, Program>,
        >,
        root_demand: RootDemand<Schema, Program>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            application,
            root: Some(root),
            root_demand,
            root_settlement: None,
            continuation: None,
            controls,
            complete: false,
        }
    }
}

impl<'application, Schema, Program>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Program>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Program>: Clone,
    Value<Schema, RootDemand<Schema, Program>>:
        WorthQueryApplicationProjection<Schema, Query<Schema, RootDemand<Schema, Program>>> + Clone,
    <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, RootDemand<Schema, Program>>,
        >,
    <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, RootDemand<Schema, Program>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    ApplicationProgramRootEdges<Schema, Program>:
        ProgramOutputContinuationFactory<'application, Schema, Program, RootDemand<Schema, Program>>,
{
    pub fn notifications(
        &self,
    ) -> Result<
        worth_query_execution::facade::primary_graph::WorthQueryOutputDemandNotifications,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        self.root
            .as_ref()
            .ok_or(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed)?
            .notifications()
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)
    }

    pub fn settled_root_observation(
        &self,
    ) -> Option<&crate::application_entry::WorthQueryApplicationReadObservation> {
        self.root_settlement.as_ref().map(
            crate::application_entry::WorthQueryApplicationOutputDemandSettlement::observation,
        )
    }

    pub fn advance(
        &mut self,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            'application,
            '_,
            '_,
            Schema,
        >,
    ) -> Result<
        WorthQueryApplicationProgramOutputProgress<
            Query<Schema, RootDemand<Schema, Program>>,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    > {
        if self.complete {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Closed);
        }
        if let Some(root) = &mut self.root {
            match root
                .advance(request)
                .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?
            {
                WorthQueryApplicationProgramDemandProgress::Pending => {
                    return Ok(WorthQueryApplicationProgramOutputProgress::Pending);
                }
                WorthQueryApplicationProgramDemandProgress::Settled {
                    settlement,
                    authority,
                } => {
                    self.continuation = Some(ApplicationProgramRootEdges::<
                        Schema,
                        Program,
                    >::start(
                        self.application,
                        &self.root_demand,
                        &settlement,
                        &authority,
                        request,
                        self.controls,
                    )?);
                    self.root_settlement = Some(settlement);
                    self.root = None;
                }
            }
        }
        let continuation = self
            .continuation
            .as_mut()
            .expect("a settled root installs its typed output continuation");
        match continuation.advance(request)? {
            ProgramOutputContinuationProgress::Pending => {
                Ok(WorthQueryApplicationProgramOutputProgress::Pending)
            }
            ProgramOutputContinuationProgress::Settled { outputs, work } => {
                self.complete = true;
                self.continuation = None;
                let root = self
                    .root_settlement
                    .take()
                    .expect("output progression retains its root settlement");
                let program_work = super::program_output_work::WorthQueryApplicationProgramWork::from_settlements(
                    work,
                    (root.receipt(), root.readiness_delivery()),
                    outputs.iter().map(|output| {
                        (output.receipt(), output.readiness_delivery())
                    }),
                );
                Ok(WorthQueryApplicationProgramOutputProgress::Settled(
                    WorthQueryApplicationProgramOutputSettlement {
                        root,
                        outputs,
                        work: program_work,
                    },
                ))
            }
        }
    }
}
