//! Document-owned authoring freezes the ordinary description without retaining a request.
use super::*;
use crate::application_entry::{
    WorthQueryOperationConstructionDenial, WorthQueryOperationDescription,
    WorthQueryOperationInputCaptureDenial,
};
use worth_query_declaration::facade::{
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
        NoApplicationMutationSource,
    },
    application_program::{ApplicationAction, ApplicationProgramDefinition},
    application_schema::TypedMutationPreconditions,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

type Conditions<S, I> = TypedMutationPreconditions<
    S, <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::Operation,
    <<<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope,
>;
type Key<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::IdempotencyKey;

#[must_use = "retain the document-owned edit or cancel it before execution"]
pub struct WorthQueryHistoryEditBuilder<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    intent: I,
    conditions: Conditions<S, I>,
}

#[must_use = "retain the frozen edit in its declared program before advancing"]
pub struct WorthQueryHistoryEditDescription<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    document: WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
    description: WorthQueryOperationDescription<S, I>,
}

#[derive(Debug)]
pub enum WorthQueryHistoryEditCaptureDenial {
    ForeignRuntime,
    Input(WorthQueryOperationInputCaptureDenial),
}

pub struct WorthQueryHistoryEditCaptureFailure<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    Payload: WorthQueryHistoryPayloadMode<S, H> = OrdinaryHistoryPayload,
    Outputs = WorthQueryHistoryNoOutputs,
> {
    builder: WorthQueryHistoryEditBuilder<'runtime, S, H, I, Payload, Outputs>,
    key: Key<S, I>,
    denial: WorthQueryHistoryEditCaptureDenial,
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryHistoryEditCaptureFailure<'runtime, S, H, I, Payload, Outputs>
{
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryHistoryEditBuilder<'runtime, S, H, I, Payload, Outputs>,
        Key<S, I>,
        WorthQueryHistoryEditCaptureDenial,
    ) {
        (self.builder, self.key, self.denial)
    }
    pub fn denial(&self) -> &WorthQueryHistoryEditCaptureDenial {
        &self.denial
    }
}

impl<
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > std::fmt::Debug for WorthQueryHistoryEditCaptureFailure<'_, S, H, I, Payload, Outputs>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryHistoryEditCaptureFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>
{
    pub fn edit<I>(
        self,
        intent: I,
    ) -> WorthQueryHistoryEditBuilder<'runtime, S, H, I, Payload, Outputs>
    where
        I: ApplicationMutationIntent<S>,
        I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        WorthQueryHistoryEditBuilder {
            document: self,
            intent,
            conditions: TypedMutationPreconditions::default(),
        }
    }
}

impl<
        'runtime,
        S: ApplicationSchema,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryHistoryEditBuilder<'runtime, S, H, I, Payload, Outputs>
where
    I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub fn preconditions(mut self, conditions: Conditions<S, I>) -> Self {
        self.conditions = conditions;
        self
    }

    pub fn cancel(self) -> WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs> {
        self.document
    }

    pub fn idempotency_owned(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        key: Key<S, I>,
    ) -> Result<
        WorthQueryHistoryEditDescription<'runtime, S, H, I, Payload, Outputs>,
        WorthQueryHistoryEditCaptureFailure<'runtime, S, H, I, Payload, Outputs>,
    >
    where
        <I::Binding as ApplicationMutationBinding<S>>::Input: Clone + Send + Sync,
    {
        if !std::ptr::eq(self.document.owner.application(), request.application) {
            return Err(WorthQueryHistoryEditCaptureFailure {
                builder: self,
                key,
                denial: WorthQueryHistoryEditCaptureDenial::ForeignRuntime,
            });
        }
        let Self {
            document,
            intent,
            conditions,
        } = self;
        let current = WorthQueryApplicationRequest {
            application: document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: document.owner.branch(),
        };
        match current
            .mutate(intent)
            .without_source()
            .preconditions(conditions)
            .idempotency_owned(key)
        {
            Ok(description) => Ok(WorthQueryHistoryEditDescription {
                document,
                description,
            }),
            Err((builder, key, denial)) => Err(WorthQueryHistoryEditCaptureFailure {
                builder: Self {
                    document,
                    intent: builder.native.intent,
                    conditions: builder.native.preconditions,
                },
                key,
                denial: WorthQueryHistoryEditCaptureDenial::Input(denial),
            }),
        }
    }
}

impl<
        'runtime,
        S: ApplicationSchema + 'static,
        H: ApplicationHistoryBinding<S>,
        I: ApplicationMutationIntent<S>,
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs,
    > WorthQueryHistoryEditDescription<'runtime, S, H, I, Payload, Outputs>
{
    pub fn cancel(self) -> WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs> {
        self.document
    }

    pub fn retain_in_program<P, A>(
        self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        super::edit::WorthQueryHistoryEdit<'runtime, S, H, I, P, A, Payload, Outputs>,
        (Self, WorthQueryOperationConstructionDenial),
    >
    where
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, I::Binding>,
        crate::application_entry::operation::HistoryMode<S, I, A>:
            crate::application_entry::WorthQueryHistoryOperationMode<S, I, P, A>,
    {
        let Self {
            document,
            description,
        } = self;
        use crate::application_entry::operation::HistoryOperationDriver;
        match crate::application_entry::operation::HistoryMode::<S, I, A>::construct(
            description,
            program,
        ) {
            Ok(operation) => Ok(super::edit::WorthQueryHistoryEdit::new(document, operation)),
            Err((description, denial)) => Err((
                Self {
                    document,
                    description,
                },
                denial,
            )),
        }
    }
}
