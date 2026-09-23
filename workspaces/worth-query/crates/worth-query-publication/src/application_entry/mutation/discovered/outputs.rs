use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
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
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationProjection,
};

use super::RootConnection;
use crate::application_entry::demand::{
    WorthQueryApplicationProgramDemandHandle, WorthQueryApplicationProgramDemandProgress,
};
use crate::application_entry::mutation::program_output_continuation::{
    ProgramOutputContinuation, ProgramOutputContinuationFactory, ProgramOutputContinuationProgress,
};
use crate::application_entry::mutation::program_output_settlement::ProgramOutputRecord;
use crate::application_entry::mutation::program_output_work::ProgramOutputTraversalWork;
use crate::application_entry::{
    WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationReadObservation,
    WorthQueryApplicationRequest, WorthQueryOutputDemandControls,
    WorthQueryRequiredOutputPreparationDenial,
};

mod settlement;
pub use settlement::{
    WorthQueryDiscoveredProgramOutputProgress, WorthQueryDiscoveredProgramOutputSettlement,
};

type Demand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationDiscoveredOutputConnection<Schema>>::Demand;
type Family<Schema, Root> =
    <Demand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Root> =
    <Family<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type Query<Schema, Root> = <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type Value<Schema, Root> =
    <<Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

struct RootNode<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    demand: Demand<Schema, Root>,
    handle: Option<
        WorthQueryApplicationProgramDemandHandle<
            'application,
            Schema,
            Program,
            Demand<Schema, Root>,
        >,
    >,
    settlement: Option<WorthQueryApplicationOutputDemandSettlement<Query<Schema, Root>>>,
    continuation: Option<Box<dyn ProgramOutputContinuation<'application, Schema> + 'application>>,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
}

pub struct WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    source_receipt:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    roots: Vec<RootNode<'application, Schema, Program, Root>>,
    superseded: Vec<Demand<Schema, Root>>,
    source_lease: Option<
        worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
    >,
    source: WorthQueryApplicationReadObservation,
    controls: WorthQueryOutputDemandControls,
    next_root: usize,
    complete: bool,
}

impl<'application, Schema, Program, Root>
    WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Demand<Schema, Root>: Clone,
{
    pub(in crate::application_entry) fn new(
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        roots: Vec<
            WorthQueryApplicationProgramDemandHandle<
                'application,
                Schema,
                Program,
                Demand<Schema, Root>,
            >,
        >,
        superseded: Vec<Demand<Schema, Root>>,
        source_lease: worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source: WorthQueryApplicationReadObservation,
        controls: WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            application,
            source_receipt,
            roots: roots
                .into_iter()
                .map(|root| RootNode {
                    demand: root.demand_clone(),
                    handle: Some(root),
                    settlement: None,
                    continuation: None,
                    outputs: Vec::new(),
                    work: ProgramOutputTraversalWork::default(),
                })
                .collect(),
            superseded,
            source_lease: Some(source_lease),
            source,
            controls,
            next_root: 0,
            complete: false,
        }
    }
}

impl<'application, Schema, Program, Root>
    WorthQueryDiscoveredProgramOutputHandle<'application, Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Demand<Schema, Root>: Clone,
    Value<Schema, Root>: WorthQueryApplicationProjection<Schema, Query<Schema, Root>> + Clone,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = Source<Schema, Root>>,
    <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <Source<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents:
        ProgramOutputContinuationFactory<'application, Schema, Program, Demand<Schema, Root>>,
{
    pub fn settled_root_observations(&self) -> Vec<&WorthQueryApplicationReadObservation> {
        self.roots
            .iter()
            .filter_map(|root| {
                root.settlement
                    .as_ref()
                    .map(|settled| settled.observation())
            })
            .collect()
    }

    pub fn settle(
        &mut self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputProgress<Query<Schema, Root>, Demand<Schema, Root>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        for _ in 0..self.controls.maximum_settlement_attempts().get() {
            let progress = self.advance(request)?;
            if matches!(
                progress,
                WorthQueryDiscoveredProgramOutputProgress::Settled(_)
            ) {
                return Ok(progress);
            }
        }
        Ok(WorthQueryDiscoveredProgramOutputProgress::Pending)
    }

    /// Advances at most one discovered root.
    ///
    /// Callers loop until settlement and may supply a freshly authorized
    /// request between calls. Outputs from retired roots remain retained by
    /// this handle while later roots advance.
    pub fn advance(
        &mut self,
        request: &WorthQueryApplicationRequest<'application, '_, '_, Schema>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputProgress<Query<Schema, Root>, Demand<Schema, Root>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        if self.complete {
            return Err(WorthQueryRequiredOutputPreparationDenial::Closed);
        }
        let mut root_superseded = false;
        if let Some(root) = self.roots.get_mut(self.next_root) {
            if let Some(handle) = &mut root.handle {
                match handle.advance(request) {
                    Err(crate::application_entry::WorthQueryApplicationOutputDemandDenial::Superseded) => {
                        self.superseded.push(root.demand.clone());
                        root.handle = None;
                        self.next_root += 1;
                        root_superseded = true;
                    }
                    Err(denial) => {
                        return Err(WorthQueryRequiredOutputPreparationDenial::Demand(denial));
                    }
                    Ok(progress) => match progress {
                    WorthQueryApplicationProgramDemandProgress::Pending => {
                        return Ok(WorthQueryDiscoveredProgramOutputProgress::Pending);
                    }
                    WorthQueryApplicationProgramDemandProgress::Settled {
                        settlement,
                        authority,
                    } => {
                        root.continuation = Some(Root::Dependents::start(
                            self.application,
                            &root.demand,
                            &settlement,
                            &authority,
                            &self.source,
                            request,
                            self.controls,
                        )?);
                        root.settlement = Some(settlement);
                        root.handle = None;
                    }
                    },
                }
            }
            if !root_superseded {
                let continuation = root
                    .continuation
                    .as_mut()
                    .expect("a settled root installs its typed continuation");
                match continuation.advance(request)? {
                    ProgramOutputContinuationProgress::Pending => {}
                    ProgramOutputContinuationProgress::Settled { outputs, work } => {
                        root.outputs = outputs;
                        root.work = work;
                        root.continuation = None;
                        self.next_root += 1;
                    }
                }
            }
        }
        if self.next_root < self.roots.len() {
            return Ok(WorthQueryDiscoveredProgramOutputProgress::Pending);
        }
        self.application.complete_program_output_source(
            &worth_query_execution::publication_boundary::program_publication_access(),
            &self.source_receipt,
        );
        self.source_lease.take();
        self.complete = true;
        let settled_count = self
            .roots
            .iter()
            .filter(|root| root.settlement.is_some())
            .count();
        let mut traversal = ProgramOutputTraversalWork::discovered(1, settled_count);
        let mut roots = Vec::with_capacity(settled_count);
        let mut outputs = Vec::new();
        for mut root in std::mem::take(&mut self.roots) {
            traversal.include(root.work);
            if let Some(settlement) = root.settlement.take() {
                roots.push((root.demand, settlement));
            }
            outputs.append(&mut root.outputs);
        }
        let work = crate::application_entry::mutation::WorthQueryApplicationProgramWork::from_all_settlements(
            traversal,
            roots.iter().map(|(_, settled)| (settled.receipt(), settled.readiness_delivery()))
                .chain(outputs.iter().map(|output| (output.receipt(), output.readiness_delivery()))),
        );
        Ok(WorthQueryDiscoveredProgramOutputProgress::Settled(
            WorthQueryDiscoveredProgramOutputSettlement::new(
                self.source.retained_clone(),
                roots,
                std::mem::take(&mut self.superseded),
                outputs,
                work,
            ),
        ))
    }
}
