//! Ordinary workflow owner acceptance consumes the sealed bound view.
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::*,
//!     declaration::{application_operation::*, application_program::ApplicationWorkflowSpec},
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryRecoverySafeRetryAdmission,
//! };
//! fn accept<S, W, O, Input, Scope, I, Preparation>(
//!     advance: WorthQueryWorkflowAdvanceRequest<'_, '_, '_, S, W, O, Input, Scope>,
//!     required: &RequiredWorkflowOperation,
//!     operation: WorthQueryBoundWorkflowOperation<'_, '_, '_, '_, S, I, Preparation>,
//!     recovery: Option<&WorthQueryRecoverySafeRetryAdmission>,
//! ) where
//!     S: ApplicationSchema,
//!     W: ApplicationWorkflowSpec<Schema = S>,
//!     O: 'static,
//!     Input: Clone + Send + Sync + 'static,
//!     I: ApplicationMutationIntent<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! {
//!     if let Some(recovery) = recovery {
//!         let _ = advance.accept_recovered_operation_from_owner(required, operation, recovery);
//!     } else {
//!         let _ = advance.accept_operation_from_owner(required, operation);
//!     }
//! }
//! ```
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     application_entry::*,
//!     declaration::{application_operation::*, application_program::ApplicationWorkflowSpec},
//!     domain::ApplicationSchema,
//!     primary_graph::WorthQueryRecoverySafeRetryAdmission,
//! };
//! use worth_query_host::facade::application_entry::advanced::WorthQueryApplicationMutationRequestWithIdempotency;
//! fn accept<S, W, O, Input, Scope, I, Preparation>(
//!     advance: WorthQueryWorkflowAdvanceRequest<'_, '_, '_, S, W, O, Input, Scope>,
//!     required: &RequiredWorkflowOperation,
//!     operation: WorthQueryApplicationMutationRequestWithIdempotency<'_, '_, '_, '_, S, I, Preparation>,
//!     recovery: Option<&WorthQueryRecoverySafeRetryAdmission>,
//! ) where
//!     S: ApplicationSchema,
//!     W: ApplicationWorkflowSpec<Schema = S>,
//!     O: 'static,
//!     Input: Clone + Send + Sync + 'static,
//!     I: ApplicationMutationIntent<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync + 'static,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! {
//!     if let Some(recovery) = recovery {
//!         let _ = advance.accept_recovered_operation_from_owner(required, operation, recovery);
//!     } else {
//!         let _ = advance.accept_operation_from_owner(required, operation);
//!     }
//! }
//! ```
//!
//! The view cannot extract the native request to regain manual controls.
//! ```compile_fail,E0616
//! use worth_query_host::facade::{application_entry::WorthQueryBoundWorkflowOperation,
//!     domain::ApplicationSchema, declaration::application_operation::ApplicationMutationIntent};
//! fn escape<S: ApplicationSchema, I: ApplicationMutationIntent<S>, P>(
//!     operation: WorthQueryBoundWorkflowOperation<'_, '_, '_, '_, S, I, P>,
//! ) { let _ = operation.native; }
//! ```
