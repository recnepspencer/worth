//! Closed Fixed delivery preserves the declared no-query/query/batch shape.
//!
//! Native close is the only handoff, and no-query delivery needs no query bounds.
//! ```
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! fn close<S,I,P,R,A>(owner: WorthQuerySettledFixedOperation<S,I,P,R,A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//! -> Result<WorthQueryClosedFixedDelivery<S,I,P,R,A>, (WorthQuerySettledFixedOperation<S,I,P,R,A>, WorthQueryOperationCloseDenial)>
//! where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
//! R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//! Connection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
//! Binding<S,I>: WorthQueryApplicationRequiredOutputSource<S,Connection<S,R>>,
//! A: ApplicationAction<S,Binding<S,I>>,
//! Delivery<S,I,A>: ApplicationFixedActionDelivery<S,Binding<S,I>,Root=R,Projection=ApplicationNoProjection> { owner.close_for_delivery(request) }
//! fn deliver<S,I,P,R,A>(owner: &WorthQueryClosedFixedDelivery<S,I,P,R,A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
//! where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
//! R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//! Connection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
//! Binding<S,I>: WorthQueryApplicationRequiredOutputSource<S,Connection<S,R>>,
//! A: ApplicationAction<S,Binding<S,I>>,
//! Delivery<S,I,A>: ApplicationFixedActionDelivery<S,Binding<S,I>,Root=R,Projection=ApplicationNoProjection> { let _ = owner.deliver_current(request, policy); }
//! ```
//!
//! Declared scalar delivery uses its real query contracts.
//! ```
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! type Read<Schema, Intent, Action> = <Delivery<Schema, Intent, Action> as ApplicationFixedQueryDelivery<Schema, Binding<Schema, Intent>>>::Read;
//! fn query<Schema, Intent, Program, Root, Action>(owner: &WorthQueryClosedFixedDelivery<Schema, Intent, Program, Root, Action>, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>)
//! where
//!     Schema: ApplicationSchema + 'static,
//!     Intent: ApplicationMutationIntent<Schema>,
//!     Program: ApplicationProgramDefinition<Schema>,
//!     Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
//!     Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
//!     Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
//!     Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
//!     Delivery<Schema, Intent, Action>: ApplicationFixedQueryDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
//!     <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Read<Schema, Intent, Action>>,
//!     <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
//!     <<Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Query> + Clone,
//!  { let _ = owner.read_current(request); }
//! ```
//!
//! Declared homogeneous batches retain their original arity and selector contracts.
//! ```
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! type BatchRead<Schema, Intent, Action, const N: usize> = <Delivery<Schema, Intent, Action> as ApplicationFixedBatchDelivery<Schema, Binding<Schema, Intent>, N>>::Read;
//! fn batch<Schema, Intent, Program, Root, Action, const N: usize>(owner: &WorthQueryClosedFixedDelivery<Schema, Intent, Program, Root, Action>, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>)
//! where
//!     Schema: ApplicationSchema + 'static,
//!     Intent: ApplicationMutationIntent<Schema>,
//!     Program: ApplicationProgramDefinition<Schema>,
//!     Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
//!     Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
//!     Binding<Schema, Intent>:
//!         WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
//!     Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
//!     Delivery<Schema, Intent, Action>:
//!         ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
//!     Delivery<Schema, Intent, Action>: ApplicationFixedBatchDelivery<Schema, Binding<Schema, Intent>, N, Root = Root>,
//!     <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = BatchRead<Schema, Intent, Action, N>>,
//!     <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
//!     <<BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <BatchRead<Schema, Intent, Action, N> as ApplicationQueryBinding<Schema>>::Query>,
//!  { let _ = owner.read_batch_current::<N>(request); }
//! ```
//!
//! Authentic cleanup-only evidence cannot mint a Fixed delivery carrier.
//! ```compile_fail,E0451
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! fn mint<S,I,P,R,A>(closed: WorthQueryClosedOperation) -> WorthQueryClosedFixedDelivery<S,I,P,R,A>
//! where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
//! R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//! Connection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
//! Binding<S,I>: WorthQueryApplicationRequiredOutputSource<S,Connection<S,R>>,
//! A: ApplicationAction<S,Binding<S,I>>,
//! Delivery<S,I,A>: ApplicationFixedActionDelivery<S,Binding<S,I>,Root=R,Projection=ApplicationNoProjection> { WorthQueryClosedFixedDelivery { closed, delivery: todo!() } }
//! ```
//!
//! A no-projection action cannot acquire query delivery.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! fn query<S,I,P,R,A>(owner: &WorthQueryClosedFixedDelivery<S,I,P,R,A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>)
//! where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
//! R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//! Connection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
//! Binding<S,I>: WorthQueryApplicationRequiredOutputSource<S,Connection<S,R>>,
//! A: ApplicationAction<S,Binding<S,I>>,
//! Delivery<S,I,A>: ApplicationFixedActionDelivery<S,Binding<S,I>,Root=R,Projection=ApplicationNoProjection> { let _ = owner.read_current(request); }
//! ```
//!
//! A declared query action cannot bypass its selector through no-query delivery.
//! ```compile_fail,E0599
//! use worth_query_host::facade::application_entry::*;
//! use worth_query_decl::facade::{application_operation::*, application_program::*, application_schema::*, application_query::*};
//! use worth_query_host::facade::primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource, WorthQueryApplicationProjection};
//! type Binding<S,I> = <I as ApplicationMutationIntent<S>>::Binding;
//! type Connection<S,R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;
//! type Delivery<S,I,A> = <A as ApplicationAction<S,Binding<S,I>>>::Delivery;
//! fn bypass<S,I,P,R,A>(owner: &WorthQueryClosedFixedDelivery<S,I,P,R,A>, request: &WorthQueryApplicationRequest<'_, '_, '_, S>, policy: WorthQueryOperationDeliveryPolicy)
//! where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>,
//! R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
//! Connection<S,R>: WorthQueryApplicationRequiredOutputConnection<S>,
//! Binding<S,I>: WorthQueryApplicationRequiredOutputSource<S,Connection<S,R>>,
//! A: ApplicationAction<S,Binding<S,I>>,
//! Delivery<S,I,A>: ApplicationFixedQueryDelivery<S,Binding<S,I>,Root=R>
//! { let _ = owner.deliver_current(request, policy); }
//! ```
//!
