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
//!
//! Effect-free construction refusal can release its runtime borrow without
//! losing the original description or issued workflow requirement.
//! ```
//! use worth_query_host::facade::{application_entry::*, domain::ApplicationSchema};
//! use worth_query_decl::facade::application_operation::ApplicationMutationIntent;
//! fn retain_original<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
//!     binding: WorthQueryOwnedWorkflowBinding<'_, S, I>,
//! ) -> (WorthQueryOperationDescription<S, I>, RequiredWorkflowOperation) {
//!     binding.into_parts()
//! }
//! ```
//!
//! Birth has its own issued authority, not a current-operation requirement.
//! Binding uses the frozen original key/input; this step performs no mutation.
//! ```
//! use worth_query_host::facade::{application_entry::*, application_installation::WorthQueryWorkflowApplicationRuntime, domain::ApplicationSchema};
//! use worth_query_decl::facade::{application_operation::*, application_program::*};
//! fn bind_birth<'w, S, I, W, P, R>(
//!     original: WorthQueryOperationDescription<S, I>,
//!     workflow: &'w WorthQueryWorkflowApplicationRuntime<S, W, P>,
//!     required: &RequiredWorkflowBirth,
//! ) -> Result<WorthQueryOwnedWorkflowBirthBinding<'w, S, I>, (WorthQueryOperationDescription<S, I>, WorthQueryWorkflowBirthBindingDenial)>
//! where S: ApplicationSchema, I: ApplicationMutationIntent<S>,
//!     W: ApplicationWorkflowSpec<Schema = S>, P: ApplicationProgramDefinition<S>,
//!     R: WorthQueryApplicationOutputRole<Schema = S, Contract = <I::Binding as ApplicationMutationBinding<S>>::Output, Action = WorthQueryCreateOutput>,
//! { original.for_workflow_birth::<W, R>(workflow, required) }
//! ```
//!
//! The move-only binding cannot duplicate original command custody.
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::*, domain::ApplicationSchema};
//! use worth_query_decl::facade::application_operation::ApplicationMutationIntent;
//! fn duplicate<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
//!     binding: WorthQueryOwnedWorkflowBirthBinding<'_, S, I>,
//! ) { let _ = binding.clone(); }
//! ```
//!
//! A construction refusal can lawfully return the original parts, without spending the slot.
//! ```
//! use worth_query_host::facade::{application_entry::*, domain::ApplicationSchema};
//! use worth_query_decl::facade::application_operation::ApplicationMutationIntent;
//! fn retain_birth_parts<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
//!     binding: WorthQueryOwnedWorkflowBirthBinding<'_, S, I>,
//! ) -> (WorthQueryOperationDescription<S, I>, RequiredWorkflowBirth) {
//!     binding.into_parts()
//! }
//! ```
