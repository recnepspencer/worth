//! Standard checked ordered managed-index adapter for Query host consumers.
//!
//! Implement [`WorthQueryOrderedIndexDomain`] (and optionally
//! [`WorthQueryOrderedIndexPredicateDomain`] for inverse watch routing) with only
//! metered ordering, associative summary enclosure, and exact probe matching; wrap
//! it in [`WorthQueryOrderedIndexAdapter::new`] to install a checked
//! `BinarySourceTree` + `CheckedSummarySourceTree` provider.
//!
//! Node coordinates and summaries stay opaque to callers; forging a node fails to compile:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::ordered_index::WorthQueryOrderedIndexNode;
//! fn forge(summary: ()) -> WorthQueryOrderedIndexNode<()> {
//!     WorthQueryOrderedIndexNode {
//!         source: todo!(), own_summary: summary, summary, left: None, right: None, height: 1,
//!     }
//! }
//! ```
//!
//! Heap-backed (`!Copy`) summaries cannot satisfy [`WorthQueryOrderedIndexDomain::Summary`],
//! preventing uncharged heap copies during tree maintenance:
//!
//! ```compile_fail,E0277
//! use std::{cmp::Ordering, convert::Infallible};
//! use worth_query_host::facade::declaration::{
//!     application_query::ApplicationManagedIndexBinding,
//!     application_schema::ApplicationSchema,
//! };
//! use worth_query_host::facade::ordered_index::{
//!     WorthQueryOrderedIndexDomain, WorthQueryOrderedIndexSources,
//! };
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryManagedIndexControl, WorthQueryManagedIndexProbeValue,
//!     WorthQueryManagedIndexProviderStop, WorthQueryManagedIndexSourceRow,
//!     WorthQueryManagedIndexSourceSlot,
//! };
//!
//! struct HeapSummaryDomain;
//!
//! impl<S, I> WorthQueryOrderedIndexDomain<S, I> for HeapSummaryDomain
//! where
//!     S: ApplicationSchema,
//!     I: ApplicationManagedIndexBinding<S>,
//! {
//!     type Summary = String;
//!     type Denial = Infallible;
//!     const IDENTITY: &'static str = "heap_summary";
//!     const VERSION: u32 = 1;
//!
//!     fn compare_rows(
//!         &self,
//!         _: WorthQueryManagedIndexSourceRow<'_, S, I>,
//!         _: WorthQueryManagedIndexSourceRow<'_, S, I>,
//!         _: &WorthQueryManagedIndexControl<'_>,
//!     ) -> Result<Ordering, WorthQueryManagedIndexProviderStop<Self::Denial>> {
//!         todo!()
//!     }
//!     fn source_summary(
//!         &self,
//!         _: WorthQueryManagedIndexSourceSlot,
//!         _: WorthQueryManagedIndexSourceRow<'_, S, I>,
//!         _: &WorthQueryManagedIndexControl<'_>,
//!     ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
//!         todo!()
//!     }
//!     fn combine_summaries(
//!         &self,
//!         _: &Self::Summary,
//!         _: &Self::Summary,
//!         _: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
//!         _: &WorthQueryManagedIndexControl<'_>,
//!     ) -> Result<Self::Summary, WorthQueryManagedIndexProviderStop<Self::Denial>> {
//!         todo!()
//!     }
//!     fn summary_may_match(
//!         &self,
//!         _: &Self::Summary,
//!         _: &WorthQueryOrderedIndexSources<'_, '_, S, I>,
//!         _: &WorthQueryManagedIndexProbeValue<S, I>,
//!         _: &WorthQueryManagedIndexControl<'_>,
//!     ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
//!         todo!()
//!     }
//!     fn matches_source(
//!         &self,
//!         _: WorthQueryManagedIndexSourceRow<'_, S, I>,
//!         _: &WorthQueryManagedIndexProbeValue<S, I>,
//!         _: &WorthQueryManagedIndexControl<'_>,
//!     ) -> Result<bool, WorthQueryManagedIndexProviderStop<Self::Denial>> {
//!         todo!()
//!     }
//! }
//! ```
//!
//! The valid counterpart wraps a domain implementation in [`WorthQueryOrderedIndexAdapter`]:
//!
//! ```
//! use worth_query_host::facade::ordered_index::{
//!     WorthQueryOrderedIndexAdapter, WorthQueryOrderedIndexDomain,
//!     WorthQueryOrderedIndexPredicateDomain, WorthQueryOrderedSlotBounds,
//!     WorthQueryOrderedTextOrder,
//! };
//! use worth_query_host::facade::primary_graph::{
//!     WorthQueryManagedIndexPredicateProvider, WorthQueryManagedIndexProvider,
//! };
//! use worth_query_host::facade::declaration::{
//!     application_query::ApplicationManagedIndexBinding,
//!     application_schema::ApplicationSchema,
//! };
//!
//! fn check_adapter<S, I, D>(domain: D) -> impl WorthQueryManagedIndexProvider<S, I>
//!     + WorthQueryManagedIndexPredicateProvider<S, I>
//! where
//!     S: ApplicationSchema,
//!     I: ApplicationManagedIndexBinding<S>,
//!     D: WorthQueryOrderedIndexPredicateDomain<S, I>,
//! {
//!     let _ = WorthQueryOrderedTextOrder::AsciiCaseInsensitive;
//!     let _ = std::any::TypeId::of::<WorthQueryOrderedSlotBounds>();
//!     WorthQueryOrderedIndexAdapter::new(domain)
//! }
//! ```

pub use crate::ordered_index::{
    compare_text_fields, compare_text_metered, map_projection_denial, map_text_read_denial,
    prefix_bounds_may_match_field, prefix_bounds_may_match_metered, prefix_matches_text_field,
    prefix_order_metered, prefix_watch_bounds_may_match_field,
    prefix_watch_bounds_may_match_metered, read_ordered_field, WorthQueryOrderedIndexAdapter,
    WorthQueryOrderedIndexDomain, WorthQueryOrderedIndexNode,
    WorthQueryOrderedIndexPredicateDomain, WorthQueryOrderedIndexSources,
    WorthQueryOrderedPrefixOrder, WorthQueryOrderedSlotBounds, WorthQueryOrderedTextOrder,
};
