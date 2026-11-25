//! Standard checked ordered managed-index adapter over Query's checked binary source tree.
//!
//! Consumers implement [`WorthQueryOrderedIndexDomain`] (and optionally
//! [`WorthQueryOrderedIndexPredicateDomain`] for inverse watch routing) with only
//! metered row comparison, associative summary, and exact probe matching.
//! [`WorthQueryOrderedIndexAdapter`] owns the AVL tree, incremental updates,
//! deterministic slot tie-breaking, and summary propagation.

mod bounds;
mod box_2d;
mod box_3d;
mod box_nd;
mod coord;
mod interval_1d;
mod probes;
mod sources;
mod text_prefix;
mod tree;

use std::cmp::Ordering;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexNodeId, WorthQueryManagedIndexNodeRegion,
    WorthQueryManagedIndexNodeWriter, WorthQueryManagedIndexPredicateProvider,
    WorthQueryManagedIndexPredicateSummary, WorthQueryManagedIndexProbeValue,
    WorthQueryManagedIndexProvider, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexSourceChange, WorthQueryManagedIndexSourceRow,
    WorthQueryManagedIndexSourceRows, WorthQueryManagedIndexSourceSlot,
    WorthQueryManagedIndexSummarySources,
};

pub use bounds::{
    compare_text_fields, compare_text_metered, map_projection_denial, map_text_read_denial,
    prefix_bounds_may_match_field, prefix_bounds_may_match_metered, prefix_matches_text_field,
    prefix_order_metered, prefix_watch_bounds_may_match_field,
    prefix_watch_bounds_may_match_metered, read_ordered_field, WorthQueryOrderedPrefixOrder,
    WorthQueryOrderedSlotBounds, WorthQueryOrderedTextOrder,
};
pub use box_2d::{WorthQueryBox2DDomain, WorthQueryBox2DEnvelope, WorthQueryBox2DIndex};
pub use box_3d::{WorthQueryBox3DDomain, WorthQueryBox3DEnvelope, WorthQueryBox3DIndex};
pub use coord::{
    WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion,
    WorthQueryStockIndexDenial,
};
pub use interval_1d::{
    WorthQueryInterval1DDomain, WorthQueryInterval1DEnvelope, WorthQueryInterval1DIndex,
};
pub use probes::{
    WorthQueryBox2D, WorthQueryBox2DBinding, WorthQueryBox2DF64, WorthQueryBox2DF64Binding,
    WorthQueryBox2DProbe, WorthQueryBox3D, WorthQueryBox3DBinding, WorthQueryBox3DF64,
    WorthQueryBox3DF64Binding, WorthQueryBox3DProbe, WorthQueryInterval1D,
    WorthQueryInterval1DBinding, WorthQueryInterval1DProbe,
};
pub use sources::{WorthQueryOrderedIndexNode, WorthQueryOrderedIndexSources};
pub use text_prefix::{WorthQueryTextPrefixDomain, WorthQueryTextPrefixIndex};

/// Pure metered domain contract for a checked ordered managed index.
///
/// Ordering ties between distinct rows break automatically by `WorthQueryManagedIndexSourceSlot`.
pub trait WorthQueryOrderedIndexDomain<S, I>: Send + Sync + 'static
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    /// Bounded inline summary copied without heap allocation during tree maintenance.
    type Summary: Copy + Eq + Send + Sync + 'static;
    type Denial: Copy + Eq + std::error::Error + Send + Sync + 'static;
    const IDENTITY: &'static str;
    const VERSION: u32;

    /// Metered total ordering over two genuine source rows (slot tie-breaking is automatic).
    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, S, I>,
        right: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    /// Computes the leaf summary enclosing one genuine source row.
    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    /// Associative enclosure merge of two summaries (e.g. min/max bounds or interval envelope).
    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    /// Sound pruning predicate: must return `true` whenever any row enclosed by `summary`
    /// satisfies [`Self::matches_source`] for `probe`.
    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    /// Exact metered match predicate evaluated on genuine candidate source rows.
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    /// Validates a decoded probe before traversal, including on empty sources.
    fn validate_probe(
        &self,
        _probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1).map_err(Into::into)
    }
}

/// Optional inverse watch-routing contract for [`WorthQueryOrderedIndexAdapter`].
pub trait WorthQueryOrderedIndexPredicateDomain<S, I>: WorthQueryOrderedIndexDomain<S, I>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    type PredicateSummary: WorthQueryManagedIndexPredicateSummary;

    fn compare_probes(
        &self,
        left: &WorthQueryManagedIndexProbeValue<S, I>,
        right: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    fn predicate_summary(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::PredicateSummary, WorthQueryManagedIndexProviderStop<Self::Denial>>;

    fn predicate_region_may_match(
        &self,
        summary: &Self::PredicateSummary,
        extrema: [&WorthQueryManagedIndexProbeValue<S, I>; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>>;
}

/// Reusable checked ordered managed-index provider backed by an AVL binary source tree.
#[derive(Clone, Debug)]
pub struct WorthQueryOrderedIndexAdapter<D> {
    domain: D,
}

impl<D> WorthQueryOrderedIndexAdapter<D> {
    /// Wraps a pure [`WorthQueryOrderedIndexDomain`] into a checked Query index provider.
    pub const fn new(domain: D) -> Self {
        Self { domain }
    }

    /// Borrows the wrapped domain contract.
    pub const fn domain(&self) -> &D {
        &self.domain
    }
}

impl<S, I, D> WorthQueryManagedIndexProvider<S, I> for WorthQueryOrderedIndexAdapter<D>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexDomain<S, I>,
{
    type Node = WorthQueryOrderedIndexNode<D::Summary>;
    type Summary = D::Summary;
    type Denial = D::Denial;
    const IDENTITY: &'static str = D::IDENTITY;
    const VERSION: u32 = D::VERSION;

    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.source_summary(slot, row, control)
    }

    fn merge_summaries(
        &self,
        parts: [Option<&Self::Summary>; 3],
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        tree::fold_summaries(
            &self.domain,
            parts,
            &WorthQueryOrderedIndexSources::from_summary(sources),
            control,
        )
    }

    fn node_summary<'node>(
        &self,
        node: &'node Self::Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<&'node Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(&node.summary)
    }

    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryManagedIndexSummarySources<'_, '_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.summary_may_match(
            summary,
            &WorthQueryOrderedIndexSources::from_summary(sources),
            probe,
            control,
        )
    }

    fn node_capacity(
        &self,
        source_count: usize,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<usize, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(source_count)
    }

    /// Cold build: O(N log N) comparisons and summary merges, O(N) nodes, zero extra heap.
    fn build(
        &self,
        sources: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Self::Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<
        Option<WorthQueryManagedIndexNodeId>,
        WorthQueryManagedIndexProviderStop<Self::Denial>,
    > {
        control.checkpoint(1)?;
        let mut root = None;
        let mut tree = tree::TreeContext::new(sources, nodes, &self.domain, control);
        for slot in sources.slots() {
            let slot = slot?;
            root = Some(tree.insert(root, slot)?);
        }
        Ok(root)
    }

    fn validate_probe(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.validate_probe(probe, control)
    }

    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.matches_source(row, probe, control)
    }

    fn decompose(
        &self,
        node: &Self::Node,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<WorthQueryManagedIndexNodeRegion, WorthQueryManagedIndexProviderStop<Self::Denial>>
    {
        control.checkpoint(1)?;
        Ok(WorthQueryManagedIndexNodeRegion::new(
            Some(node.source),
            [node.left, node.right],
        ))
    }

    /// Warm incremental maintenance: O(|changes| log N) removals under `before` followed by
    /// insertions under `after`, propagating summaries across all touched ancestors and rotations.
    fn update(
        &self,
        before: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        after: &WorthQueryManagedIndexSourceRows<'_, S, I>,
        changes: &[WorthQueryManagedIndexSourceChange],
        mut root: Option<WorthQueryManagedIndexNodeId>,
        nodes: &mut WorthQueryManagedIndexNodeWriter<'_, Self::Node>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<
        Option<WorthQueryManagedIndexNodeId>,
        WorthQueryManagedIndexProviderStop<Self::Denial>,
    > {
        control.checkpoint(1)?;
        {
            let mut before_tree = tree::TreeContext::new(before, nodes, &self.domain, control);
            for change in changes {
                control.checkpoint(1)?;
                if let Some(slot) = change.before() {
                    root = before_tree.remove(root, slot)?;
                }
            }
        }
        {
            let mut after_tree = tree::TreeContext::new(after, nodes, &self.domain, control);
            for change in changes {
                control.checkpoint(1)?;
                if let Some(slot) = change.after() {
                    root = Some(after_tree.insert(root, slot)?);
                }
            }
        }
        Ok(root)
    }
}

impl<S, I, D> WorthQueryManagedIndexPredicateProvider<S, I> for WorthQueryOrderedIndexAdapter<D>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    D: WorthQueryOrderedIndexPredicateDomain<S, I>,
{
    type PredicateSummary = D::PredicateSummary;

    fn compare_probes(
        &self,
        left: &WorthQueryManagedIndexProbeValue<S, I>,
        right: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.compare_probes(left, right, control)
    }

    fn predicate_summary(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::PredicateSummary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain.predicate_summary(probe, control)
    }

    fn predicate_region_may_match(
        &self,
        summary: &Self::PredicateSummary,
        extrema: [&WorthQueryManagedIndexProbeValue<S, I>; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        self.domain
            .predicate_region_may_match(summary, extrema, row, control)
    }
}
