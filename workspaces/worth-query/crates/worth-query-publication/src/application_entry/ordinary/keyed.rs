//! Borrowed-key workflow binding; ordinary execution retains an owner, native execution selects advanced.
use crate::application_entry::mutation::request::WorthQueryMutationSourceUnprepared;
use crate::application_entry::mutation::WorthQueryApplicationMutationRequestWithIdempotency;
use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_schema::ApplicationSchema;

/// Borrowed-key construction for binding issued workflow authority.
/// Ordinary workflow execution retains its owner; deliberate native execution selects `advanced()`.
/// It cannot execute or recover a fixed, discovered or source-only action. Those lanes require
/// `idempotency_owned` and an exact declared action through the owning builder.
///
/// Borrowing a key cannot reopen manual discovered publication on the ordinary surface.
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_program::*,application_operation::*,application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime,primary_graph::WorthQueryApplicationDiscoveredOutputConnection,runtime::ExecutionAllocationPolicy};
/// fn manual<'a,'p,'s,'k,S,I,P,R>(pending:WorthQueryPendingApplicationLane<'a,'p,'s,'k,S,I,WorthQueryMutationSourcePrepared>,runtime:&'a WorthQueryProgramApplicationRuntime<S,P>,allocation:ExecutionAllocationPolicy<'_, '_>)
/// where S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,
/// I::Binding:ApplicationMutationAdmissionBinding<S,Admission=ApplicationConventionalAdmission>,
/// <I::Binding as ApplicationMutationBinding<S>>::Input:Clone+Send+Sync,
/// <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:ApplicationMutationScopeResolution<S,<I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
/// P:ApplicationProgramDefinition<S>,P::Outputs:ApplicationProgramOutputsShape<S>,
/// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding> {
///     let _ = pending.execute_performed_discovered::<P,R>(runtime,allocation);
/// }
/// ```
/// The same native call is lawful with identical binding/input/scope/root and lifetime bounds
/// on the explicit advanced receiver. No missing admission bound explains the negative.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_program::*,application_operation::*,application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime,primary_graph::WorthQueryApplicationDiscoveredOutputConnection,runtime::ExecutionAllocationPolicy};
/// use worth_query_publication::facade::application_entry::advanced::WorthQueryApplicationMutationRequestWithIdempotency;
/// fn manual<'a,'p,'s,'k,S,I,P,R>(pending:WorthQueryApplicationMutationRequestWithIdempotency<'a,'p,'s,'k,S,I,WorthQueryMutationSourcePrepared>,runtime:&'a WorthQueryProgramApplicationRuntime<S,P>,allocation:ExecutionAllocationPolicy<'_, '_>)
/// where S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,
/// I::Binding:ApplicationMutationAdmissionBinding<S,Admission=ApplicationConventionalAdmission>,
/// <I::Binding as ApplicationMutationBinding<S>>::Input:Clone+Send+Sync,
/// <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:ApplicationMutationScopeResolution<S,<I::Binding as ApplicationMutationBinding<S>>::PrincipalIdentity>,
/// P:ApplicationProgramDefinition<S>,P::Outputs:ApplicationProgramOutputsShape<S>,
/// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding> {
///     let _ = pending.execute_performed_discovered::<P,R>(runtime,allocation);
/// }
/// ```
/// An original owned description instead selects the real installed discovered owner.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_program::*,application_operation::ApplicationMutationIntent,application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime,primary_graph::WorthQueryApplicationDiscoveredOutputConnection};
/// fn retain<S,I,P,R,A>(description:WorthQueryOperationDescription<S,I>,runtime:&WorthQueryProgramApplicationRuntime<S,P>)
/// where S:ApplicationSchema+'static,I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
/// A:ApplicationDiscoveredAction<S,I::Binding>,A::Delivery:ApplicationDiscoveredActionDelivery<S,I::Binding,Root=R>,
/// <A::Delivery as ApplicationDiscoveredActionDelivery<S,I::Binding>>::Projection:WorthQueryOutputProjectionInstallation<S> {
///     let _ = description.retain_operation::<P,A>(runtime);
/// }
/// ```
#[must_use = "select an issued workflow slot; fixed, discovered and source-only actions need an owned key"]
pub struct WorthQueryPendingApplicationLane<
    'application,
    'principal,
    'scope,
    'key,
    Schema,
    Intent,
    SourcePreparation = WorthQueryMutationSourceUnprepared,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) native: WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >,
}
