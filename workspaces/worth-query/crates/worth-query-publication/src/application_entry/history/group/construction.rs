//! Freeze a bounded list without effects; every constructed owner stays in the root.
use super::*;

impl<
        'runtime,
        S: ApplicationSchema + 'static,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
{
    /// Each member must satisfy installed history membership and stored ownership.
    /// A group lends shared document access; it cannot enroll a nested group.
    /// ```compile_fail,E0507
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{
    ///     application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    ///     application_operation::{ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding, NoApplicationMutationSource},
    ///     application_program::{ApplicationAction, ApplicationActionDelivery, ApplicationProgramDefinition, ApplicationSourceCompletion},
    ///     application_schema::ApplicationSchema,
    /// };
    /// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
    /// fn nested<S, H, I, P, A>(
    ///     group: &mut WorthQueryGroupedHistoryDocument<'_, S, H, P, WorthQueryHistoryTypedMembers<S,I,P,A>>,
    ///     program: &WorthQueryProgramApplicationRuntime<S, P>,
    ///     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ///     member: (I, <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey),
    /// )
    /// where S: ApplicationSchema + 'static, H: ApplicationHistoryBinding<S>,
    ///     I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A: ApplicationAction<S, I::Binding>,
    ///     A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    ///     I::Binding: WorthQuerySourceOperationBinding<S> + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    ///     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    /// {
    ///     let (document, _) = group.parts();
    ///     let _ = document.group_edits::<I, P, A, 1>(program, request, 2, [member]);
    /// }
    /// ```
    /// Abort the first owner before transferring the recovered document to a new group.
    /// ```
    /// use worth_query_publication::facade::application_entry::*;
    /// use worth_query_declaration::facade::{
    ///     application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    ///     application_operation::{ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding, NoApplicationMutationSource},
    ///     application_program::{ApplicationAction, ApplicationActionDelivery, ApplicationProgramDefinition, ApplicationSourceCompletion},
    ///     application_schema::ApplicationSchema,
    /// };
    /// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
    /// fn abort_then_open<'r, S, H, I, P, A>(
    ///     group: WorthQueryGroupedHistoryDocument<'r, S, H, P, WorthQueryHistoryTypedMembers<S,I,P,A>>,
    ///     program: &WorthQueryProgramApplicationRuntime<S, P>,
    ///     request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ///     member: (I, <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey),
    /// ) -> WorthQueryGroupedHistoryDocument<'r, S, H, P, WorthQueryHistoryTypedMembers<S,I,P,A>>
    /// where S: ApplicationSchema + 'static, H: ApplicationHistoryBinding<S>,
    ///     I: ApplicationMutationIntent<S>, P: ApplicationProgramDefinition<S>, A: ApplicationAction<S, I::Binding>,
    ///     A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion>,
    ///     I::Binding: WorthQuerySourceOperationBinding<S> + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    ///     <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    ///     <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    /// {
    ///     let document = group.abort(request).unwrap();
    ///     document.group_edits::<I, P, A, 1>(program, request, 2, [member]).unwrap()
    /// }
    /// ```
    pub fn group_edits<I, P, A, const N: usize>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: u128,
        members: [(I, Key<S, I>); N],
    ) -> Result<
        WorthQueryGroupedHistoryDocument<
            'runtime,
            S,
            H,
            P,
            WorthQueryHistoryTypedMembers<S, I, P, A>,
            Payload,
            Outputs,
        >,
        (Self, WorthQueryHistoryGroupDenial),
    >
    where
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        I::Binding: WorthQuerySourceOperationBinding<S>
            + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
        <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
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
                        + std::mem::size_of::<Completion<S, I, P, A>>(),
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
                members: WorthQueryHistoryTypedMembers {
                    items: descriptions,
                    completed: Vec::with_capacity(N),
                },
                reservation: None,
                candidate: None,
                recovery: None,
                cleanup: None,
                aborting: None,
                selected: None,
                next: 0,
                closing: 0,
                posture: WorthQueryHistoryGroupPosture::Constructing,
                _memory: memory.into_iter().collect(),
                frozen: false,
                marker: std::marker::PhantomData,
            },
        })
    }
}
