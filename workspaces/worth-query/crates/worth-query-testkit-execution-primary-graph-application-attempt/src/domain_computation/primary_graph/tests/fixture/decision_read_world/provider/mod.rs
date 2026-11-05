//! The smallest candidate provider: one unit node over a nonempty Source.
use crate::domain_computation::primary_graph::tests::fixture::decision_read_world::queries::DecisionIndex;
use crate::domain_computation::primary_graph::tests::fixture::decision_read_world::schema::DecisionReadSchema;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexControl;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexNodeId;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexNodeWriter;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexProvider;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexProviderStop;
use worth_query_execution::facade::domain_computation::WorthQueryManagedIndexSourceRows;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionIndexDenial {}
impl std::fmt::Display for DecisionIndexDenial {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {}
    }
}
impl std::error::Error for DecisionIndexDenial {}

pub type Stop = WorthQueryManagedIndexProviderStop<DecisionIndexDenial>;

pub struct DecisionIndexProvider;
impl WorthQueryManagedIndexProvider<DecisionReadSchema, DecisionIndex> for DecisionIndexProvider {
    type Node = ();
    type Summary = ();
    type Denial = DecisionIndexDenial;
    const IDENTITY: &'static str = DecisionIndex::PROVIDER;
    const VERSION: u32 = DecisionIndex::PROVIDER_VERSION;

    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, Stop> {
        control.checkpoint(1)?;
        Ok(source_count.min(1))
    }
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, DecisionReadSchema, DecisionIndex>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, ()>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Option<WorthQueryManagedIndexNodeId>, Stop> {
        control.checkpoint(1)?;
        if sources.is_empty() {
            return Ok(None);
        }
        Ok(Some(nodes.push(())?))
    }
}
