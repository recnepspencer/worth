//! Concise (~50 LOC) checked title-prefix domain adapter over native `String` probes.
use super::queries::task_title;
use super::schema::{DaymarkSchema, TitlePrefixIndex};
use std::cmp::Ordering;
use worth_query_decl::facade::application_query::ApplicationManagedIndexBinding;
use worth_query_host::facade::ordered_index::*;
use worth_query_host::facade::primary_graph::*;

const ORDER: WorthQueryOrderedTextOrder = WorthQueryOrderedTextOrder::AsciiCaseInsensitive;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TitlePrefixDenial;
impl std::fmt::Display for TitlePrefixDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("title prefix denial")
    }
}
impl std::error::Error for TitlePrefixDenial {}

pub struct TitlePrefixDomain;
impl WorthQueryOrderedIndexDomain<DaymarkSchema, TitlePrefixIndex> for TitlePrefixDomain {
    type Summary = WorthQueryOrderedSlotBounds;
    type Denial = TitlePrefixDenial;
    const IDENTITY: &'static str = TitlePrefixIndex::PROVIDER;
    const VERSION: u32 = TitlePrefixIndex::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        right: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        compare_text_fields(&left, &right, task_title(), ORDER, control)
    }
    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        _: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(WorthQueryOrderedSlotBounds::point(slot))
    }
    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        left.merge_with(right, sources, control, |a, b, c| {
            self.compare_rows(a, b, c)
        })
    }
    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, DaymarkSchema, TitlePrefixIndex>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_bounds_may_match_field(summary, sources, task_title(), probe, ORDER, control)
    }
    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_matches_text_field(&row, task_title(), probe, ORDER, control)
    }
}

impl WorthQueryOrderedIndexPredicateDomain<DaymarkSchema, TitlePrefixIndex> for TitlePrefixDomain {
    type PredicateSummary = ();
    fn compare_probes(
        &self,
        left: &String,
        right: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        compare_text_metered(left, right, ORDER, control)
    }
    fn predicate_summary(
        &self,
        _: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1).map_err(Into::into)
    }
    fn predicate_region_may_match(
        &self,
        _: &(),
        extrema: [&String; 2],
        row: WorthQueryManagedIndexSourceRow<'_, DaymarkSchema, TitlePrefixIndex>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_watch_bounds_may_match_field(extrema, &row, task_title(), ORDER, control)
    }
}
