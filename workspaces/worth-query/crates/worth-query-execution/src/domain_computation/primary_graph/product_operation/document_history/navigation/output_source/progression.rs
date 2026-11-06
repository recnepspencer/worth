//! Completed history custody remains distinct from ordinary output admission.
use super::*;
use crate::domain_computation::primary_graph::application_installation::WorthQuerySettledProgramOutput;

/// A history settlement cannot admit ordinary dependent work.
///
/// ```compile_fail,E0308
/// use worth_query_execution::{
///     facade::{application_installation::WorthQuerySettledProgramOutput,
///         application_contribution::WorthQueryApplicationOutputDemand},
///     publication_boundary::document_history::WorthQuerySettledHistoryProgramOutput,
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
/// fn ordinary<S, P, D>(_: &WorthQuerySettledProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {}
/// fn bypass<S, P, D>(history: &WorthQuerySettledHistoryProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {
///     ordinary(history);
/// }
/// ```
///
/// ```
/// use worth_query_execution::facade::{
///     application_installation::WorthQuerySettledProgramOutput,
///     application_contribution::WorthQueryApplicationOutputDemand,
/// };
/// use worth_query_installation::facade::ApplicationSchema;
/// use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
/// fn ordinary<S, P, D>(_: &WorthQuerySettledProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {}
/// fn supported<S, P, D>(settled: &WorthQuerySettledProgramOutput<S, P, D>)
/// where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
///     D: WorthQueryApplicationOutputDemand<S> {
///     ordinary(settled);
/// }
/// ```
pub struct WorthQuerySettledHistoryProgramOutput<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub(in crate::domain_computation::primary_graph) settled:
        WorthQuerySettledProgramOutput<S, P, D>,
    pub(in crate::domain_computation::primary_graph) profile: WorthQueryHistoryOutputProfile,
    pub(in crate::domain_computation::primary_graph) original:
        Arc<WorthQueryApplicationReadObservation>,
}

pub enum WorthQueryHistoryProgramOutputAdvance<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    Pending,
    Progressed,
    Settled(WorthQuerySettledHistoryProgramOutput<S, P, D>),
}

#[derive(Debug)]
pub enum WorthQueryHistoryOutputAdvanceDenial {
    Authority(WorthQueryHistoryNavigationDenial),
    Demand(WorthQueryOutputDemandDenial),
}

impl<S, P, D> WorthQuerySettledHistoryProgramOutput<S, P, D>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D: WorthQueryApplicationOutputDemand<S>,
{
    pub fn retained(
        &self,
    ) -> Arc<crate::domain_computation::primary_graph::WorthQueryOutputDemandSettlement> {
        self.settled.retained()
    }
}
