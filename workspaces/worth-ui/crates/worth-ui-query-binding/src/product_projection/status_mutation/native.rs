//! Concrete retained-commit operations; discriminants preserve genuine native custody.
use super::super::application_program::{
    WorthUiStatusAction, WorthUiStatusProgram, WorthUiStatusUpdateAction,
};
use super::*;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequest, WorthQueryCompletedSourceOperation,
        WorthQueryOperationDescription, WorthQueryOperationInputCaptureRefusal,
        WorthQuerySourceCompletion, WorthQuerySourceOnlyAdvance, WorthQuerySourceOnlyOperation,
        WorthQueryTerminalSourceOperation,
    },
    declaration::{
        application_operation::ApplicationMutationIntent,
        application_program::ApplicationRetainedCommit,
    },
};

mod close;
mod observation;

pub(super) enum Step {
    Ready,
    Progress,
    Terminal,
}

pub(super) enum State<I: ApplicationMutationIntent<WorthUiApplicationSchema> + 'static, A> {
    // Each phase owns its original carrier; transitions move bounded custody slots.
    Original(Box<I>),
    Capture(Box<WorthQueryOperationInputCaptureRefusal<WorthUiApplicationSchema, I>>),
    Description(Box<WorthQueryOperationDescription<WorthUiApplicationSchema, I>>),
    Pending(
        Box<
            WorthQuerySourceOnlyOperation<
                WorthUiApplicationSchema,
                I,
                WorthUiStatusProgram,
                A,
                ApplicationRetainedCommit,
            >,
        >,
    ),
    Completed(
        Box<
            WorthQueryCompletedSourceOperation<
                WorthUiApplicationSchema,
                I,
                WorthUiStatusProgram,
                A,
                ApplicationRetainedCommit,
            >,
        >,
    ),
    Terminal(
        Box<
            WorthQueryTerminalSourceOperation<
                WorthUiApplicationSchema,
                I,
                WorthUiStatusProgram,
                A,
                ApplicationRetainedCommit,
            >,
        >,
    ),
}

pub(super) enum NativeCustody {
    Source(State<WorthUiScalarProjectionSourceRecord, WorthUiStatusUpdateAction>),
    Action(State<WorthUiStatusActionRequest, WorthUiStatusAction>),
}

// Both concrete bindings share this exact native lifecycle, not a second execution engine.
macro_rules! retained_status {
    ($input:ty, $action:ty, $key:expr) => {
        impl State<$input, $action> {
            fn prepare(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
                source: &WorthQueryObservedSource<WorthUiStatusQuery>,
                application: &WorthUiStatusApplication,
            ) -> (Self, Result<(), WorthUiStatusMutationStop>) {
                let mut description = match self {
                    Self::Original(input) => {
                        let key = ($key)(&input);
                        match request
                            .mutate(*input)
                            .expect_source(source.clone())
                            .idempotency_owned(key)
                        {
                            Ok(description) => Box::new(description),
                            Err((builder, key, denial)) => {
                                return (
                                    Self::Capture(Box::new(
                                        builder.into_capture_refusal(key, denial),
                                    )),
                                    Err(WorthUiStatusMutationStop::Capture(
                                        WorthQueryOperationInputCaptureRetryDenial::Capture,
                                    )),
                                );
                            }
                        }
                    }
                    Self::Capture(mut original) => match (*original).retry_capture(request) {
                        Ok(description) => Box::new(description),
                        Err((returned, denial)) => {
                            *original = returned;
                            return (
                                Self::Capture(original),
                                Err(WorthUiStatusMutationStop::Capture(denial)),
                            );
                        }
                    },
                    Self::Description(description) => description,
                    other => return (other, Ok(())),
                };
                match (*description).retain_operation::<WorthUiStatusProgram, $action>(application)
                {
                    Ok(operation) => (Self::Pending(Box::new(operation)), Ok(())),
                    Err((returned, denial)) => {
                        *description = returned;
                        (
                            Self::Description(description),
                            Err(WorthUiStatusMutationStop::Construction(denial)),
                        )
                    }
                }
            }
            fn advance(
                self,
                request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
                history: NonZeroUsize,
            ) -> (Self, Result<Step, WorthUiStatusMutationStop>) {
                let Self::Pending(mut operation) = self else {
                    return match self {
                        Self::Completed(_) => (self, Ok(Step::Ready)),
                        Self::Terminal(_) => (self, Ok(Step::Terminal)),
                        _ => unreachable!("production preparation precedes advancement"),
                    };
                };
                match (*operation).advance(request, history) {
                    WorthQuerySourceOnlyAdvance::Progress(returned) => {
                        *operation = returned;
                        (Self::Pending(operation), Ok(Step::Progress))
                    }
                    WorthQuerySourceOnlyAdvance::Refused {
                        operation: returned,
                        denial,
                    } => {
                        *operation = returned;
                        (
                            Self::Pending(operation),
                            Err(WorthUiStatusMutationStop::Advance(denial)),
                        )
                    }
                    WorthQuerySourceOnlyAdvance::Completed(operation) => {
                        (Self::Completed(Box::new(operation)), Ok(Step::Ready))
                    }
                    WorthQuerySourceOnlyAdvance::Terminal(operation) => {
                        (Self::Terminal(Box::new(operation)), Ok(Step::Terminal))
                    }
                }
            }
        }
    };
}
retained_status!(
    WorthUiScalarProjectionSourceRecord,
    WorthUiStatusUpdateAction,
    |input: &WorthUiScalarProjectionSourceRecord| input.revision()
);
retained_status!(
    WorthUiStatusActionRequest,
    WorthUiStatusAction,
    |input: &WorthUiStatusActionRequest| input.identity()
);

impl NativeCustody {
    pub(super) fn source(input: WorthUiScalarProjectionSourceRecord) -> Self {
        Self::Source(State::Original(Box::new(input)))
    }
    pub(super) fn action(input: WorthUiStatusActionRequest) -> Self {
        Self::Action(State::Original(Box::new(input)))
    }
    pub(super) fn prepare(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        source: &WorthQueryObservedSource<WorthUiStatusQuery>,
        application: &WorthUiStatusApplication,
    ) -> (Self, Result<(), WorthUiStatusMutationStop>) {
        match self {
            Self::Source(state) => {
                let (state, result) = state.prepare(request, source, application);
                (Self::Source(state), result)
            }
            Self::Action(state) => {
                let (state, result) = state.prepare(request, source, application);
                (Self::Action(state), result)
            }
        }
    }
    pub(super) fn advance(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, WorthUiApplicationSchema>,
        history: NonZeroUsize,
    ) -> (Self, Result<Step, WorthUiStatusMutationStop>) {
        match self {
            Self::Source(state) => {
                let (state, result) = state.advance(request, history);
                (Self::Source(state), result)
            }
            Self::Action(state) => {
                let (state, result) = state.advance(request, history);
                (Self::Action(state), result)
            }
        }
    }
}
