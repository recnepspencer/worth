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
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQuerySettledProgramOutput,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDiscoveredOutputConnection, WorthQueryApplicationProjection,
};

use super::DiscoveredRootStartKind;
use super::{Discovery, RootConnection};
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

mod admission;
mod settlement;
use admission::{DiscoveryBinding, DiscoveryQuery, DiscoveryValue, RootAdmission};
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

struct RootNode<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    demand: Demand<Schema, Root>,
    handle: Option<WorthQueryApplicationProgramDemandHandle<Schema, Program, Demand<Schema, Root>>>,
    settlement: Option<WorthQueryApplicationOutputDemandSettlement<Query<Schema, Root>>>,
    pending_authority: Option<
        std::sync::Arc<WorthQuerySettledProgramOutput<Schema, Program, Demand<Schema, Root>>>,
    >,
    continuation: Option<Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>>,
    outputs: Vec<ProgramOutputRecord>,
    work: ProgramOutputTraversalWork,
}

/// Drives a mutation's discovered program outputs to settlement. `advance` takes every
/// discovered root as far as one call can.
pub struct WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
{
    source_receipt:
        worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
    roots: Vec<RootNode<Schema, Program, Root>>,
    admission: RootAdmission<Schema, Root>,
    superseded: Vec<Demand<Schema, Root>>,
    source_lease: Option<
        worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
    >,
    source: WorthQueryApplicationReadObservation,
    controls: WorthQueryOutputDemandControls,
    next_root: usize,
    complete: bool,
}

impl<Schema, Program, Root> WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationDiscoveredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationDiscoveredOutputConnection<Schema>,
    Demand<Schema, Root>: Clone,
{
    pub(super) fn new(
        source_receipt: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        discovery: Discovery<Schema, Root>,
        source_lease: worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source: WorthQueryApplicationReadObservation,
        controls: WorthQueryOutputDemandControls,
        start_kind: DiscoveredRootStartKind,
    ) -> Self {
        Self {
            source_receipt,
            roots: Vec::new(),
            admission: RootAdmission::new(discovery, start_kind),
            superseded: Vec::new(),
            source_lease: Some(source_lease),
            source,
            controls,
            next_root: 0,
            complete: false,
        }
    }
}

impl<Schema, Program, Root> WorthQueryDiscoveredProgramOutputHandle<Schema, Program, Root>
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
    DiscoveryValue<Schema, Root>:
        WorthQueryApplicationProjection<Schema, DiscoveryQuery<Schema, Root>> + Clone,
    <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <DiscoveryBinding<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, Demand<Schema, Root>>,
{
    /// The first native admission refusal remains available after a later retry.
    /// It is diagnostic evidence, not continuation or completion authority.
    pub fn first_admission_denial(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial> {
        self.admission.first_denial()
    }

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
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputProgress<Query<Schema, Root>, Demand<Schema, Root>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        for _ in 0..self
            .controls
            .resolve(application.runtime().output_demand_resource_profile())
            .settlement_attempts()
        {
            let progress = self.advance(application, request)?;
            if matches!(
                progress,
                WorthQueryDiscoveredProgramOutputProgress::Settled(_)
            ) {
                return Ok(progress);
            }
        }
        Ok(WorthQueryDiscoveredProgramOutputProgress::Pending)
    }

    /// Advances the discovered roots in order, each with its dependents.
    ///
    /// `Pending` means a root waits on work outside this call. Callers may
    /// supply a freshly authorized request for the next call. Outputs from
    /// retired roots remain retained by this handle while later roots advance.
    pub fn advance(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<
        WorthQueryDiscoveredProgramOutputProgress<Query<Schema, Root>, Demand<Schema, Root>>,
        WorthQueryRequiredOutputPreparationDenial,
    > {
        if self.complete {
            return Err(WorthQueryRequiredOutputPreparationDenial::Closed);
        }
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(WorthQueryRequiredOutputPreparationDenial::ForeignProgram);
        }
        let prepared = self
            .source_lease
            .as_ref()
            .expect("an unfinished continuation owns its source");
        application
            .validate_discovered_program_source::<Root>(
                &worth_query_execution::publication_boundary::program_publication_access(),
                prepared,
                &self.source_receipt,
                &self.source.retained,
                request.principal,
                request.scope,
                request.branch,
            )
            .map_err(WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
        self.admission.admit(
            application,
            request,
            &self.source_receipt,
            prepared,
            &self.source,
            self.controls,
            &mut self.roots,
            &mut self.superseded,
        )?;
        while let Some(root) = self.roots.get_mut(self.next_root) {
            if let Some(handle) = &mut root.handle {
                match handle.advance(application, request) {
                    Err(crate::application_entry::WorthQueryApplicationOutputDemandDenial::Superseded) => {
                        self.superseded.push(root.demand.clone());
                        root.handle = None;
                        self.next_root += 1;
                        continue;
                    }
                    Err(denial) => {
                        return Err(WorthQueryRequiredOutputPreparationDenial::Demand(denial));
                    }
                    Ok(WorthQueryApplicationProgramDemandProgress::Pending) => {
                        return Ok(WorthQueryDiscoveredProgramOutputProgress::Pending);
                    }
                    Ok(WorthQueryApplicationProgramDemandProgress::Settled {
                        settlement,
                        authority,
                    }) => {
                        root.settlement = Some(settlement);
                        root.pending_authority = Some(std::sync::Arc::new(authority));
                        root.handle = None;
                    }
                }
            }
            // A settled root starts its dependents in the same call. A start
            // that is refused keeps the authority for the next call.
            if let Some(authority) = root.pending_authority.as_ref() {
                let settlement = root
                    .settlement
                    .as_ref()
                    .expect("a settled root retains its output settlement");
                root.continuation = Some(Root::Dependents::start(
                    application,
                    &root.demand,
                    settlement,
                    authority,
                    &self.source,
                    request,
                    self.controls,
                )?);
                root.pending_authority = None;
            }
            let continuation = root
                .continuation
                .as_mut()
                .expect("a settled root installs its typed continuation");
            match continuation.advance(application, request)? {
                ProgramOutputContinuationProgress::Pending => {
                    return Ok(WorthQueryDiscoveredProgramOutputProgress::Pending);
                }
                ProgramOutputContinuationProgress::Settled { outputs, work } => {
                    root.outputs = outputs;
                    root.work = work;
                    root.continuation = None;
                    self.next_root += 1;
                }
            }
        }
        application.complete_program_output_source(
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
            roots.iter().map(|(_, settled)| (
                settled.application_commit_receipt(),
                settled.readiness_delivery(),
                settled.checkpoint_readmission_work_units(),
                settled.checkpoint_readmission_work_bound(),
                settled.checkpoint_readmission_charged_preparation_bytes(),
            ))
                .chain(outputs.iter().map(|output| (
                    output.receipt(), output.readiness_delivery(),
                    output.checkpoint_readmission_work_units(),
                    output.checkpoint_readmission_work_bound(),
                    output.checkpoint_readmission_charged_preparation_bytes()))),
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
