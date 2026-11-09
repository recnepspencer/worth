//! Runtime-length storage retains concrete SDK owners behind a sealed driver.
use super::*;
use std::any::Any;

pub struct WorthQueryHistoryGestureMembers<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub(in super::super) items: Vec<Box<dyn ErasedMember<S, H, P>>>,
    pub(in super::super) completed: Vec<WorthQueryHistoryGestureCompletion<S, P>>,
}

struct OriginalCompletion<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
{
    value: Completion<S, I, P, A>,
    member: std::marker::PhantomData<fn() -> (S, I, P, A)>,
}

/// Descriptive projection of an original completed member, never reconstructed evidence.
pub struct WorthQueryHistoryGestureCompletion<
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
> {
    original: Box<dyn Any>,
    marker: std::marker::PhantomData<fn() -> (S, P)>,
}
impl<S: ApplicationSchema, P: ApplicationProgramDefinition<S>>
    WorthQueryHistoryGestureCompletion<S, P>
{
    pub fn original<I, A>(&self) -> Option<&Completion<S, I, P, A>>
    where
        I: ApplicationMutationIntent<S> + 'static,
        A: ApplicationAction<S, I::Binding>,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
        Completion<S, I, P, A>: 'static,
    {
        self.original
            .downcast_ref::<OriginalCompletion<S, I, P, A>>()
            .map(|original| &original.value)
    }
    pub fn into_original<I, A>(self) -> Result<Completion<S, I, P, A>, Self>
    where
        I: ApplicationMutationIntent<S> + 'static,
        A: ApplicationAction<S, I::Binding>,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
        Completion<S, I, P, A>: 'static,
    {
        match self.original.downcast::<OriginalCompletion<S, I, P, A>>() {
            Ok(original) => Ok(original.value),
            Err(original) => Err(Self {
                original,
                marker: std::marker::PhantomData,
            }),
        }
    }
}

pub trait ErasedMember<S, H, P>: GroupMemberDriver<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    fn original(&self) -> &dyn Any;
    fn finish_erased(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        Option<WorthQueryHistoryGestureCompletion<S, P>>,
        crate::application_entry::WorthQueryOperationCloseDenial,
    >;
}
impl<S, H, I, P, A> ErasedMember<S, H, P> for Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S> + 'static,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    Self: 'static,
    Completion<S, I, P, A>: 'static,
{
    fn original(&self) -> &dyn Any {
        self
    }
    fn finish_erased(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        Option<WorthQueryHistoryGestureCompletion<S, P>>,
        crate::application_entry::WorthQueryOperationCloseDenial,
    > {
        self.finish(request).map(|completion| {
            completion.map(|original| WorthQueryHistoryGestureCompletion {
                original: Box::new(OriginalCompletion::<S, I, P, A> {
                    value: original,
                    member: std::marker::PhantomData,
                }),
                marker: std::marker::PhantomData,
            })
        })
    }
}

pub trait GestureInput<S, H, P, A>: ApplicationMutationIntent<S>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, Self::Binding>,
{
    fn retained_bytes() -> Option<u64>;
    fn capture(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: Key<S, Self>,
        access: &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<
        Box<dyn ErasedMember<S, H, P>>,
        crate::application_entry::WorthQueryOperationInputCaptureDenial,
    >;
}
/// Sealed availability of the original typed member for heterogeneous storage.
pub trait WorthQueryHistoryGestureIntent<S, H, P, A>: GestureInput<S, H, P, A>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, Self::Binding>,
{
}
impl<S, H, I, P, A> WorthQueryHistoryGestureIntent<S, H, P, A> for I
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: GestureInput<S, H, P, A>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
{
}
impl<S, H, I, P, A> GestureInput<S, H, P, A> for I
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S> + 'static,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    Member<S, I, P, A>: 'static,
    Completion<S, I, P, A>: 'static,
{
    fn retained_bytes() -> Option<u64> {
        let bytes = std::mem::size_of::<Member<S, I, P, A>>()
            .checked_add(std::mem::size_of::<Completion<S, I, P, A>>())?;
        u64::try_from(bytes).ok()
    }
    fn capture(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: Key<S, Self>,
        _access: &worth_query_execution::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Result<
        Box<dyn ErasedMember<S, H, P>>,
        crate::application_entry::WorthQueryOperationInputCaptureDenial,
    > {
        request
            .mutate(self)
            .without_source()
            .idempotency_owned(key)
            .map(|description| {
                Box::new(Member::<S, I, P, A>::Description(Some(description)))
                    as Box<dyn ErasedMember<S, H, P>>
            })
            .map_err(|(_, _, cause)| cause)
    }
}

impl<S, H, P> WorthQueryHistoryGroupMembers<S, H, P> for WorthQueryHistoryGestureMembers<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
}
impl<S, H, P> GroupMembersDriver<S, H, P> for WorthQueryHistoryGestureMembers<S, H, P>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    type Completion = WorthQueryHistoryGestureCompletion<S, P>;
    fn validate_outputs(
        &self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        for item in &self.items {
            item.validate_outputs(program)?;
        }
        Ok(())
    }
    fn len(&self) -> usize {
        self.items.len()
    }
    fn member(&self, index: usize) -> &dyn GroupMemberDriver<S, H, P> {
        self.items[index].as_ref()
    }
    fn member_mut(&mut self, index: usize) -> &mut dyn GroupMemberDriver<S, H, P> {
        self.items[index].as_mut()
    }
    fn finish_member(
        &mut self,
        index: usize,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), crate::application_entry::WorthQueryOperationCloseDenial> {
        if let Some(completion) = self.items[index].finish_erased(request)? {
            self.completed.push(completion)
        }
        Ok(())
    }
    fn into_completions(self) -> Vec<Self::Completion> {
        self.completed
    }
}
impl<S, H, P> WorthQueryHistoryGroupExecution<'_, S, H, P, WorthQueryHistoryGestureMembers<S, H, P>>
where
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
{
    pub fn completed_source<I, A>(&self, index: usize) -> Option<&Completed<S, I, P, A>>
    where
        I: ApplicationMutationIntent<S> + 'static,
        A: ApplicationAction<S, I::Binding>,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
        I: 'static,
        A: 'static,
        Operation<S, I, P, A>: 'static,
        Completed<S, I, P, A>: 'static,
        Terminal<S, I, P, A>: 'static,
        Closing<S, I, P, A>: 'static,
    {
        match self
            .members
            .items
            .get(index)?
            .original()
            .downcast_ref::<Member<S, I, P, A>>()?
        {
            Member::Completed(completed) => completed.as_ref(),
            _ => None,
        }
    }
    pub fn source_operation<I, A>(&self, index: usize) -> Option<&Operation<S, I, P, A>>
    where
        I: ApplicationMutationIntent<S> + 'static,
        A: ApplicationAction<S, I::Binding>,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
        I: 'static,
        A: 'static,
        Operation<S, I, P, A>: 'static,
        Completed<S, I, P, A>: 'static,
        Terminal<S, I, P, A>: 'static,
        Closing<S, I, P, A>: 'static,
    {
        match self
            .members
            .items
            .get(index)?
            .original()
            .downcast_ref::<Member<S, I, P, A>>()?
        {
            Member::Unbound(operation) | Member::Operation(operation) => operation.as_ref(),
            _ => None,
        }
    }
    pub fn terminal_source<I, A>(&self) -> Option<&Terminal<S, I, P, A>>
    where
        I: ApplicationMutationIntent<S> + 'static,
        A: ApplicationAction<S, I::Binding>,
        Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
        I: 'static,
        A: 'static,
        Operation<S, I, P, A>: 'static,
        Completed<S, I, P, A>: 'static,
        Terminal<S, I, P, A>: 'static,
        Closing<S, I, P, A>: 'static,
    {
        self.members.items.iter().find_map(|item| {
            match item.original().downcast_ref::<Member<S, I, P, A>>()? {
                Member::Terminal(terminal) => terminal.as_ref(),
                _ => None,
            }
        })
    }
}
