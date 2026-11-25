//! Stock checked text-prefix managed-index adapter built on [`WorthQueryOrderedIndexAdapter`].
use super::bounds::{
    compare_text_fields, compare_text_metered, prefix_bounds_may_match_field,
    prefix_matches_text_field, prefix_watch_bounds_may_match_field, WorthQueryOrderedSlotBounds,
    WorthQueryOrderedTextOrder,
};
use super::coord::WorthQueryStockIndexDenial;
use super::{
    WorthQueryOrderedIndexAdapter, WorthQueryOrderedIndexDomain,
    WorthQueryOrderedIndexPredicateDomain, WorthQueryOrderedIndexSources,
};
use std::cmp::Ordering;
use std::marker::PhantomData;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationManagedIndexBinding, ApplicationQueryBinding, ApplicationQueryResultFieldRef,
    },
    application_schema::{
        fragment::RecordSelection, ApplicationReadableScalarValueBinding, ApplicationSchema,
    },
    portable_identity::WorthQueryPortableType,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryManagedIndexControl, WorthQueryManagedIndexProviderStop,
    WorthQueryManagedIndexSourceRow, WorthQueryManagedIndexSourceSlot,
};

type SourceQuery<S, I> =
    <<I as ApplicationManagedIndexBinding<S>>::SourceBinding as ApplicationQueryBinding<S>>::Query;

/// Stock checked text-prefix domain over one declared `String` field and native `String` probe.
pub struct WorthQueryTextPrefixDomain<S, I, FieldSlot, Field: RecordSelection>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    field: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    order: WorthQueryOrderedTextOrder,
    _marker: PhantomData<fn() -> (S, I)>,
}

/// Stock checked text-prefix managed-index provider with optional inverse watch routing.
pub type WorthQueryTextPrefixIndex<S, I, FieldSlot, Field> =
    WorthQueryOrderedIndexAdapter<WorthQueryTextPrefixDomain<S, I, FieldSlot, Field>>;

impl<S, I, FieldSlot, Field: RecordSelection> WorthQueryTextPrefixIndex<S, I, FieldSlot, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    /// Creates an exact UTF-8 byte-ordered prefix index on `field`.
    pub const fn exact_utf8(
        field: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    ) -> Self {
        Self::new_with_order(field, WorthQueryOrderedTextOrder::ExactUtf8)
    }

    /// Creates an ASCII case-insensitive prefix index on `field`.
    pub const fn ascii_case_insensitive(
        field: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
    ) -> Self {
        Self::new_with_order(field, WorthQueryOrderedTextOrder::AsciiCaseInsensitive)
    }

    /// Creates a prefix index on `field` with the given normalization policy.
    pub const fn new_with_order(
        field: ApplicationQueryResultFieldRef<SourceQuery<S, I>, FieldSlot, S, Field>,
        order: WorthQueryOrderedTextOrder,
    ) -> Self {
        WorthQueryOrderedIndexAdapter::new(WorthQueryTextPrefixDomain {
            field,
            order,
            _marker: PhantomData,
        })
    }
}

impl<S, I, FieldSlot, Entity, Aspect, Field> WorthQueryOrderedIndexDomain<S, I>
    for WorthQueryTextPrefixDomain<S, I, FieldSlot, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    I::ProbeBinding: ApplicationReadableScalarValueBinding<Value = String>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect> + 'static,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType + 'static,
{
    type Summary = WorthQueryOrderedSlotBounds;
    type Denial = WorthQueryStockIndexDenial;
    const IDENTITY: &'static str = I::PROVIDER;
    const VERSION: u32 = I::PROVIDER_VERSION;

    fn compare_rows(
        &self,
        left: WorthQueryManagedIndexSourceRow<'_, S, I>,
        right: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        compare_text_fields(&left, &right, self.field, self.order, control)
    }

    fn source_summary(
        &self,
        slot: WorthQueryManagedIndexSourceSlot,
        _row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1)?;
        Ok(WorthQueryOrderedSlotBounds::point(slot))
    }

    fn combine_summaries(
        &self,
        left: &Self::Summary,
        right: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        left.merge_with(right, sources, control, |a, b, c| {
            self.compare_rows(a, b, c)
        })
    }

    fn summary_may_match(
        &self,
        summary: &Self::Summary,
        sources: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_bounds_may_match_field(summary, sources, self.field, probe, self.order, control)
    }

    fn matches_source(
        &self,
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_matches_text_field(&row, self.field, probe, self.order, control)
    }
}

impl<S, I, FieldSlot, Entity, Aspect, Field> WorthQueryOrderedIndexPredicateDomain<S, I>
    for WorthQueryTextPrefixDomain<S, I, FieldSlot, Field>
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
    I::ProbeBinding: ApplicationReadableScalarValueBinding<Value = String>,
    Field: RecordSelection<Entity = Entity, Aspect = Aspect> + 'static,
    Field::Binding: ApplicationReadableScalarValueBinding,
    Field::Value: Into<String>,
    FieldSlot: WorthQueryPortableType + 'static,
{
    type PredicateSummary = ();

    fn compare_probes(
        &self,
        left: &String,
        right: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        compare_text_metered(left, right, self.order, control)
    }

    fn predicate_summary(
        &self,
        _probe: &String,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<(), WorthQueryManagedIndexProviderStop<Self::Denial>> {
        control.checkpoint(1).map_err(Into::into)
    }

    fn predicate_region_may_match(
        &self,
        _summary: &(),
        extrema: [&String; 2],
        row: WorthQueryManagedIndexSourceRow<'_, S, I>,
        control: &WorthQueryManagedIndexControl<'_>,
    ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
        prefix_watch_bounds_may_match_field(extrema, &row, self.field, self.order, control)
    }
}
