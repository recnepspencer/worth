//! One installed required-root recipe supplies validation and fulfillment together.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot;
use worth_query_execution::facade::primary_graph::{
    WorthQueryFulfilledRequiredRoot, WorthQueryOutputSourceEvidence,
    WorthQueryPreparedRequiredOutputSource,
};
pub(super) struct RequiredSourceBinding<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(super) validate: source::RequiredSourceValidator<S, P>,
    pub(super) fulfill: RequiredSourceFulfillment<S, P, D>,
}
type RequiredSourceFulfillment<S, P, D> = fn(
    &AdvancementPhase<'_>,
    &WorthQueryProgramApplicationRuntime<S, P>,
    &WorthQueryApplicationProgramDemandHandle<S, P, D>,
    &WorthQueryPreparedRequiredOutputSource,
    &WorthQueryOutputSourceEvidence,
    &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<Query<S, D>>,
) -> Result<
    WorthQueryFulfilledRequiredRoot,
    crate::application_entry::WorthQueryApplicationOutputDemandDenial,
>;
impl<S, P, D> RequiredSourceBinding<S, P, D>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(super) fn of<R>() -> Self
    where
        R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
        RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S, Demand = D>,
    {
        Self {
            validate: source::validate::<S, P, R>,
            fulfill: fulfill::<S, P, D, R>,
        }
    }
}
fn fulfill<S, P, D, R>(
    phase: &AdvancementPhase<'_>,
    application: &WorthQueryProgramApplicationRuntime<S, P>,
    demand: &WorthQueryApplicationProgramDemandHandle<S, P, D>,
    prepared: &WorthQueryPreparedRequiredOutputSource,
    receipt: &WorthQueryOutputSourceEvidence,
    settlement: &crate::application_entry::WorthQueryApplicationOutputDemandSettlement<Query<S, D>>,
) -> Result<
    WorthQueryFulfilledRequiredRoot,
    crate::application_entry::WorthQueryApplicationOutputDemandDenial,
>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    RootConnection<S, R>: WorthQueryApplicationRequiredOutputConnection<S, Demand = D>,
{
    demand.retain_fulfilled_root::<R>(phase, application, prepared, receipt, settlement)
}
