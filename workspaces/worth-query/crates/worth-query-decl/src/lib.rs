//! Query declaration audience facade.
//!
//! Start with [Build an Application](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md)
//! and the [Capability Index](https://github.com/recnepspencer/worth/blob/master/docs/capability-index.md).
//!
//! Declare application meaning with the unified derive & program macros:
//! - `#[derive(WorthQuerySchema)]`, `#[derive(WorthQueryRecord)]`,
//!   `#[derive(WorthQueryRelation)]`, `#[derive(WorthQueryAction)]`
//! - `#[derive(WorthQueryRow)]` / `#[derive(WorthQueryProjection)]` (directly
//!   usable as `result = MyRow` on queries)
//! - `#[derive(WorthQuery)]` supporting `where_equal` / `where_not_equal` /
//!   `where_lt` / `where_lte` / `where_gt` / `where_gte` (and `*_optional` /
//!   `where_optional` for `Option<T>` parameters), `where_no_related` /
//!   `where_related_exists`, `root_path`, `order_by`, `managed_index`, and
//!   `live = whole_result`
//! - `#[derive(WorthQueryCorrection)]` supporting `mode = recorded_inverse`
//!   and `mode = authored_body`
//! - `#[derive(WorthManagedIndex)]` and `#[derive(WorthBinaryValue)]`
//! - `CanonicalDate::parse_iso`, `add_days`, `start_of_day_utc`, `Display`/`FromStr`
//!
//! Law: ordinary declaration consumers depend on this crate, never on Query's
//! internal authority packages. This crate re-exports declaration types only; it adds no
//! behavior.
//!
//! Blessed import:
//!
//! ```
//! use worth_query_decl::facade::CanonicalQueryArtifact;
//! ```
//!
//! Type identity with the engine (no wrapper drift):
//!
//! ```
//! use worth_query_decl::facade::CanonicalQueryArtifact;
//! # fn _same_type(authority: worth_query_declaration::facade::canonicalization::CanonicalQueryArtifact) -> CanonicalQueryArtifact {
//! #     authority
//! # }
//! ```

mod derive_contract;
pub mod facade;
mod managed_index_derive_contract;
mod naming_derive_contract;
pub mod program_transition;
mod stored_name_derive_contract;
mod unified_query_derive_contract;
mod unified_query_literal_contract;

#[cfg(test)]
mod canonical_refusal_tests;
