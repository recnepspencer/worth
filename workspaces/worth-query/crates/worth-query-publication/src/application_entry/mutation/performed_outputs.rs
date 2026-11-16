mod advancement;
mod fulfillment;
mod history;
mod read_close;
mod source;
mod source_binding;
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
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
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
    ProgramOutputParentAuthority,
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

/// Drives one installed fixed output tree to settlement.
pub struct WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
{
    recording_recipes: Option<(std::sync::Arc<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin>, usize)>,
    source_binding:
        Option<source_binding::RequiredSourceBinding<Schema, Program, RootDemand<Schema, Root>>>,
    source_preparation: Option<
        worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
    >,
    source_receipt:
        Option<worth_query_execution::facade::primary_graph::WorthQueryOutputSourceEvidence>,
    source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
    root:
        Option<WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>>,
    root_demand: RootDemand<Schema, Root>,
    root_settlement: Option<
        crate::application_entry::WorthQueryApplicationOutputDemandSettlement<
            Query<Schema, RootDemand<Schema, Root>>,
        >,
    >,
    pending_root_authority:
        Option<ProgramOutputParentAuthority<Schema, Program, RootDemand<Schema, Root>>>,
    pending_root_fulfillment:
        Option<worth_query_execution::facade::primary_graph::WorthQueryFulfilledRequiredRoot>,
    continuation: Option<Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>>,
    controls: crate::application_entry::WorthQueryOutputDemandControls,
    complete: bool,
    pending_history_delivery: Option<
        WorthQueryApplicationProgramOutputSettlement<Query<Schema, RootDemand<Schema, Root>>>,
    >,
}

impl<Schema, Program, Root> WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>
where
    Schema: ApplicationSchema + 'static,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>
        + worth_query_declaration::facade::application_program::ApplicationFixedOutputRoot,
    RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    RootDemand<Schema, Root>: Clone,
    Root::Dependents: ProgramOutputContinuationFactory<Schema, Program, RootDemand<Schema, Root>>,
{
    pub(in crate::application_entry) fn record_history_discovery(
        mut self,
        origin: std::sync::Arc<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRecordingOutputOrigin>,
        index: usize,
    ) -> Self {
        self.recording_recipes = Some((origin, index));
        self
    }
    pub(in crate::application_entry) fn new(
        source_receipt: impl Into<
            worth_query_execution::facade::primary_graph::WorthQueryOutputSourceEvidence,
        >,
        preparation: worth_query_execution::facade::primary_graph::WorthQueryPreparedRequiredOutputSource,
        source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
        root: WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self
    where
        Root: worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
    {
        Self {
            recording_recipes: None,
            source_binding: Some(source_binding::RequiredSourceBinding::of::<Root>()),
            source_preparation: Some(preparation),
            source_receipt: Some(source_receipt.into()),
            source_observation,
            root: Some(root),
            root_demand,
            root_settlement: None,
            pending_root_authority: None,
            pending_root_fulfillment: None,
            continuation: None,
            controls,
            complete: false,
            pending_history_delivery: None,
        }
    }

    pub(in crate::application_entry) fn new_initial(
        source_observation: crate::application_entry::WorthQueryApplicationReadObservation,
        root: WorthQueryApplicationProgramDemandHandle<Schema, Program, RootDemand<Schema, Root>>,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Self {
        Self {
            recording_recipes: None,
            source_binding: None,
            source_preparation: None,
            source_receipt: None,
            source_observation,
            root: Some(root),
            root_demand,
            root_settlement: None,
            pending_root_authority: None,
            pending_root_fulfillment: None,
            continuation: None,
            controls,
            complete: false,
            pending_history_delivery: None,
        }
    }
}
