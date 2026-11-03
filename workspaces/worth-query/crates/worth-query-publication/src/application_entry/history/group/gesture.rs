//! Incremental input capture freezes into the existing atomic group lifecycle.
use super::*;

impl<'r, S, H, Payload, Outputs> WorthQueryApplicationHistoryDocument<'r, S, H, Payload, Outputs>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Describe a runtime-length gesture, then advance the same owning group.
    /// Each push chooses an installed action; the first advance freezes membership.
    pub fn gesture<P: ApplicationProgramDefinition<S>>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: u128,
    ) -> Result<
        WorthQueryGroupedHistoryDocument<
            'r,
            S,
            H,
            P,
            WorthQueryHistoryGestureMembers<S, H, P>,
            Payload,
            Outputs,
        >,
        (Self, WorthQueryHistoryGroupDenial),
    > {
        if !std::ptr::eq(self.owner.application(), request.application)
            || !std::ptr::eq(self.owner.application(), program.runtime())
        {
            return Err((
                self,
                WorthQueryHistoryGroupDenial::History(
                    WorthQueryApplicationHistoryDenial::ForeignRuntime,
                ),
            ));
        }
        use WorthQueryHistoryGroupDenial as Denial;
        let runtime = self.owner.application();
        let installed = match runtime.installed_schema().installed_history::<H>() {
            Ok(installed) => installed,
            Err(cause) => {
                return Err((
                    self,
                    Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause)),
                ))
            }
        };
        let limits = installed.group_resources();
        let capacity = limits
            .maximum_members
            .get()
            .min(limits.maximum_retained_results.get());
        let bytes=match capacity.checked_mul(
            std::mem::size_of::<Box<dyn storage::GroupMemberDriver<S,H,P>>>()
                +std::mem::size_of::<WorthQueryHistoryGestureCompletion<S,P>>()
                +std::mem::size_of::<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryGroupMemberAdmission>()
                +std::mem::size_of::<worth_execution::ExecutionMemoryReservation>(),
        ).and_then(|bytes|bytes.checked_add(std::mem::size_of::<worth_execution::ExecutionMemoryReservation>())) {
            Some(bytes)=>bytes as u64,None=>return Err((self,Denial::Capacity)),
        };
        let memory = match reserve_gesture_memory(runtime, request, bytes) {
            Ok(memory) => memory,
            Err(cause) => return Err((self, cause)),
        };
        let mut grants = Vec::with_capacity(capacity + 1);
        if let Some(memory) = memory {
            grants.push(memory)
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
                members: WorthQueryHistoryGestureMembers {
                    items: Vec::with_capacity(capacity),
                    completed: Vec::with_capacity(capacity),
                },
                reservation: None,
                candidate: None,
                recovery: None,
                cleanup: None,
                aborting: None,
                selected: None,
                next: 0,
                closing: 0,
                frozen: false,
                posture: WorthQueryHistoryGroupPosture::Constructing,
                _memory: grants,
                marker: std::marker::PhantomData,
            },
        })
    }
}
impl<'r, S, H, P, Payload, Outputs>
    WorthQueryGroupedHistoryDocument<
        'r,
        S,
        H,
        P,
        WorthQueryHistoryGestureMembers<S, H, P>,
        Payload,
        Outputs,
    >
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H>,
{
    /// Capture one original input only after the installed limits and storage
    /// grant admit it. A refused push preserves every previously captured member.
    pub fn push<I, A>(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
        key: Key<S, I>,
    ) -> Result<usize, WorthQueryHistoryGroupDenial>
    where
        I: WorthQueryHistoryGestureIntent<S, H, P, A>,
        A: ApplicationAction<S, I::Binding>,
    {
        use WorthQueryHistoryGroupDenial as Denial;
        self.execution.validate_document(&self.document, request)?;
        if self.execution.frozen {
            return Err(Denial::MembershipFrozen);
        }
        let runtime = self.document.owner.application();
        let installed = runtime
            .installed_schema()
            .installed_history::<H>()
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause))
            })?;
        let limits = installed.group_resources();
        let count = self.execution.members.items.len();
        let refused = if count >= limits.maximum_members.get() {
            Some(WorthQueryDocumentHistoryDenial::GroupMemberLimitExceeded)
        } else if count >= limits.maximum_retained_results.get() {
            Some(WorthQueryDocumentHistoryDenial::GroupResultLimitExceeded)
        } else {
            None
        };
        if let Some(cause) = refused {
            return Err(Denial::History(
                WorthQueryApplicationHistoryDenial::Document(cause),
            ));
        }
        let bytes =
            <I as storage::GestureInput<S, H, P, A>>::retained_bytes().ok_or(Denial::Capacity)?;
        let memory = reserve_gesture_memory(runtime, request, bytes)?;
        let current = WorthQueryApplicationRequest {
            application: runtime,
            principal: request.principal,
            scope: request.scope,
            branch: self.document.owner.branch(),
        };
        let member = <I as storage::GestureInput<S, H, P, A>>::capture(
            intent,
            &current,
            key,
            &program_publication_access(),
        )
        .map_err(Denial::Input)?;
        self.execution.members.items.push(member);
        if let Some(memory) = memory {
            self.execution._memory.push(memory)
        }
        Ok(count)
    }
}

fn reserve_gesture_memory<S: ApplicationSchema>(
    runtime:&worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    bytes: u64,
) -> Result<Option<worth_execution::ExecutionMemoryReservation>, WorthQueryHistoryGroupDenial> {
    use WorthQueryHistoryGroupDenial as Denial;
    runtime
        .with_application_advancement(request.scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
                })?;
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
        })
        .map_err(|cause| {
            Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
        })?
}
