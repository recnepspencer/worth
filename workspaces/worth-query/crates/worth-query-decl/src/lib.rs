//! Query declaration audience facade.
//!
//! Start with [Build an Application](https://github.com/recnepspencer/worth/blob/master/docs/build-an-application.md)
//! and the [Capability Index](https://github.com/recnepspencer/worth/blob/master/docs/capability-index.md).
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

pub mod facade;
