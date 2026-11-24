//! Concise (~55 LOC) checked calendar-interval overlap domain adapter over binary `FocusWindow`.
use super::queries::{end_min, start_min};
use super::schema::{DaymarkSchema, FocusIntervalIndex, FocusWindow};
use std::cmp::Ordering;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::ordered_index::*;
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusIntervalDenial;
impl std::fmt::Display for FocusIntervalDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("focus interval denial")
    }
}
impl std::error::Error for FocusIntervalDenial {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FocusEnvelope {
    pub min_start: i64,
    pub max_end: i64,
}
impl WorthQueryManagedIndexPredicateSummary for FocusEnvelope {
    fn merge(own: &Self, children: [Option<&Self>; 2]) -> Self {
        children
            .into_iter()
            .flatten()
            .fold(*own, |acc, child| Self {
                min_start: acc.min_start.min(child.min_start),
                max_end: acc.max_end.max(child.max_end),
            })
    }
}

fn read_span(
    row: &WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
    control: &WorthQueryManagedIndexControl<'_>,
) -> Result<(i64, i64), WorthQueryManagedIndexProviderStop<FocusIntervalDenial>> {
    let start = read_ordered_field(row, start_min(), control)?;
    let end = read_ordered_field(row, end_min(), control)?;
    Ok((start, end))
}

pub struct FocusIntervalDomain;
impl WorthQueryOrderedIndexDomain<DaymarkSchema, FocusIntervalIndex> for FocusIntervalDomain {
    type Summary = FocusEnvelope;
    type Denial = FocusIntervalDenial;
    const IDENTITY: &'static str = FocusIntervalIndex::PROVIDER;
    const VERSION: u32 = FocusIntervalIndex::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
        right: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        Ok(read_span(&left, control)?.cmp(&read_span(&right, control)?))
    }
    fn source_summary(
        &self,
        _: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let (min_start, max_end) = read_span(&row, control)?;
        Ok(FocusEnvelope { min_start, max_end })
    }
    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        _: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, FocusIntervalIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(FocusEnvelope {
            min_start: left.min_start.min(right.min_start),
            max_end: left.max_end.max(right.max_end),
        })
    }
    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        _: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, FocusIntervalIndex>,
        probe: &FocusWindow,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(summary.min_start < probe.end && probe.start < summary.max_end)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
        probe: &FocusWindow,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let (start, end) = read_span(&row, control)?;
        Ok(start < probe.end && probe.start < end)
    }
}

impl WorthQueryOrderedIndexPredicateDomain<DaymarkSchema, FocusIntervalIndex>
    for FocusIntervalDomain
{
    type PredicateSummary = FocusEnvelope;
    fn compare_probes(
        &self,
        left: &FocusWindow,
        right: &FocusWindow,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok((left.start, left.end).cmp(&(right.start, right.end)))
    }
    fn predicate_summary(
        &self,
        probe: &FocusWindow,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<FocusEnvelope, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(FocusEnvelope {
            min_start: probe.start,
            max_end: probe.end,
        })
    }
    fn predicate_region_may_match(
        &self,
        summary: &FocusEnvelope,
        _: [&FocusWindow; 2],
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, FocusIntervalIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        let (start, end) = read_span(&row, control)?;
        Ok(start < summary.max_end && summary.min_start < end)
    }
}
