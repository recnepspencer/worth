mod unavailable;
pub use unavailable::WorthQueryApplicationProgramOutputUnavailable;
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
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQuerySettledProgramOutput,
};
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

/// Drives a mutation's required program outputs to settlement.
pub struct WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    source_preparation: Option<
        worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
    >,
    source_receipt:
        Option<worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt>,
    source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
    root:
        Option<WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>>,
    root_demand: RootDemand<Schema, Root>,
    root_settlement: Option<
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
    >,
    pending_root_authority: Option<
        std::sync::Arc<WorthQuerySettledProgramOutput<Schema, Program, RootDemand<Schema, Root>>>,
    >,
    continuation: Option<Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    complete: bool,
}

impl<Schema, Program, Root> WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
{
    pub(in crate::application_entry) fn new(
        source_receipt: worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        preparation: worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
        root: WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            source_preparation: Some(preparation),
            source_receipt: Some(source_receipt),
            source_observation,
            root: Some(root),
            root_demand,
            root_settlement: None,
            pending_root_authority: None,
            continuation: None,
            controls,
            complete: false,
        }
    }

    pub(in crate::application_entry) fn new_initial(
        source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
        root: WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            source_preparation: None,
            source_receipt: None,
            source_observation,
            root: Some(root),
            root_demand,
            root_settlement: None,
            pending_root_authority: None,
            continuation: None,
            controls,
            complete: false,
        }
    }
}

impl<Schema, Program, Root>
    WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
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
        ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
{
    /// Exact original read basis held by this continuation, distinct from later outputs.
    pub fn source_observation(&self) -> &crate::application_entry::WorthQueryApplicationReadObservation { &self.source_observation }

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

    pub fn settle(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            '_,
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
        for _ in 0..self.controls.resolve(application.runtime().output_demand_resource_profile()).settlement_attempts() {
            let progress = self.advance(application, request)?;
            if matches!(progress, WorthQueryApplicationProgramOutputProgress::Settled(_)) {
                return Ok(progress);
            }
        }
        Ok(WorthQueryApplicationProgramOutputProgress::Pending)
    }

    pub fn advance(
        &mut self,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &crate::application_entry::WorthQueryApplicationRequest<
            '_,
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
        if !std::ptr::eq(application.runtime(), request.application) {
            return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(crate::application_entry::WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch));
        }
        let access = worth_query_execution::publication_boundary::program_publication_access();
        match (&self.source_receipt, &self.source_preparation) {
            (Some(receipt), Some(prepared)) => application.validate_required_program_source::<Root>(
                &access, prepared, receipt, &self.source_observation.retained, request.principal, request.scope, request.branch),
            (None, None) => application.validate_initial_program_source::<Root>(&access, &self.source_observation.retained, request.branch),
            _ => return Err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery),
        }.map_err(|denial| crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
            if denial.kind() == worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded {
                crate::application_entry::WorthQueryApplicationOutputDemandDenial::Superseded
            } else { crate::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial) }))?;
        if let Some(root) = &mut self.root {
            match root
                .advance(application, request)
                .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?
            {
                WorthQueryApplicationProgramDemandProgress::Pending => {
                    return Ok(WorthQueryApplicationProgramOutputProgress::Pending);
                }
                WorthQueryApplicationProgramDemandProgress::Settled {
                    settlement,
                    authority,
                } => {
                    self.root_settlement = Some(settlement);
                    self.pending_root_authority = Some(std::sync::Arc::new(authority));
                    self.root = None;
                }
            }
        }
        // A settled root starts its dependents in the same call. A start
        // that is refused keeps the authority for the next call.
        if let Some(authority) = self.pending_root_authority.as_ref() {
            let settlement = self
                .root_settlement
                .as_ref()
                .expect("a settled root retains its output settlement");
            self.continuation = Some(Root::Dependents::start(
                application,
                &self.root_demand,
                settlement,
                authority,
                &self.source_observation,
                request,
                self.controls,
            )?);
            self.pending_root_authority = None;
        }
        let continuation = self
            .continuation
            .as_mut()
            .expect("a settled root installs its typed output continuation");
        match continuation.advance(application, request)? {
            ProgramOutputContinuationProgress::Pending => {
                Ok(WorthQueryApplicationProgramOutputProgress::Pending)
            }
            ProgramOutputContinuationProgress::Settled { outputs, work } => {
                if let Some(source_receipt) = &self.source_receipt {
                    application.complete_program_output_source(
                        &worth_query_execution::publication_boundary::program_publication_access(),
                        source_receipt,
                    );
                }
                self.complete = true;
                self.source_preparation = None;
                self.continuation = None;
                let root = self
                    .root_settlement
                    .take()
                    .expect("output progression retains its root settlement");
                let program_work = super::program_output_work::WorthQueryApplicationProgramWork::from_settlements(
                    work,
                    (
                        root.application_commit_receipt(),
                        root.readiness_delivery(),
                        root.checkpoint_readmission_work_units(),
                        root.checkpoint_readmission_work_bound(),
                        root.checkpoint_readmission_charged_preparation_bytes(),
                    ),
                    outputs.iter().map(|output| {
                        (output.receipt(), output.readiness_delivery(),
                            output.checkpoint_readmission_work_units(),
                            output.checkpoint_readmission_work_bound(),
                            output.checkpoint_readmission_charged_preparation_bytes())
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
