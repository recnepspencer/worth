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
//!
//! Caller constants matching internal macro names (`__worth_managed_index_setup`,
//! `__worth_managed_index_install`) cannot break a `#[derive(WorthManagedIndex)]`
//! declaration with `kind = text_prefix(...)`:
//!
//! ```
//! use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
//! use worth_query_decl::facade::application_query::*;
//! use worth_query_decl::facade::application_schema::{fragment::*, *};
//! use worth_query_decl::facade::{
//!     authentication::*, worth_query_identity_namespace, worth_query_native_scalar_bindings,
//!     worth_query_structured_value_binding, WorthManagedIndex, WorthQuery, WorthQueryProjection,
//!     WorthQueryRecord, WorthQuerySchema,
//! };
//! use worth_query_host::facade::application_contribution::WorthQueryApplicationManagedIndexRegistration;
//!
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_managed_index_setup: usize = 0;
//! #[allow(dead_code, non_upper_case_globals)]
//! const __worth_managed_index_install: usize = 0;
//!
//! const S_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.mi_hyg.src.v1");
//! const R_NS: ApplicationIdentityNamespace = worth_query_identity_namespace!("doc.mi_hyg.row.v1");
//!
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(schema, owner = "doc.mi_hyg", name = "S", members(F))]
//! pub struct S;
//! #[derive(Clone, Copy, WorthQuerySchema)]
//! #[worth_query(fragment, bindings = C, members(M, P, E))]
//! pub struct F;
//! worth_query_native_scalar_bindings!(pub C);
//! impl Scalar<WorthQueryExternalPrincipalIdentity> for C {
//!     type Binding = WorthQueryExternalPrincipalIdentityBinding;
//!     const WRITABLE: bool = false;
//! }
//! impl Scalar<WorthQueryPrincipalMappingStatus> for C {
//!     type Binding = WorthQueryPrincipalMappingStatusBinding;
//! }
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(MA), relations(MT))] pub struct M;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(PA))] pub struct P;
//! #[derive(Clone, Copy, WorthQuerySchema)] #[worth_query(entity, aspects(EA), reference(S, F))] pub struct E;
//! #[derive(WorthQuerySchema)] #[worth_query(relation, to = P, from = M, reference(S, F))] pub struct MT;
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = MA, fragment = F, entity = M)]
//! pub struct MRow { #[worth_query(equality)] pub ext: WorthQueryExternalPrincipalIdentity, #[worth_query(equality)] pub st: WorthQueryPrincipalMappingStatus }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = PA, fragment = F, entity = P)]
//! pub struct PRow { #[worth_query(equality, read_only)] pub pid: u64 }
//! #[derive(WorthQueryRecord)] #[worth_query(bindings = C, aspect = EA, fragment = F, entity = E)]
//! pub struct ERow { #[worth_query(equality, read_only)] pub k: u64, pub title: String }
//! #[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
//! #[worth_query(schema = S, root = E, namespace = R_NS)]
//! pub struct Row { #[worth_query(field = K)] pub k: u64, #[worth_query(field = Title)] pub title: String }
//! worth_query_structured_value_binding!(pub RowB for Row { identity: "doc.mi_hyg.rowb.v1" });
//! worth_query_structured_value_binding!(pub DenB for () { identity: "doc.mi_hyg.denb.v1" });
//! pub struct PB;
//! pub struct Ctx;
//! impl ApplicationBindingContext<S> for Ctx {
//!     type PrincipalBinding = PB; type Mapping = M; type Principal = P;
//!     type PrincipalIdentity = u64; type PrincipalIdentityBinding = U64ApplicationValueBinding;
//!     type IdempotencyKey = u64; type DenialBinding = DenB;
//!     const IDEMPOTENCY_IDENTITY: &'static str = "doc.mi_hyg.idem.v1";
//!     fn principal_binding() -> ApplicationPrincipalBindingRef<S, PB, M, P, u64, U64ApplicationValueBinding> { unimplemented!() }
//! }
//! #[derive(Clone, Debug, WorthQuery)]
//! #[worth_query(schema = S, context = Ctx, namespace = S_NS, result = RowB, root = E, scope = K, limits = 8)]
//! pub struct Src { #[worth_query(scope)] pub k: u64 }
//! #[derive(WorthManagedIndex)]
//! #[worth_index(
//!     schema = S,
//!     record = E,
//!     source = SrcBinding,
//!     probe = String,
//!     kind = text_prefix(Row::title()),
//!     namespace = "doc.mi_hyg.title_prefix",
//!     version = 1,
//! )]
//! pub struct TitleIdx;
//!
//! fn assert_registered<I: WorthQueryApplicationManagedIndexRegistration<S>>() {}
//! assert_registered::<TitleIdx>();
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
