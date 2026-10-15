//! Removed currentness vocabulary does not survive behind the native audience.
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
