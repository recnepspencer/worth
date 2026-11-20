//! Query certification audience facade (cert-only).
//!
//! Law: only cert-band consumers may depend on this crate. Ordinary bands use
//! `worth-query-decl` / `worth-query-host`. This crate re-exports authority
//! types only; it adds no executors or certification harnesses.
//!
//! Blessed import:
//!
//! ```
//! use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;
//! ```
//!
//! Type identity with the owner (no wrapper drift):
//!
//! ```
//! use worth_query_replay::facade::WorthQueryCertificationCostScope;
//! # fn _same_type(
//! #     owner: worth_query_execution::facade::certification::WorthQueryCertificationCostScope,
//! # ) -> WorthQueryCertificationCostScope {
//! #     owner
//! # }
//! ```

pub mod facade;
