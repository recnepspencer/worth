//! Bundled bounds and source custody keep every fence of the direct Host calls.
//!
//! Generic helpers name one bound per intent and one per action. Custody
//! survives the request frame; no request, principal or phase is retained:
//!
//! ```
//! use worth_query_host::facade::{
//!     application_entry::{
//!         WorthQueryApplicationRequest, WorthQueryProjectedQuery,
//!         WorthQuerySourceCompletionAction, WorthQuerySourcelessMutation,
//!     },
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{
//!         application_operation::ApplicationMutationBinding,
//!         application_program::ApplicationProgramDefinition,
//!     },
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody, WorthQuerySourceCustodyStep,
//! };
//! type Key<S, I> = <<I as worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::IdempotencyKey;
//! fn submit<S, P, I, A>(
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//!     intent: I,
//!     key: Key<S, I>,
//! ) -> Option<WorthQuerySourceCustodyStep<S, I, P, A>>
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourcelessMutation<S>,
//!     A: WorthQuerySourceCompletionAction<S, I::Binding>,
//! {
//!     let described = request.mutate(intent).without_source().idempotency_owned(key).ok()?;
//!     Some(WorthQuerySourceCustody::Described(described).resume(runtime, request))
//! }
//! fn next_frame<S, P, I, A>(
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     fresh: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//!     step: WorthQuerySourceCustodyStep<S, I, P, A>,
//! ) -> WorthQuerySourceCustodyStep<S, I, P, A>
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourcelessMutation<S>,
//!     A: WorthQuerySourceCompletionAction<S, I::Binding>,
//! {
//!     match step {
//!         WorthQuerySourceCustodyStep::Pending { custody, .. } => custody.resume(runtime, fresh),
//!         settled => settled,
//!     }
//! }
//! fn read<S: ApplicationSchema, Q: WorthQueryProjectedQuery<S>>(
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//!     intent: Q,
//! ) -> usize {
//!     request.query(intent).execute().map_or(0, |result| result.rows().len())
//! }
//! ```
//!
//! An action whose declared delivery does not complete the source cannot
//! drive custody, exactly as it cannot retain a source-completion operation:
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryApplicationRequest, WorthQuerySourceMutation},
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::{ApplicationAction, ApplicationProgramDefinition},
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn any_delivery<S, P, I, A>(
//!     custody: WorthQuerySourceCustody<S, I, P, A>,
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourceMutation<S>,
//!     A: ApplicationAction<S, I::Binding>,
//! {
//!     let _ = custody.resume(runtime, request);
//! }
//! ```
//!
//! A bare intent bound carries none of the binding's native requirements:
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryApplicationRequest, WorthQuerySourceCompletionAction},
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{
//!         application_operation::ApplicationMutationIntent,
//!         application_program::ApplicationProgramDefinition,
//!     },
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn bare_intent<S, P, I, A>(
//!     custody: WorthQuerySourceCustody<S, I, P, A>,
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: ApplicationMutationIntent<S> + 'static,
//!     A: WorthQuerySourceCompletionAction<S, I::Binding>,
//! {
//!     let _ = custody.resume(runtime, request);
//! }
//! ```
//!
//! Nor does a bare query intent execute:
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{
//!     application_entry::WorthQueryApplicationRequest,
//!     declaration::application_query::ApplicationQueryIntent, domain::ApplicationSchema,
//! };
//! fn bare_query<S: ApplicationSchema, Q: ApplicationQueryIntent<S>>(
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//!     intent: Q,
//! ) {
//!     let _ = request.query(intent).execute();
//! }
//! ```
//!
//! Custody of one schema cannot be resumed with another schema's request:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     application_entry::{
//!         WorthQueryApplicationRequest, WorthQuerySourceCompletionAction, WorthQuerySourceMutation,
//!     },
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition,
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn foreign_schema<S, T, P, I, A>(
//!     custody: WorthQuerySourceCustody<S, I, P, A>,
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     foreign: &WorthQueryApplicationRequest<'_, '_, '_, T>,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     T: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourceMutation<S>,
//!     A: WorthQuerySourceCompletionAction<S, I::Binding>,
//! {
//!     let _ = custody.resume(runtime, foreign);
//! }
//! ```
//!
//! Completed custody can only close; it cannot re-enter the advancing phase:
//!
//! ```compile_fail,E0308
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryCompletedSourceOperation, WorthQuerySourceMutation},
//!     declaration::application_program::{
//!         ApplicationProgramDefinition, ApplicationSourceCompletion,
//!     },
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn re_advance<S, P, I, A>(
//!     completed: WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
//! ) -> WorthQuerySourceCustody<S, I, P, A>
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourceMutation<S>,
//! {
//!     WorthQuerySourceCustody::Retained(completed)
//! }
//! ```
//!
//! Custody phases hold only Publication-issued owners; none can be assembled:
//!
//! ```compile_fail,E0451
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryCompletedSourceOperation, WorthQuerySourceOnlyOperation},
//!     declaration::application_program::{
//!         ApplicationProgramDefinition, ApplicationSourceCompletion,
//!     },
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn forge<S, P, I, A>(
//!     operation: WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>,
//! ) -> WorthQuerySourceCustody<S, I, P, A>
//! where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: worth_query_host::facade::declaration::application_operation::ApplicationMutationIntent<S>,
//! {
//!     WorthQuerySourceCustody::Completed(WorthQueryCompletedSourceOperation { operation })
//! }
//! ```
//!
//! Resume consumes custody; a pending owner is resumed only from its step:
//!
//! ```compile_fail,E0382
//! use worth_query_host::facade::{
//!     application_entry::{
//!         WorthQueryApplicationRequest, WorthQuerySourceCompletionAction, WorthQuerySourceMutation,
//!     },
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::application_program::ApplicationProgramDefinition,
//!     domain::ApplicationSchema,
//!     WorthQuerySourceCustody,
//! };
//! fn resume_twice<S, P, I, A>(
//!     custody: WorthQuerySourceCustody<S, I, P, A>,
//!     runtime: &WorthQueryProgramApplicationRuntime<S, P>,
//!     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
//! ) where
//!     S: ApplicationSchema + 'static,
//!     P: ApplicationProgramDefinition<S>,
//!     I: WorthQuerySourceMutation<S>,
//!     A: WorthQuerySourceCompletionAction<S, I::Binding>,
//! {
//!     let _ = custody.resume(runtime, request);
//!     let _ = custody.resume(runtime, request);
//! }
//! ```
