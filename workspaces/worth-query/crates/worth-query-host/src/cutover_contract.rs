//! Public facade cutovers leave one supported ordinary entrance.
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::application_entry::{
//!     WorthQueryOutputCurrentnessDenial,
//!     advanced::{WorthQueryApplicationMutationRequest, WorthQueryProgramOutputCurrentnessDenial},
//! };
//! ```
//!
//! Current native control and complete currentness diagnostics remain available.
//! ```
//! use worth_query_host::facade::application_entry::{
//!     WorthQueryOutputCurrentnessDenial, advanced::WorthQueryApplicationMutationRequest,
//! };
//! fn current(denial: WorthQueryOutputCurrentnessDenial) -> WorthQueryOutputCurrentnessDenial { denial }
//! ```
//!
//! The provisional correction facade is removed.
//! ```compile_fail,E0432
//! use worth_query_host::facade::provisional_aftermath;
//! ```
//!
//! Recorded correction enters through the ordinary request.
//! ```
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryApplicationRequest, WorthQueryApplicationCorrections},
//!     domain::ApplicationSchema,
//! };
//! fn corrections<'a, 'p, 's, S: ApplicationSchema>(
//!     request: &WorthQueryApplicationRequest<'a, 'p, 's, S>,
//! ) -> WorthQueryApplicationCorrections<'a, 'p, 's, S> {
//!     request.corrections()
//! }
//! ```
