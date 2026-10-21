use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputEdge, ApplicationOutputEdgesShape,
    ApplicationOutputGraphShape, ApplicationOutputLeaf, ApplicationProgramDefinition,
    ApplicationRequiredOutputRoot,
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
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection,
};

use super::program_output_settlement::ProgramOutputRecord;
use super::program_output_work::ProgramOutputTraversalWork;
use crate::application_entry::demand::{
    WorthQueryApplicationProgramDemandHandle, WorthQueryApplicationProgramDemandProgress,
};
use crate::application_entry::{
    WorthQueryApplicationOutputDemandSettlement, WorthQueryApplicationRequest,
    WorthQueryOutputDemandControls, WorthQueryRequiredOutputPreparationDenial,
};

use worth_query_execution::publication_boundary::discovery_currentness::WorthQueryRetainedQueryDependencies;

mod branch;
mod edge;
mod leaf;

mod sealed {
    pub trait Continuation {}
    pub trait Factory {}
}

type Family<Schema, Demand> = <Demand as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type Source<Schema, Demand> =
    <Family<Schema, Demand> as WorthQueryProducerOutputFamily<Schema>>::Source;
type SourceQuery<Schema, Demand> =
    <Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::Query;
type SourceValue<Schema, Demand> =
    <<Source<Schema, Demand> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type Binding<Schema, Connection> = <Connection as ApplicationConnectionShape<Schema>>::Binding;
type ChildDemand<Schema, Connection> =
    <Binding<Schema, Connection> as WorthQueryApplicationDependentOutputConnection<Schema>>::Demand;
type Discovery<Schema, Connection> =
    <Binding<Schema, Connection> as WorthQueryApplicationDependentOutputConnection<Schema>>::Discovery;
type DiscoveryBinding<Schema, Connection> =
    <Discovery<Schema, Connection> as ApplicationQueryIntent<Schema>>::Binding;
type DiscoveryValue<Schema, Connection> =
    <<DiscoveryBinding<Schema, Connection> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

/// A sealed factory carries leaf eligibility; the native owner still decides legality.
#[doc(hidden)]
pub type ProgramOutputUnavailableCompletion<Schema, Program, Root> = fn(
    &mut super::performed_outputs::WorthQueryApplicationProgramOutputHandle<Schema, Program, Root>,
    &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    &WorthQueryProgramApplicationRuntime<Schema, Program>,
    &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
) -> Result<
    super::performed_outputs::WorthQueryApplicationProgramOutputUnavailable,
    WorthQueryRequiredOutputPreparationDenial,
>;

#[doc(hidden)]
pub enum ProgramOutputContinuationProgress {
    Pending,
    /// The same owner made native progress and remains runnable.
    Progressed,
    Settled {
        outputs: Vec<ProgramOutputRecord>,
        work: ProgramOutputTraversalWork,
        dependencies: Option<WorthQueryRetainedQueryDependencies>,
    },
}

#[doc(hidden)]
pub trait ProgramOutputContinuation<Schema, Program>: sealed::Continuation
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn close_read_request(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial>;
    fn advance(
        &mut self,
        phase: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<
            '_,
        >,
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    ) -> Result<ProgramOutputContinuationProgress, WorthQueryRequiredOutputPreparationDenial>;
}

#[doc(hidden)]
/// Creates owned, effect-free traversal state. Source queries and native
/// admissions belong to `advance`, after the state is retained by its parent.
pub trait ProgramOutputContinuationFactory<Schema, Program, ParentDemand>: sealed::Factory
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    ParentDemand: WorthQueryApplicationOutputDemand<Schema> + Clone,
{
    /// Nonleaf shapes carry no negative-completion capability.
    fn unavailable_completion<Root>() -> Option<ProgramOutputUnavailableCompletion<Schema, Program, Root>>
    where
        Schema: 'static,
        Self: Sized,
        Root: ApplicationOutputGraphShape<Schema, Dependents = Self> + ApplicationRequiredOutputRoot,
        Binding<Schema, Root::RootConnection>: WorthQueryApplicationRequiredOutputConnection<Schema, Demand = ParentDemand>,
        SourceValue<Schema, ParentDemand>: WorthQueryApplicationProjection<Schema, SourceQuery<Schema, ParentDemand>> + Clone,
        <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Source<Schema, ParentDemand>>,
        <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Source<Schema, ParentDemand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    {
        None
    }

    fn start(
        application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
        parent_demand: &ParentDemand,
        _parent_settlement: &WorthQueryApplicationOutputDemandSettlement<
            SourceQuery<Schema, ParentDemand>,
        >,
        parent_authority: &std::sync::Arc<
            WorthQuerySettledProgramOutput<Schema, Program, ParentDemand>,
        >,
        minimum_observation: &crate::application_entry::WorthQueryApplicationReadObservation,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        controls: WorthQueryOutputDemandControls,
    ) -> Result<
        Box<dyn ProgramOutputContinuation<Schema, Program> + 'static>,
        WorthQueryRequiredOutputPreparationDenial,
    >;
}
