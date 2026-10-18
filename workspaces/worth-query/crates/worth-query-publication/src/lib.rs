//! Derived Query publication authority.
//!
//! Execution produces governed evidence. Publication owns its derived,
//! policy-materialized decision attachments and canonical disclosure copies.
//!
//! Ordinary fixed/discovered operations retain the authentic installed runtime
//! allocation and complete resolved action through pending, refused, settled and
//! terminal custody. Construction returns an owner without borrowing that runtime;
//! each advance, delivery and close still borrows fresh request authority. Native
//! obligations remain move-only. Refused close returns the whole owner, and Drop
//! is abandonment rather than cleanup or crash-durable continuation. Applications
//! use these owners through `worth_query_host::facade::application_entry`.

#![forbid(unsafe_code)]
// The native source-read closure participates in the runtime auto-trait graph.
#![recursion_limit = "256"]

mod application_aftermath;
mod application_authorization;
mod application_entry;
mod domain_computation;
mod publication_digest;

pub mod facade;
