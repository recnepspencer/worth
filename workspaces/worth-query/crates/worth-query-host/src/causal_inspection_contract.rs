//! Source-rich inspection enters only through actual current diagnostic admission.
//!
//! The ordinary Host facade accepts independent metadata and diagnostic capabilities.
//! This signature returns an owned result without borrowing the original receipt.
//! ```
//! use worth_query_host::facade::{
//!     causal_inspection::{WorthQueryMaterializedSourceInspectionRequest,
//!         WorthQueryMaterializedPerformedSourceInspection,WorthQueryApplicationInspectionDenial},
//!     primary_graph::WorthQueryAdmittedApplicationCapabilityAccess,
//!     declaration::{application_schema::ApplicationSchema,
//!         application_operation::ApplicationSourceDisclosureBinding,
//!         application_capability::ApplicationCapabilityRequest,
//!         application_query::{ApplicationQueryBinding,ApplicationQueryScopeBinding}},
//! };
//! type SourceScope<S,D> = <<<D as ApplicationSourceDisclosureBinding<S>>::Disclosure
//!     as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope;
//! fn admitted<S,D,C,O,I>(
//!     request:WorthQueryMaterializedSourceInspectionRequest<'_,'_,'_,'_,S,D>,
//!     metadata:WorthQueryAdmittedApplicationCapabilityAccess<S,C,O,I>,
//!     diagnostic:WorthQueryAdmittedApplicationCapabilityAccess<S,C,O,I>,
//! ) -> Result<WorthQueryMaterializedPerformedSourceInspection,WorthQueryApplicationInspectionDenial>
//! where S:ApplicationSchema,D:ApplicationSourceDisclosureBinding<S>,
//!     I:ApplicationCapabilityRequest<S,C,Scope=SourceScope<S,D>> {
//!     request.execute_governed(metadata,diagnostic)
//! }
//! ```
//!
//! A rich intent has no diagnostic-free execution door, even for public metadata.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     causal_inspection::WorthQueryMaterializedSourceInspectionRequest,
//!     declaration::{application_schema::ApplicationSchema,
//!         application_operation::ApplicationSourceDisclosureBinding},
//! };
//! fn cannot_skip_diagnostics<S:ApplicationSchema,D:ApplicationSourceDisclosureBinding<S>>(
//!     request:WorthQueryMaterializedSourceInspectionRequest<'_,'_,'_,'_,S,D>,
//! ) {
//!     let _ = request.execute();
//! }
//! ```
//!
//! Source-rich success is published by the consuming inspection terminal, not callers.
//! ```compile_fail,E0451
//! use worth_query_host::facade::causal_inspection::WorthQueryMaterializedPerformedSourceInspection;
//! fn cannot_forge_success() -> WorthQueryMaterializedPerformedSourceInspection {
//!     WorthQueryMaterializedPerformedSourceInspection {
//!         original:panic!(), report:panic!(), materialization:panic!(),
//!         identity:panic!(), _memory:None,
//!     }
//! }
//! ```

//! The ordinary inspection namespace cannot reopen the retained engine audience.
//! ```compile_fail,E0432
//! use worth_query_host::facade::causal_inspection::CausalInspection;
//! ```
//! ```compile_fail,E0432
//! use worth_query_host::facade::causal_inspection::ScopedInspectionBasis;
//! ```
//!
//! Admitted conditional history views have no public constructor.
//! ```compile_fail,E0451
//! use worth_query_host::facade::causal_inspection::WorthQueryMaterializedConditionalHistory;
//! fn cannot_mint_history() -> WorthQueryMaterializedConditionalHistory {
//!     WorthQueryMaterializedConditionalHistory { original:panic!(), detail:panic!() }
//! }
//! ```
