//! Ordinary and advanced entry contracts through the application audience facade.
//!
//! Ordinary construction has no native execution or recovery methods, even after binding a borrowed key.
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryPendingApplicationLane, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryPendingApplicationLane<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.execute_in_program(program, todo!()); }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryPendingApplicationLane, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryPendingApplicationLane<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.prepare_in_program(program, todo!()); }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryPendingApplicationLane, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryPendingApplicationLane<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.recover_unpublished_in_program(todo!(), program); }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryPendingApplicationLane, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryPendingApplicationLane<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.resolve_idempotency_in_program(program); }
//! ```
//!
//! ```compile_fail,E0432
//! use worth_query_host::facade::application_entry::{WorthQueryApplicationMutationRequest, WorthQueryApplicationMutationRequestWithIdempotency};
//! ```
//!
//! ```compile_fail,E0616
//! use worth_query_host::facade::{application_entry::WorthQueryOperationBuilder, domain::ApplicationSchema,
//!     declaration::application_operation::ApplicationMutationIntent};
//! fn escape<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(builder: WorthQueryOperationBuilder<'_, '_, '_, S, I>) {
//!     let native = builder.native;
//! }
//! ```
//!
//! ```
//! use worth_query_host::facade::{application_entry::{WorthQueryApplicationRequest, WorthQueryOperationBuilder,
//!     advanced::WorthQueryApplicationMutationRequest}, domain::ApplicationSchema,
//!     declaration::application_operation::ApplicationMutationIntent};
//! fn entrances<'a, 'p, 's, S: ApplicationSchema, I: ApplicationMutationIntent<S>>(
//!     request: &WorthQueryApplicationRequest<'a, 'p, 's, S>, first: I, second: I,
//! ) {
//!     let ordinary: WorthQueryOperationBuilder<'a, 'p, 's, S, I> = request.mutate(first);
//!     let advanced: WorthQueryApplicationMutationRequest<'a, 'p, 's, S, I> = request.advanced().mutate(second);
//!     drop((ordinary, advanced));
//! }
//! ```
//!
//! This valid native counterpart compile-checks the explicit advanced call.
//! ```
//! use worth_query_host::facade::{application_entry::WorthQueryApplicationRequest,
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition},
//!     runtime::ExecutionAllocationPolicy};
//! fn native<'a, S, I, P>(request: &WorthQueryApplicationRequest<'a, '_, '_, S>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>, intent: I,
//!     key: &<I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey)
//! where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! {
//!     let _ = request.advanced().mutate(intent).idempotency(key)
//!         .execute_in_program(program, ExecutionAllocationPolicy::SystemAllocation);
//! }
//! ```
//!
//! Ordinary no-source construction needs declaration-authorized source meaning.
//! ```compile_fail,E0271
//! use worth_query_host::facade::{application_entry::WorthQueryOperationBuilder, domain::ApplicationSchema,
//!     declaration::application_operation::ApplicationMutationIntent};
//! fn erase_source<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(builder: WorthQueryOperationBuilder<'_, '_, '_, S, I>) {
//!     let _ = builder.without_source();
//! }
//! ```
//! ```
//! use worth_query_host::facade::{application_entry::{WorthQueryOperationBuilder, WorthQueryOperationDescription},
//!     domain::ApplicationSchema, declaration::application_operation::*};
//! fn original<S: ApplicationSchema, I: ApplicationMutationIntent<S>>(builder: WorthQueryOperationBuilder<'_, '_, '_, S, I>,
//!     key: <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey) -> WorthQueryOperationDescription<S, I>
//! where I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync {
//!     builder.without_source().idempotency_owned(key).unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
//! }
//! ```
//!
//! Matching advanced native controls and workflow bindings.
//! ```
//! use worth_query_host::facade::{application_entry::{advanced::WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryApplicationMutationRequestWithIdempotency<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.execute_in_program(program, todo!()); }
//! ```
//!
//! ```
//! use worth_query_host::facade::{application_entry::{advanced::WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryApplicationMutationRequestWithIdempotency<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.prepare_in_program(program, todo!()); }
//! ```
//!
//! ```
//! use worth_query_host::facade::{application_entry::{advanced::WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryApplicationMutationRequestWithIdempotency<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.recover_unpublished_in_program(todo!(), program); }
//! ```
//!
//! ```
//! use worth_query_host::facade::{application_entry::{advanced::WorthQueryApplicationMutationRequestWithIdempotency, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryApplicationMutationRequestWithIdempotency<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.resolve_idempotency_in_program(program); }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryBoundWorkflowOperation, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryBoundWorkflowOperation<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.prepare_in_program(program, todo!()); }
//! ```
//!
//! ```compile_fail,E0599
//! use worth_query_host::facade::{application_entry::{WorthQueryBoundWorkflowOperation, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryBoundWorkflowOperation<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.recover_unpublished_in_program(todo!(), program); }
//! ```
//!
//! ```
//! use worth_query_host::facade::{application_entry::{WorthQueryBoundWorkflowOperation, WorthQueryMutationSourcePrepared},
//!     application_installation::WorthQueryProgramApplicationRuntime, domain::ApplicationSchema,
//!     declaration::{application_operation::*, application_program::ApplicationProgramDefinition}};
//! fn manual<'a, S, I, P>(
//!     mut keyed: WorthQueryBoundWorkflowOperation<'a, '_, '_, '_, S, I, WorthQueryMutationSourcePrepared>,
//!     program: &'a WorthQueryProgramApplicationRuntime<S, P>,
//! ) where S: ApplicationSchema, I: ApplicationMutationIntent<S> + Clone + Send + Sync,
//!     P: ApplicationProgramDefinition<S>,
//!     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
//!     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
//!         ApplicationMutationScopeResolution<S, <I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
//! { let _ = keyed.advanced().execute_in_program(program, todo!()); }
//! ```

//! Fixed and discovered owners outlive the authentic runtime handle used for construction.
//! These compile through the application audience and impose no Clone bound on intent.
//! Applications keep a separate authentic handle for later fresh requests.
//! ```
//! use worth_query_host::facade::{
//!     application_entry::{WorthQueryOperationDescription, WorthQueryOperationConstructionDenial,
//!         WorthQueryFixedOperation, WorthQueryDiscoveredOperation, WorthQueryOutputProjectionInstallation},
//!     application_installation::WorthQueryProgramApplicationRuntime,
//!     declaration::{application_operation::*, application_program::*, application_schema::ApplicationSchema},
//!     primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
//!         WorthQueryApplicationDiscoveredOutputConnection},
//! };
//! fn fixed<S, I, P, R, A>(
//!     description: WorthQueryOperationDescription<S, I>,
//!     constructing: WorthQueryProgramApplicationRuntime<S, P>,
//! ) -> Result<WorthQueryFixedOperation<S, I, P, R, A>,
//!     (WorthQueryOperationDescription<S, I>, WorthQueryOperationConstructionDenial)>
//! where
//!     S: ApplicationSchema + 'static,
//!     I: ApplicationMutationIntent<S>,
//!     P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
//!     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
//!     A: ApplicationAction<S, I::Binding>,
//!     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
//!     <A::Delivery as ApplicationFixedActionDelivery<S, I::Binding>>::Projection: WorthQueryOutputProjectionInstallation<S>,
//! {
//!     let retained = description.retain_operation::<P, A>(&constructing);
//!     drop(constructing);
//!     retained
//! }
//! fn discovered<S, I, P, R, A>(
//!     description: WorthQueryOperationDescription<S, I>,
//!     constructing: WorthQueryProgramApplicationRuntime<S, P>,
//! ) -> Result<WorthQueryDiscoveredOperation<S, I, P, R, A>,
//!     (WorthQueryOperationDescription<S, I>, WorthQueryOperationConstructionDenial)>
//! where
//!     S: ApplicationSchema + 'static,
//!     I: ApplicationMutationIntent<S>,
//!     P: ApplicationProgramDefinition<S>,
//!     R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
//!     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
//!     A: ApplicationAction<S, I::Binding>,
//!     A::Delivery: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
//!     <A::Delivery as ApplicationDiscoveredActionDelivery<S, I::Binding>>::Projection: WorthQueryOutputProjectionInstallation<S>,
//! {
//!     let retained = description.retain_operation::<P, A>(&constructing);
//!     drop(constructing);
//!     retained
//! }
//! ```
