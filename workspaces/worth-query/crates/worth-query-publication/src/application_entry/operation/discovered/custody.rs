//! Move-only ordinary custody; discriminants select existing native owners, never model traversal.
use super::*;
/// Retains the immutable description and its genuine discovered-source/output custody.
/// Every advancement borrows fresh authority and returns all custody on refusal.
///
/// Retrying cannot substitute original conditions, key, input or selected delivery.
/// ```compile_fail,E0616
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationIntent, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn original<S,I,P,R,A>(owner: &WorthQueryDiscoveredOperation<'_,S,I,P,R,A>)
/// where S:ApplicationSchema+'static, I:ApplicationMutationIntent<S>, P:ApplicationProgramDefinition<S>,
/// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
/// A:ApplicationDiscoveredAction<S,I::Binding>, A::Delivery:ApplicationDiscoveredActionDelivery<S,I::Binding,Root=R> {
///     let _ = &owner.description;
/// }
/// ```
/// The same owner exposes its original issued branch without granting authority.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationIntent, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn original<S,I,P,R,A>(owner: &WorthQueryDiscoveredOperation<'_,S,I,P,R,A>)
/// where S:ApplicationSchema+'static, I:ApplicationMutationIntent<S>, P:ApplicationProgramDefinition<S>,
/// R:ApplicationOutputGraphShape<S>+ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding:WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
/// A:ApplicationDiscoveredAction<S,I::Binding>, A::Delivery:ApplicationDiscoveredActionDelivery<S,I::Binding,Root=R> {
///     let _ = owner.branch();
/// }
/// ```
#[must_use = "retain discovered source, output and cleanup custody until lawful close"]
pub struct WorthQueryDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub(super) runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
    pub(super) installed_action: &'r ApplicationActionDeclaration,
    pub(super) description: WorthQueryOperationDescription<S, I>,
    pub(super) initial_cause:
        Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) result: Option<<I::Binding as ApplicationMutationBinding<S>>::Result>,
    pub(super) recovered: Option<WorthQueryManagedApplicationRecoveryPerformed>,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) cleanup: WorthQueryOperationCleanup,
    pub(super) action: std::marker::PhantomData<A>,
    pub(super) state: DiscoveredState<S, I, P, R>,
}
/// Native completed aggregate and original description; delivery does not discharge cleanup.
#[must_use = "retain the complete settlement and cleanup until lawful close"]
pub struct WorthQuerySettledDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub(super) runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
    pub(super) installed_action: &'r ApplicationActionDeclaration,
    pub(super) description: WorthQueryOperationDescription<S, I>,
    pub(super) initial_cause:
        Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) result: Option<<I::Binding as ApplicationMutationBinding<S>>::Result>,
    pub(super) recovered: Option<WorthQueryManagedApplicationRecoveryPerformed>,
    pub(super) prior_cleanup: Vec<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) cleanup: WorthQueryOperationCleanup,
    pub(super) action: std::marker::PhantomData<A>,
    pub(super) settlement:
        WorthQueryDiscoveredProgramOutputSettlement<RootQuery<S, R>, RootDemand<S, R>>,
}
/// Terminal native custody has observations and lawful cleanup, but no advance entrance.
///
/// ```compile_fail,E0599
/// use worth_query_publication::facade::application_entry::{WorthQueryTerminalDiscoveredOperation, WorthQueryApplicationRequest};
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationIntent, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn terminal_cannot_advance<'r,S,I,P,R,A>(owner:WorthQueryTerminalDiscoveredOperation<'r,S,I,P,R,A>,request:&WorthQueryApplicationRequest<'_, '_, '_, S>)
/// where
/// S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>,
/// P: ApplicationProgramDefinition<S>,
/// R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
/// A: ApplicationDiscoveredAction<S,I::Binding>,
/// A::Delivery: ApplicationDiscoveredActionDelivery<S,I::Binding,Root=R>,
/// { let _ = owner.native_outcome(); let _ = owner.advance(request); }
/// ```
/// ```no_run
/// use worth_query_publication::facade::application_entry::{WorthQueryTerminalDiscoveredOperation, WorthQueryApplicationRequest};
/// use worth_query_declaration::facade::{application_program::*, application_operation::ApplicationMutationIntent, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::primary_graph::WorthQueryApplicationDiscoveredOutputConnection;
/// fn terminal_observes_native_custody<'r,S,I,P,R,A>(owner:&WorthQueryTerminalDiscoveredOperation<'r,S,I,P,R,A>)
/// where
/// S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>,
/// P: ApplicationProgramDefinition<S>,
/// R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
/// <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationDiscoveredOutputConnection<S,Source=I::Binding>,
/// A: ApplicationDiscoveredAction<S,I::Binding>,
/// A::Delivery: ApplicationDiscoveredActionDelivery<S,I::Binding,Root=R>,
/// { let _ = owner.native_outcome(); }
/// ```
#[must_use = "terminal evidence can retain independent native obligations"]
pub struct WorthQueryTerminalDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub(super) operation: WorthQueryDiscoveredOperation<'r, S, I, P, R, A>,
}
pub(super) enum DiscoveredState<S, I, P, R>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
{
    Initial,
    Replay(WorthQueryApplicationCommitReceipt),
    Unpublished(WorthQueryUnpublishedDiscoveredApplicationMutation<S, I, P, R>),
    Retention(WorthQueryDiscoveredOutputRetentionFailure<S, I, P, R>),
    Performed(WorthQueryPerformedDiscoveredApplicationMutation<S, I, P, R>),
    Outputs(WorthQueryDiscoveredProgramOutputHandle<S, P, R>),
    Blocked(
        crate::application_entry::mutation::WorthQueryBlockedProgramSource<
            <I::Binding as ApplicationMutationBinding<S>>::Denial,
            <I::Binding as ApplicationMutationBinding<S>>::Result,
            Discovery<S, R>,
        >,
    ),
    NotPerformed(NativeOutcome<S, I>),
}
impl<'r, S, I, P, R, A> WorthQueryDiscoveredOperation<'r, S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationDiscoveredAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    /// Original owner-issued selector, never permission for another branch.
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.description.branch
    }
    pub fn declared_action(&self) -> &ApplicationActionDeclaration {
        self.installed_action
    }
    /// Genuine Fresh result where the native handoff makes it directly available.
    /// A partial recovery never manufactures the original handler result.
    pub fn handler_result(&self) -> Option<&<I::Binding as ApplicationMutationBinding<S>>::Result> {
        match &self.state {
            State::Retention(failure) => Some(failure.result()),
            State::Performed(performed) => Some(performed.result()),
            _ => self.result.as_ref(),
        }
    }
    pub fn recovered_progress(&self) -> Option<&WorthQueryManagedApplicationRecoveryPerformed> {
        match &self.state {
            State::Unpublished(partial) => partial.performed(),
            _ => self.recovered.as_ref(),
        }
    }
    pub fn initial_cause(
        &self,
    ) -> Option<worth_query_execution::facade::runtime::ProductUnpublishedCause> {
        self.initial_cause
    }
    pub(super) fn terminal(
        mut self,
        denial: WorthQueryOperationAdvanceDenial,
    ) -> DiscoveredAdvance<'r, S, I, P, R, A> {
        if let State::Unpublished(partial) = &mut self.state {
            self.prior_cleanup = partial.take_prior_cleanup();
        }
        WorthQueryOperationAdvance::Terminal {
            operation: WorthQueryTerminalDiscoveredOperation { operation: self },
            denial,
        }
    }
}
