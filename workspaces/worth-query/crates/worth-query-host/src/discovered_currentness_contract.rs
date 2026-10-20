//! Native aggregate guard accepts only a completed owning aggregate, never a
//! caller-selected root iterator or a latest-observation proxy.
//!
//! ```no_run
//! use std::num::NonZeroUsize;
//! use worth_query_host::facade::{application_entry::{WorthQueryApplicationRetainedRequest,
//!     WorthQueryDiscoveredProgramOutputSettlement, WorthQueryOutputCurrentnessDenial}, domain::ApplicationSchema};
//! fn complete<S: ApplicationSchema, Q, D>(
//!     request: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
//!     original: &WorthQueryDiscoveredProgramOutputSettlement<Q, D>, work: NonZeroUsize,
//! ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
//!     request.require_current_discovered_program_outputs(original, Some((work).get()))
//! }
//! ```
//!
//! ```compile_fail,E0308
//! use std::num::NonZeroUsize;
//! use worth_query_host::facade::{application_entry::{WorthQueryApplicationRetainedRequest,
//!     WorthQueryDiscoveredProgramOutputSettlement, WorthQueryOutputCurrentnessDenial}, domain::ApplicationSchema};
//! fn subset<S: ApplicationSchema, Q, D>(
//!     request: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
//!     original: &WorthQueryDiscoveredProgramOutputSettlement<Q, D>, work: NonZeroUsize,
//! ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
//!     request.require_current_discovered_program_outputs(original.root_outputs(), Some((work).get()))
//! }
//! ```
//!
//! ```compile_fail,E0308
//! use std::num::NonZeroUsize;
//! use worth_query_host::facade::{application_entry::{WorthQueryApplicationRetainedRequest,
//!     WorthQueryDiscoveredProgramOutputSettlement, WorthQueryOutputCurrentnessDenial}, domain::ApplicationSchema};
//! fn observation_is_not_proof<S: ApplicationSchema, Q, D>(
//!     request: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
//!     original: &WorthQueryDiscoveredProgramOutputSettlement<Q, D>, work: NonZeroUsize,
//! ) -> Result<(), WorthQueryOutputCurrentnessDenial> {
//!     request.require_current_discovered_program_outputs(original.observation(), Some((work).get()))
//! }
//! ```
