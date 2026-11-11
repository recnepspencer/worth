//! Homogeneous groups keep their original flat storage and typed result collection.
use super::*;
pub struct WorthQueryHistoryTypedMembers<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    pub(in super::super) items: Vec<Member<S, I, P, A>>,
    pub(in super::super) completed: Vec<Completion<S, I, P, A>>,
}
impl<S, H, I, P, A> WorthQueryHistoryGroupMembers<S, H, P>
    for WorthQueryHistoryTypedMembers<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
}
impl<S, H, I, P, A> GroupMembersDriver<S, H, P> for WorthQueryHistoryTypedMembers<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    type Completion = Completion<S, I, P, A>;
    fn validate_outputs(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        Mode::<S, I, A>::validate_group_outputs(program).map_err(|cause| {
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(
                cause,
            ))
        })
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn member(&self, index: usize) -> &dyn GroupMemberDriver<S, H, P> {
        &self.items[index]
    }
    fn member_mut(&mut self, index: usize) -> &mut dyn GroupMemberDriver<S, H, P> {
        &mut self.items[index]
    }
    fn finish_member(
        &mut self,
        index: usize,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial> {
        if let Some(completion) = self.items[index].finish(request)? {
            self.completed.push(completion)
        }
        Ok(())
    }
    fn into_completions(self) -> Vec<Self::Completion> {
        self.completed
    }
}
impl<S, H, I, P, A>
    WorthQueryHistoryGroupExecution<'_, S, H, P, WorthQueryHistoryTypedMembers<S, I, P, A>>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub fn terminal_source(&self) -> Option<&Terminal<S, I, P, A>> {
        self.members.items.iter().find_map(|member| {
            if let Member::Terminal(terminal) = member {
                terminal.as_ref()
            } else {
                None
            }
        })
    }
    pub fn completed_source(&self, index: usize) -> Option<&Completed<S, I, P, A>> {
        match self.members.items.get(index)? {
            Member::Completed(completed) => completed.as_ref(),
            _ => None,
        }
    }
    pub fn source_operation(&self, index: usize) -> Option<&Operation<S, I, P, A>> {
        match self.members.items.get(index)? {
            Member::Unbound(operation) | Member::Operation(operation) => operation.as_ref(),
            _ => None,
        }
    }
}
