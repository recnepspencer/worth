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
//! Application Home has no flat inverse undo entry point.
//! ```compile_fail,E0599
//! use worth_query_decl::facade::application_program::ApplicationProgramDefinition;
//! use worth_query_decl::facade::application_schema::ApplicationSchema;
//! use worth_query_host::facade::WorthQueryApplicationHome;
//!
//! fn invoke<Schema, Program>()
//! where
//!     Schema: ApplicationSchema + 'static,
//!     Program: ApplicationProgramDefinition<Schema> + 'static,
//! {
//!     let _ = WorthQueryApplicationHome::<Schema, Program>::undo;
//! }
//! ```
