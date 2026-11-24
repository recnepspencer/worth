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
    prefix_watch_bounds_may_match_metered, read_ordered_field, WorthQueryAxisEnvelope,
    WorthQueryBox2D, WorthQueryBox2DBinding, WorthQueryBox2DDomain, WorthQueryBox2DEnvelope,
    WorthQueryBox2DF64, WorthQueryBox2DF64Binding, WorthQueryBox2DIndex, WorthQueryBox2DProbe,
    WorthQueryBox3D, WorthQueryBox3DBinding, WorthQueryBox3DDomain, WorthQueryBox3DEnvelope,
    WorthQueryBox3DF64, WorthQueryBox3DF64Binding, WorthQueryBox3DIndex, WorthQueryBox3DProbe,
    WorthQueryInterval1D, WorthQueryInterval1DBinding, WorthQueryInterval1DDomain,
    WorthQueryInterval1DEnvelope, WorthQueryInterval1DIndex, WorthQueryInterval1DProbe,
    WorthQueryOrderedCoord, WorthQueryOrderedIndexAdapter, WorthQueryOrderedIndexDomain,
    WorthQueryOrderedIndexNode, WorthQueryOrderedIndexPredicateDomain,
    WorthQueryOrderedIndexSources, WorthQueryOrderedPrefixOrder, WorthQueryOrderedSlotBounds,
    WorthQueryOrderedTextOrder, WorthQueryRangeInclusion, WorthQueryStockIndexDenial,
    WorthQueryTextPrefixDomain, WorthQueryTextPrefixIndex,
};
