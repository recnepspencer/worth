//! An owning read-origin request for the complete installed output tree.
use crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory;
use crate::application_entry::*;
use worth_query_declaration::facade::{
    application_program::{
        ApplicationConnectionShape, ApplicationFixedOutputRoot, ApplicationOutputGraphShape,
        ApplicationProgramDefinition,
    },
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::{ApplicationSchema, ApplicationStructuredValueBinding},
};
use worth_query_execution::facade::product::WorthQueryProductBranch;
use worth_query_execution::facade::{
    application_contribution::{
        WorthQueryAdvancementPhase, WorthQueryApplicationOutputDemand,
        WorthQueryProducerOutputFamily,
    },
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
        WorthQueryOutputReadCleanup, WorthQueryOutputReadCloseDenial,
    },
};

type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
type Demand<S, R> = <Connection<S, R> as WorthQueryApplicationRequiredOutputConnection<S>>::Demand;
type Family<S, R> = <Demand<S, R> as WorthQueryApplicationOutputDemand<S>>::OutputFamily;
type Source<S, R> = <Family<S, R> as WorthQueryProducerOutputFamily<S>>::Source;
type Query<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::Query;
type Value<S,R> = <<Source<S,R> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type SourceIntent<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::Input;
type SourceScope<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::ScopeBinding;
type SourcePrincipal<S, R> = <Source<S, R> as ApplicationQueryBinding<S>>::PrincipalIdentity;
type ReadBinding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
type ReadQuery<S, I> = <ReadBinding<S, I> as ApplicationQueryBinding<S>>::Query;
type ReadValue<S,I> = <<ReadBinding<S,I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
mod advance;
mod close;
mod closed_delivery;
mod construction;
mod delivery;
mod delivery_close;
mod delivery_core;
mod native_close;
pub use advance::{WorthQueryProgramOutputAdvanceDenial, WorthQueryProgramOutputOperationAdvance};
pub use close::{
    WorthQueryClosedProgramOutputRequest, WorthQueryProgramOutputCloseDenial,
    WorthQueryProgramOutputCloseRefusal,
};
pub use closed_delivery::WorthQueryClosedProgramOutputDelivery;
pub use construction::{
    WorthQueryProgramOutputConstructionDenial, WorthQueryProgramOutputConstructionRefusal,
};
pub use delivery_close::WorthQueryProgramOutputDeliveryCloseRefusal;
use delivery_core::ProgramOutputDeliveryCore;

struct ReadCustody<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    cleanup: Option<WorthQueryOutputReadCleanup>,
    tree: Option<WorthQueryApplicationProgramOutputHandle<S, P, R>>,
    basis: Option<WorthQueryApplicationReadObservation>,
    demand: Demand<S, R>,
    controls: WorthQueryOutputDemandControls,
}
struct Core<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    custody: ReadCustody<S, P, R>,
    branch: WorthQueryProductBranch,
    // All native custody is destroyed before the authentic runtime.
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
}
/// One original complete-tree read request. No initiating scope borrow escapes.
#[must_use]
pub struct WorthQueryProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    core: Core<S, P, R>,
}
/// All original tree members settled; fresh delivery still checks their currentness.
#[must_use]
pub struct WorthQuerySettledProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    custody: ReadCustody<S, P, R>,
    // The delivery core retains the runtime until native custody is destroyed.
    delivery: ProgramOutputDeliveryCore<S, P, R>,
}
/// A genuine terminal read request, retaining its original tree and cleanup.
#[must_use]
pub struct WorthQueryTerminalProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    denial: WorthQueryProgramOutputAdvanceDenial,
    core: Core<S, P, R>,
}
impl<S, P, R> Core<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    fn matches(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) -> bool {
        std::ptr::eq(self.runtime.runtime(), request.application) && self.branch == request.branch
    }
}
impl<S, P, R> WorthQueryTerminalProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn producer_outcome(
        &mut self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryApplicationUncommitted>
    {
        self.core
            .custody
            .cleanup
            .as_mut()
            .and_then(WorthQueryOutputReadCleanup::producer_outcome)
    }
    pub fn denial(&self) -> &WorthQueryProgramOutputAdvanceDenial {
        &self.denial
    }
}

impl<S, P, R> WorthQueryProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub fn source_observation(&self) -> &WorthQueryApplicationReadObservation {
        self.core
            .custody
            .basis
            .as_ref()
            .expect("successful construction retains its issued basis")
    }
}
