//! One provider contract for spatial and nonspatial pure mechanisms.
use super::{
    WorthQueryManagedIndexCandidateWriter, WorthQueryManagedIndexControl,
    WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeRegion,
    WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexNodes,
    WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexSourceRows,
    WorthQueryManagedIndexSummarySources,
};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding,
    application_schema::{ApplicationScalarValueBinding, ApplicationSchema},
};

/// A Native-issued occurrence delta. Coordinates describe material and carry no
/// Source loan, public admission or mutation authority.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryManagedIndexSourceChange {
    pub(super) before: Option<super::WorthQueryManagedIndexSourceSlot>,
    pub(super) after: Option<super::WorthQueryManagedIndexSourceSlot>,
}
impl WorthQueryManagedIndexSourceChange {
    pub const fn before(&self) -> Option<super::WorthQueryManagedIndexSourceSlot> {
        self.before
    }
    pub const fn after(&self) -> Option<super::WorthQueryManagedIndexSourceSlot> {
        self.after
    }
}

/// The actual declared probe codec's domain value.
pub type WorthQueryManagedIndexProbeValue<S, I> = <<I as ApplicationManagedIndexBinding<S>>::ProbeBinding as ApplicationScalarValueBinding>::Value;
/// One actual Source row selected by Query's complete traversal.
pub type WorthQueryManagedIndexSourceRow<'a, S, I> = super::super::WorthQueryApplicationProjectionRow<
    'a, S, <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as worth_query_declaration::facade::application_query::ApplicationQueryBinding<S>>::Query,
>;

/// Pure controlled mechanics over finite framework storage and genuine source slots.
/// No callback returns a root object, entity handle, permission or completeness proof.
pub trait WorthQueryManagedIndexProvider<S, I>: Send + Sync + 'static
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    type Node: Copy + Send + Sync + 'static;
    type Summary: Copy + Eq + Send + Sync + 'static;
    type Denial: Copy + Eq + std::error::Error + Send + Sync + 'static;
    const IDENTITY: &'static str;
    const VERSION: u32;

    fn source_summary(
        &self,
        _slot: super::WorthQueryManagedIndexSourceSlot,
        _row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::SummaryUnavailable)
            .into())
    }
    fn merge_summaries(
        &self,
        _parts: [Option<Self::Summary>; 3],
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::SummaryUnavailable)
            .into())
    }
    fn node_summary(
        &self,
        _node: Self::Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::SummaryUnavailable)
            .into())
    }
    fn summary_may_match(
        &self,
        _summary: Self::Summary,
        _sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        _probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::SummaryUnavailable)
            .into())
    }

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
        _sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        _nodes: &WorthQueryManagedIndexNodes<Self::Node>,
        _probe: &WorthQueryManagedIndexProbeValue<S, I>,
        _candidates: &mut WorthQueryManagedIndexCandidateWriter<'_>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::MembershipUnavailable)
            .into())
    }

    /// Runs even for an empty Source, before complete traversal or candidates.
    fn validate_probe(
        &self,
        _probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.poll().map_err(Into::into)
    }

    /// Pure candidate predicate. Query controls exhaustive visitation and emission.
    /// This is distinct from exact domain refinement and grants no READ authority.
    fn matches_source(
        &self,
        _row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        _probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::MembershipUnavailable)
            .into())
    }

    /// Describe only the actual node's Source contribution and child coordinates.
    /// Source-dependent interpretation cannot be reused after local node edits.
    /// BinarySourceTree roots are checked by Query; this is no query coverage assertion.
    fn decompose(
        &self,
        _node: Self::Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, WorthQueryManagedIndexProviderStop<Self::Denial>>
    {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::SourceStructureUnavailable)
            .into())
    }

    /// Isolated maintenance of the actual changed occurrences. The writer edits
    /// immutable retained pages; returning a root never publishes it. A provider
    /// without maintenance refuses instead of rebuilding a warm Source.
    fn update(
        &self,
        _before: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        _after: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        _changes: &[WorthQueryManagedIndexSourceChange],
        _root: Option<WorthQueryManagedIndexNodeId>,
        _nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Self::Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<
        Option<WorthQueryManagedIndexNodeId>,
        WorthQueryManagedIndexProviderStop<Self::Denial>,
    > {
        Err(control
            .refuse(super::WorthQueryManagedIndexStop::IncrementalUnavailable)
            .into())
    }
}
