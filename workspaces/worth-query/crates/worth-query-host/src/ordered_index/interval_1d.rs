//! Stock checked 1D interval-overlap managed-index adapter built on
//! [`WorthQueryOrderedIndexAdapter`].
//!
//! Either endpoint may be an Optional field: a row with an absent endpoint is
//! sparse (see [`WorthQueryOrderedKeyField`]).
use super::coord::{
    WorthQueryAxisEnvelope, WorthQueryOrderedCoord, WorthQueryRangeInclusion,
    WorthQueryStockIndexDenial,
};
use super::key_field::WorthQueryOrderedKeyField;
use super::probes::WorthQueryInterval1DProbe;
use super::{
    WorthQueryOrderedIndexAdapter, WorthQueryOrderedIndexDomain,
    WorthQueryOrderedIndexPredicateDomain, WorthQueryOrderedIndexSources,
};
use std::cmp::Ordering;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexProbeValue,
    WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexSourceRow,
    WorthQueryManagedIndexSourceSlot,
};

/// Fixed-size `Copy` associative 1D interval envelope summary.
pub type WorthQueryInterval1DEnvelope<C> = WorthQueryAxisEnvelope<C, 1>;

/// Stock checked 1D interval domain over declared Required or Optional
/// `(start, end)` fields of `i64`, `u64`, `CanonicalDate`, `CanonicalTimestamp`,
/// or `CanonicalF64`.
pub struct WorthQueryInterval1DDomain<S, I, Start, End, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    start: Start,
    end: End,
    inclusion: WorthQueryRangeInclusion,
    _marker: PhantomData<fn() -> (S, I, C)>,
}

/// Stock checked 1D interval-overlap managed-index provider with optional inverse watch routing.
pub type WorthQueryInterval1DIndex<S, I, Start, End, C> =
    WorthQueryOrderedIndexAdapter<WorthQueryInterval1DDomain<S, I, Start, End, C>>;

impl<S, I, Start, End, C> WorthQueryInterval1DIndex<S, I, Start, End, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
{
    /// Half-open `[start, end)` interval index (valid iff `start < end`).
    pub const fn half_open(start: Start, end: End) -> Self {
        Self::new_with_inclusion(start, end, WorthQueryRangeInclusion::HalfOpen)
    }

    /// Closed `[start, end]` interval index (valid iff `start <= end`).
    pub const fn closed(start: Start, end: End) -> Self {
        Self::new_with_inclusion(start, end, WorthQueryRangeInclusion::Closed)
    }

    /// Interval index with explicit [`WorthQueryRangeInclusion`].
    pub const fn new_with_inclusion(
        start: Start,
        end: End,
        inclusion: WorthQueryRangeInclusion,
    ) -> Self {
        WorthQueryOrderedIndexAdapter::new(WorthQueryInterval1DDomain {
            start,
            end,
            inclusion,
            _marker: PhantomData,
        })
    }
}

type RowBounds<C> = Option<([C; 1], [C; 1])>;

impl<S, I, Start, End, C> WorthQueryInterval1DDomain<S, I, Start, End, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
    Start: WorthQueryOrderedKeyField<S, I, Value = C>,
    End: WorthQueryOrderedKeyField<S, I, Value = C>,
{
    /// The row's `([start], [end])`, or `None` when either endpoint is absent.
    fn read_bounds(
        &self,
        row: &WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<RowBounds<C>, WorthQueryManagedIndexProviderStop<WorthQueryStockIndexDenial>> {
        let Some(lo) = self.start.read_key(row, control)? else {
            return Ok(None);
        };
        Ok(self.end.read_key(row, control)?.map(|hi| ([lo], [hi])))
    }
}

impl<S, I, Start, End, C> WorthQueryOrderedIndexDomain<S, I>
    for WorthQueryInterval1DDomain<S, I, Start, End, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
    WorthQueryManagedIndexProbeValue<S, I>: WorthQueryInterval1DProbe<C>,
    Start: WorthQueryOrderedKeyField<S, I, Value = C>,
    End: WorthQueryOrderedKeyField<S, I, Value = C>,
{
    type Summary = WorthQueryInterval1DEnvelope<C>;
    type Denial = WorthQueryStockIndexDenial;
    const IDENTITY: &'static str = I::PROVIDER;
    const VERSION: u32 = I::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, S, I>,
        right: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let left = self.read_bounds(&left, control)?;
        let right = self.read_bounds(&right, control)?;
        control.checkpoint(1)?;
        Ok(WorthQueryAxisEnvelope::row_cmp(left, right))
    }

    fn source_summary(
        &self,
        _slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let bounds = self.read_bounds(&row, control)?;
        control.checkpoint(1)?;
        Ok(WorthQueryAxisEnvelope::from_row_bounds(
            bounds,
            self.inclusion,
        ))
    }

    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        _sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(left.combine(*right))
    }

    fn validate_probe(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        let (lo, hi) = probe.interval_1d_bounds();
        if !WorthQueryAxisEnvelope::bounds_valid([lo], [hi], self.inclusion) {
            return Err(WorthQueryManagedIndexProviderStop::Domain(
                WorthQueryStockIndexDenial::InvalidProbeBounds,
            ));
        }
        Ok(())
    }

    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        _sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        let (lo, hi) = probe.interval_1d_bounds();
        Ok(summary.may_overlap_bounds([lo], [hi], self.inclusion))
    }

    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let bounds = self.read_bounds(&row, control)?;
        control.checkpoint(1)?;
        let (p_lo, p_hi) = probe.interval_1d_bounds();
        Ok(bounds.is_some_and(|(r_min, r_max)| {
            WorthQueryAxisEnvelope::bounds_overlap(r_min, r_max, [p_lo], [p_hi], self.inclusion)
        }))
    }
}

impl<S, I, Start, End, C> WorthQueryOrderedIndexPredicateDomain<S, I>
    for WorthQueryInterval1DDomain<S, I, Start, End, C>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    C: WorthQueryOrderedCoord,
    WorthQueryManagedIndexProbeValue<S, I>: WorthQueryInterval1DProbe<C>,
    Start: WorthQueryOrderedKeyField<S, I, Value = C>,
    End: WorthQueryOrderedKeyField<S, I, Value = C>,
{
    type PredicateSummary = WorthQueryInterval1DEnvelope<C>;

    fn compare_probes(
        &self,
        left: &WorthQueryManagedIndexProbeValue<S, I>,
        right: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        let (l_lo, l_hi) = left.interval_1d_bounds();
        let (r_lo, r_hi) = right.interval_1d_bounds();
        Ok(WorthQueryAxisEnvelope::lex_cmp(
            [l_lo],
            [l_hi],
            [r_lo],
            [r_hi],
        ))
    }

    fn predicate_summary(
        &self,
        probe: &WorthQueryManagedIndexProbeValue<S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::PredicateSummary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        let (lo, hi) = probe.interval_1d_bounds();
        Ok(WorthQueryAxisEnvelope::from_bounds(
            [lo],
            [hi],
            self.inclusion,
        ))
    }

    fn predicate_region_may_match(
        &self,
        summary: &Self::PredicateSummary,
        _extrema: [&WorthQueryManagedIndexProbeValue<S, I>; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let bounds = self.read_bounds(&row, control)?;
        control.checkpoint(1)?;
        Ok(bounds
            .is_some_and(|(r_min, r_max)| summary.may_overlap_bounds(r_min, r_max, self.inclusion)))
    }
}
