//! Freeze a bounded list without effects; every constructed owner stays in the root.
use super::*;

impl<'runtime, S: ApplicationSchema + 'static, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryDocument<'runtime, S, H>
{
    /// A group cannot enroll an operation whose scope is unrelated to its model.
    /// ```compile_fail,E0271
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{
    ///     application_history::{ApplicationHistoryBinding,ApplicationHistoryModel},
    ///     application_operation::{ApplicationMutationBinding,ApplicationMutationIntent,ApplicationMutationScopeBinding,NoApplicationMutationSource},
    ///     application_program::{ApplicationAction,ApplicationActionDelivery,ApplicationProgramDefinition,ApplicationSourceCompletion},
    ///     application_schema::ApplicationSchema,
    /// };
    /// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
    /// fn open<S,H,I,P,A>(
    ///     document:WorthQueryApplicationHistoryDocument<'_,S,H>,
    ///     program:&WorthQueryProgramApplicationRuntime<S,P>,
    ///     request:&WorthQueryApplicationRequest<'_, '_, '_,S>,
    ///     member:(I,<I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey),
    /// )
    /// where S:ApplicationSchema+'static,H:ApplicationHistoryBinding<S>,
    ///     I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,A:ApplicationAction<S,I::Binding>,
    ///     A::Delivery:ApplicationActionDelivery<S,I::Binding,Mode=ApplicationSourceCompletion>,
    ///     I::Binding:WorthQuerySourceOperationBinding<S>+ApplicationMutationBinding<S,SourceExpectation=NoApplicationMutationSource>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input:Clone+Send+Sync,
    /// { let _=document.group_edits::<I,P,A,1>(program,request,1,[member]); }
    /// ```
    /// The binding's exact model scope is the lawful counterpart.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{
    ///     application_history::{ApplicationHistoryBinding,ApplicationHistoryModel},
    ///     application_operation::{ApplicationMutationBinding,ApplicationMutationIntent,ApplicationMutationScopeBinding,NoApplicationMutationSource},
    ///     application_program::{ApplicationAction,ApplicationActionDelivery,ApplicationProgramDefinition,ApplicationSourceCompletion},
    ///     application_schema::ApplicationSchema,
    /// };
    /// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
    /// fn open<S,H,I,P,A>(
    ///     document:WorthQueryApplicationHistoryDocument<'_,S,H>,
    ///     program:&WorthQueryProgramApplicationRuntime<S,P>,
    ///     request:&WorthQueryApplicationRequest<'_, '_, '_,S>,
    ///     member:(I,<I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey),
    /// )
    /// where S:ApplicationSchema+'static,H:ApplicationHistoryBinding<S>,
    ///     I:ApplicationMutationIntent<S>,P:ApplicationProgramDefinition<S>,A:ApplicationAction<S,I::Binding>,
    ///     A::Delivery:ApplicationActionDelivery<S,I::Binding,Mode=ApplicationSourceCompletion>,
    ///     I::Binding:WorthQuerySourceOperationBinding<S>+ApplicationMutationBinding<S,SourceExpectation=NoApplicationMutationSource>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input:Clone+Send+Sync,
    ///     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:ApplicationMutationScopeBinding<S,Scope=ApplicationHistoryModel<S,H>>,
    /// { let _=document.group_edits::<I,P,A,1>(program,request,1,[member]); }
    /// ```
    pub fn group_edits<I, P, A, const N: usize>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: u128,
        members: [(I, Key<S, I>); N],
    ) -> Result<
        WorthQueryGroupedHistoryDocument<'runtime, S, H, I, P, A>,
        (Self, WorthQueryHistoryGroupDenial),
    >
    where
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        I::Binding: WorthQuerySourceOperationBinding<S>
            + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
        <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
        A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = Mode>,
    {
        use WorthQueryHistoryGroupDenial as Denial;
        if !std::ptr::eq(self.owner.application(), request.application)
            || !std::ptr::eq(self.owner.application(), program.runtime())
        {
            return Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::ForeignRuntime),
            ));
        }
        let installed = match self
            .owner
            .application()
            .installed_schema()
            .installed_history::<H>()
        {
            Ok(installed) => installed,
            Err(cause) => {
                return Err((
                    self,
                    Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause)),
                ))
            }
        };
        let limits = installed.group_resources();
        let refused = if N == 0 {
            Some(WorthQueryDocumentHistoryDenial::EmptyGroup)
        } else if N > limits.maximum_members.get() {
            Some(WorthQueryDocumentHistoryDenial::GroupMemberLimitExceeded)
        } else if N > limits.maximum_retained_results.get() {
            Some(WorthQueryDocumentHistoryDenial::GroupResultLimitExceeded)
        } else {
            None
        };
        if let Some(cause) = refused {
            return Err((
                self,
                Denial::History(WorthQueryApplicationHistoryDenial::Document(cause)),
            ));
        }
        let runtime = self.owner.application();
        let memory = runtime.with_application_advancement(request.scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
                })?;
            let bytes = N
                .checked_mul(
                    std::mem::size_of::<Member<S, I, P, A>>()
                        + std::mem::size_of::<(WorthQueryClosedOperation, Completion<S, I>)>(),
                )
                .ok_or(Denial::Capacity)? as u64;
            match execution
                .in_scope(|lease| {
                    worth_execution::ExecutionMemoryReservation::reserve_in_scope(lease, bytes)
                })
                .map_err(Denial::Work)?
            {
                Ok(ticket) => Ok(Some(ticket)),
                Err(worth_execution::LeaseDenial::NoActiveExecutionScope)
                    if !execution.has_accounting() =>
                {
                    Ok(None)
                }
                Err(cause) => Err(Denial::Memory(cause)),
            }
        });
        let memory = match memory {
            Ok(Ok(memory)) => memory,
            Ok(Err(cause)) => return Err((self, cause)),
            Err(cause) => {
                return Err((
                    self,
                    Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause)),
                ))
            }
        };
        let mut descriptions = Vec::with_capacity(N);
        for (intent, member_key) in members {
            let current = WorthQueryApplicationRequest {
                application: runtime,
                principal: request.principal,
                scope: request.scope,
                branch: self.owner.branch(),
            };
            match current
                .mutate(intent)
                .without_source()
                .idempotency_owned(member_key)
            {
                Ok(description) => descriptions.push(Member::Description(Some(description))),
                Err((_, _, cause)) => return Err((self, Denial::Input(cause))),
            }
        }
        let document_identity = self
            .owner
            .history_document_identity(&program_publication_access());
        Ok(WorthQueryGroupedHistoryDocument {
            document: self,
            execution: WorthQueryHistoryGroupExecution {
                program: program.clone(),
                key,
                handoff: None,
                document_identity,
                members: descriptions,
                completed: Vec::with_capacity(N),
                reservation: None,
                candidate: None,
                recovery: None,
                cleanup: None,
                aborting: None,
                selected: None,
                next: 0,
                closing: 0,
                posture: WorthQueryHistoryGroupPosture::Constructing,
                _memory: memory,
                marker: std::marker::PhantomData,
            },
        })
    }
}
