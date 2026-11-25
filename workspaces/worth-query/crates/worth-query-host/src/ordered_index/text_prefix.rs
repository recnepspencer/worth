//! Stock checked text-prefix managed-index adapter built on [`WorthQueryOrderedIndexAdapter`].
//!
//! The key may be an Optional field: a row without text is sparse (see
//! [`WorthQueryOrderedTextKeyField`]); it orders before every keyed row and
//! stays outside every summary's bounds.
use super::bounds::{
    compare_text_metered, prefix_bounds_may_match_metered, prefix_order_metered,
    prefix_watch_bounds_may_match_metered, WorthQueryOrderedPrefixOrder,
    WorthQueryOrderedSlotBounds, WorthQueryOrderedTextOrder,
};
use super::coord::WorthQueryStockIndexDenial;
use super::key_field::WorthQueryOrderedTextKeyField;
use super::{
    WorthQueryOrderedIndexAdapter, WorthQueryOrderedIndexDomain,
    WorthQueryOrderedIndexPredicateDomain, WorthQueryOrderedIndexSources,
};
use std::cmp::Ordering;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding,
    application_schema::{ApplicationReadableScalarValueBinding, ApplicationSchema},
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceSlot,
};

type Stop = WorthQueryManagedIndexProviderStop<WorthQueryStockIndexDenial>;

/// Stock checked text-prefix domain over one declared Required or Optional
/// `String` field and a native `String` probe.
pub struct WorthQueryTextPrefixDomain<S, I, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    field: Field,
    order: WorthQueryOrderedTextOrder,
    _marker: PhantomData<fn() -> (S, I)>,
}

/// Stock checked text-prefix managed-index provider with optional inverse watch routing.
pub type WorthQueryTextPrefixIndex<S, I, Field> =
    WorthQueryOrderedIndexAdapter<WorthQueryTextPrefixDomain<S, I, Field>>;

impl<S, I, Field> WorthQueryTextPrefixIndex<S, I, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    /// Creates an exact UTF-8 byte-ordered prefix index on `field`.
    pub const fn exact_utf8(field: Field) -> Self {
        Self::new_with_order(field, WorthQueryOrderedTextOrder::ExactUtf8)
    }

    /// Creates an ASCII case-insensitive prefix index on `field`.
    pub const fn ascii_case_insensitive(field: Field) -> Self {
        Self::new_with_order(field, WorthQueryOrderedTextOrder::AsciiCaseInsensitive)
    }

    /// Creates a prefix index on `field` with the given normalization policy.
    pub const fn new_with_order(field: Field, order: WorthQueryOrderedTextOrder) -> Self {
        WorthQueryOrderedIndexAdapter::new(WorthQueryTextPrefixDomain {
            field,
            order,
            _marker: PhantomData,
        })
    }
}

impl<S, I, Field> WorthQueryOrderedIndexDomain<S, I> for WorthQueryTextPrefixDomain<S, I, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    I::ProbeBinding: ApplicationReadableScalarValueBinding<Value = String>,
    Field: WorthQueryOrderedTextKeyField<S, I>,
{
    /// Slot bounds over the subtree's keyed rows; `None` when it has none.
    type Summary = Option<WorthQueryOrderedSlotBounds>;
    type Denial = WorthQueryStockIndexDenial;
    const IDENTITY: &'static str = I::PROVIDER;
    const VERSION: u32 = I::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, S, I>,
        right: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, Stop> {
        let keyed = self.field.with_key_text(&left, control, |a| {
            let right_keyed = self.field.with_key_text(&right, control, |b| {
                compare_text_metered(a, b, self.order, control)
            })?;
            Ok(right_keyed.unwrap_or(Ordering::Greater))
        })?;
        match keyed {
            Some(ordering) => Ok(ordering),
            None if self.field.key_present(&right, control)? => Ok(Ordering::Less),
            None => Ok(Ordering::Equal),
        }
    }

    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        control.checkpoint(1)?;
        Ok(self
            .field
            .key_present(&row, control)?
            .then_some(WorthQueryOrderedSlotBounds::point(slot)))
    }

    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, Stop> {
        match (left, right) {
            (Some(left), Some(right)) => left
                .merge_with(right, sources, control, |a, b, c| {
                    self.compare_rows(a, b, c)
                })
                .map(Some),
            (keyed, None) | (None, keyed) => {
                control.checkpoint(1)?;
                Ok(*keyed)
            }
        }
    }

    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let Some(bounds) = summary else {
            control.checkpoint(1)?;
            return Ok(false);
        };
        let lower = sources.get(bounds.lower())?;
        let upper = sources.get(bounds.upper())?;
        let may_match = self.field.with_key_text(&lower, control, |lo| {
            let within = self.field.with_key_text(&upper, control, |hi| {
                prefix_bounds_may_match_metered(lo, hi, probe, self.order, control)
            })?;
            // Bounds name keyed rows only; an unkeyed bound cannot prune.
            Ok(within.unwrap_or(true))
        })?;
        Ok(may_match.unwrap_or(true))
    }

    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let order = self.field.with_key_text(&row, control, |text| {
            prefix_order_metered(text, probe, self.order, control)
        })?;
        Ok(order == Some(WorthQueryOrderedPrefixOrder::Matches))
    }
}

impl<S, I, Field> WorthQueryOrderedIndexPredicateDomain<S, I>
    for WorthQueryTextPrefixDomain<S, I, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    I::ProbeBinding: ApplicationReadableScalarValueBinding<Value = String>,
    Field: WorthQueryOrderedTextKeyField<S, I>,
{
    type PredicateSummary = ();

    fn compare_probes(
        &self,
        left: &String,
        right: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, Stop> {
        compare_text_metered(left, right, self.order, control)
    }

    fn predicate_summary(
        &self,
        _probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), Stop> {
        control.checkpoint(1).map_err(Into::into)
    }

    fn predicate_region_may_match(
        &self,
        _summary: &(),
        extrema: [&String; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, Stop> {
        let may_match = self.field.with_key_text(&row, control, |text| {
            prefix_watch_bounds_may_match_metered(
                extrema[0].as_str(),
                extrema[1].as_str(),
                text,
                self.order,
                control,
            )
        })?;
        Ok(may_match.unwrap_or(false))
    }
}
