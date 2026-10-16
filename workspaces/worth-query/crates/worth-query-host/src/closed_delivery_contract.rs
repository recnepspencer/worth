//! Compiler-visible original complete-tree custody after native close.
//!
//! Positive handoffs escape the request borrow. Only an actual settled owner or
//! its close-only refusal can produce the closed stage; fresh delivery borrows it.
//! ```
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn close<S,P,R>(owner: WorthQuerySettledProgramOutputOperation<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//!     -> Result<WorthQueryClosedProgramOutputDelivery<S,P,R>, WorthQueryProgramOutputDeliveryCloseRefusal<S,P,R>>
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     owner.close_for_delivery(request)
//! }
//! fn retry<S,P,R>(owner: WorthQueryProgramOutputDeliveryCloseRefusal<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//!     -> Result<WorthQueryClosedProgramOutputDelivery<S,P,R>, WorthQueryProgramOutputDeliveryCloseRefusal<S,P,R>>
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     owner.retry_close(request)
//! }
//! fn deliver<S,P,R>(owner: &WorthQueryClosedProgramOutputDelivery<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _ = owner.deliver_current(request, policy);
//!     let _ = (owner.settlement().root_settlement(), owner.close_evidence());
//! }
//! ```
//!
//! Even authentic cleanup-only evidence cannot construct a closed delivery owner.
//! ```compile_fail,E0451
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn mint<S,P,R>(closed: WorthQueryClosedProgramOutputRequest) -> WorthQueryClosedProgramOutputDelivery<S,P,R>
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     WorthQueryClosedProgramOutputDelivery { closed, delivery: todo!() }
//! }
//! ```
//!
//! A discharged tree cannot resume business advancement.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn advance<S,P,R>(owner: WorthQueryClosedProgramOutputDelivery<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _ = owner.advance(request);
//! }
//! ```
//!
//! The closed stage cannot run cleanup again.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn reclose<S,P,R>(owner: WorthQueryClosedProgramOutputDelivery<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _ = owner.close(request);
//! }
//! ```
//!
//! A partially discharged positive tree is close-only, not deliverable.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn partial<S,P,R>(owner: WorthQueryProgramOutputDeliveryCloseRefusal<S,P,R>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
//! where S: ApplicationSchema + 'static, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _ = owner.deliver_current(request, policy);
//! }
//! ```
//!
//! Closed delivery does not duplicate the original readonly custody.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn clone<S,P,R>(owner: WorthQueryClosedProgramOutputDelivery<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _ = owner.clone();
//! }
//! ```
//!
//! Closed evidence cannot be converted back into a live settled tree.
//! ```compile_fail,E0308
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_program::*, application_schema::ApplicationSchema};
//! use worth_query_host::facade::primary_graph::WorthQueryApplicationRequiredOutputConnection;
//! fn reopen<S,P,R>(owner: WorthQueryClosedProgramOutputDelivery<S,P,R>)
//! where S: ApplicationSchema, P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S> {
//!     let _: WorthQuerySettledProgramOutputOperation<S,P,R> = owner;
//! }
//! ```
mod fixed;
