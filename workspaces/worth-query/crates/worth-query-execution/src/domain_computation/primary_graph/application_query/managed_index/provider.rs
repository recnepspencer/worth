//! One provider contract for spatial and nonspatial pure mechanisms.
use super::{
    WorthQueryManagedIndexCandidateWriter, WorthQueryManagedIndexControl,
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes,
    WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexSourceRows,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding,
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
};

/// The actual declared probe codec's domain value.
pub type WorthQueryManagedIndexProbeValue<S, I> = <<I as ApplicationManagedIndexBinding<S>>::ProbeBinding as ApplicationScalarValueBinding>::Value;

/// Pure controlled mechanics over finite framework storage and genuine source slots.
/// No callback returns a root object, entity handle, permission or completeness proof.
pub trait WorthQueryManagedIndexProvider<S, I>: Send + Sync + 'static
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    type Node: Copy + Send + Sync + 'static;
    type Denial: Copy + Eq + std::error::Error + Send + Sync + 'static;
    const IDENTITY: &'static str;
    const VERSION: u32;

    /// Node count only. The runtime computes and admits the actual storage layout.
    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, WorthQueryManagedIndexProviderStop<Self::Denial>>;
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Self::Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<
        Option<WorthQueryManagedIndexNodeId>,
        WorthQueryManagedIndexProviderStop<Self::Denial>,
    >;
    fn query(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        nodes: &WorthQueryManagedIndexNodes<Self::Node>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>>;
}
