use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
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

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
type RootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type Query<Schema, Demand> = <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type Value<Schema, Demand> = <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

pub struct WorthQueryApplicationProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    root: Option<
        WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            RootDemand<Schema, Root>,
        >,
    >,
    root_demand: RootDemand<Schema, Root>,
    root_settlement: Option<
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
    >,
    continuation: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    complete: bool,
}

impl<'application, Schema, Program, Root>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Root::Dependents:
        ProgramOutputContinuationFactory<'application, Schema, Program, RootDemand<Schema, Root>>,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        root: WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            RootDemand<Schema, Root>,
        >,
        root_demand: RootDemand<Schema, Root>,
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

impl<'application, Schema, Program, Root>
    WorthQueryApplicationProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Value<Schema, RootDemand<Schema, Root>>:
        WorthQueryApplicationProjection<Schema, Query<Schema, RootDemand<Schema, Root>>> + Clone,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<
            Schema,
            Binding = Source<Schema, RootDemand<Schema, Root>>,
        >,
    <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, RootDemand<Schema, Root>> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents:
        ProgramOutputContinuationFactory<'application, Schema, Program, RootDemand<Schema, Root>>,
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
            Query<Schema, RootDemand<Schema, Root>>,
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
                    self.continuation = Some(Root::Dependents::start(
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
