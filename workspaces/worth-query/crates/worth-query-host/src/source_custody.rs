//! Host custody of one source-only command across interactive frames.
//!
//! Each [`WorthQuerySourceCustody::resume`] borrows fresh request authority and
//! performs at most one native retention, one advance and one close. It returns
//! the command committed, declined, or pending with the exact owner it must be
//! resumed from. No request, principal or advancement phase is retained.
use worth_foundational::facade::ProgramSupportObligationIdentity;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationSourceCompletion,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationRequest, WorthQueryClosedOperation, WorthQueryCompletedSourceOperation,
    WorthQueryOperationCloseDenial, WorthQueryOperationConstructionDenial,
    WorthQueryOperationDescription, WorthQuerySourceCompletion, WorthQuerySourceCompletionAction,
    WorthQuerySourceMutation, WorthQuerySourceOnlyAdvance, WorthQuerySourceOnlyDenial,
    WorthQuerySourceOnlyOperation, WorthQuerySourceOnlyPosture, WorthQueryTerminalSourceEvidence,
    WorthQueryTerminalSourceOperation,
};

type Result<S, I> =
    <<I as ApplicationMutationIntent<S>>::Binding as ApplicationMutationBinding<S>>::Result;

mod erasure;
pub use erasure::{
    WorthQueryAnySourceCompletion, WorthQueryAnySourceCustody, WorthQueryAnySourceCustodyStep,
    WorthQueryAnyTerminalSourceEvidence,
};

/// The native owner of one source-completion command, in its current phase.
/// Every variant holds a genuine Publication owner; matching on it lends that
/// owner, and moving it out hands its exact custody back to direct Host calls.
#[must_use = "resume custody until it commits or declines; dropping it abandons the command"]
pub enum WorthQuerySourceCustody<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Captured input awaiting durable retention.
    Described(WorthQueryOperationDescription<S, I>),
    /// Retained operation awaiting its next native advance.
    Retained(WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>),
    /// Committed evidence awaiting cleanup close.
    Completed(WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>),
    /// Stopped operation awaiting close; its evidence is readable in place.
    Terminal(WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>),
}

/// What one resume established.
///
/// Drop a `Declined` step's evidence once it has been reported: it is a raw
/// native carrier, not a value to keep across frames.
#[must_use = "a pending step still owns the command"]
pub enum WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// The command committed and its cleanup closed.
    Committed {
        closed: WorthQueryClosedOperation,
        evidence: WorthQuerySourceCompletion<Result<S, I>>,
    },
    /// The command stopped without committing and its cleanup closed.
    Declined {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryTerminalSourceEvidence<S, I>,
    },
    /// The command still needs a later resume with fresh request authority.
    Pending {
        custody: WorthQuerySourceCustody<S, I, P, A>,
        cause: WorthQuerySourceCustodyPending,
    },
}

/// Why custody is pending. Each variant is the native owner's own refusal.
/// Match on the cause before resuming: `Close(NativeObligationRetained)`
/// means native recovery still owns the command, so resuming again in the
/// same state does not release it.
#[derive(Debug)]
pub enum WorthQuerySourceCustodyPending {
    /// Durable retention refused; the description is retained.
    Retention(WorthQueryOperationConstructionDenial),
    /// The operation progressed natively; resume continues it.
    Progress,
    /// The advance was refused; the operation is retained for a fresh retry.
    Refused(WorthQuerySourceOnlyDenial),
    /// Close of completed or terminal custody refused; the owner is retained.
    Close(WorthQueryOperationCloseDenial),
}

/// How an unexecuted command was lawfully withdrawn from Host custody without executing.
pub enum WorthQuerySourceCustodyWithdrawal {
    /// The command had not constructed a retained operation yet (and any durable original on its
    /// description was discharged).
    BeforeConstruction,
    /// A retained unexecuted operation closed its cleanup and discharged its durable original.
    Closed(WorthQueryClosedOperation),
}

impl std::fmt::Debug for WorthQuerySourceCustodyWithdrawal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeConstruction => f.write_str("BeforeConstruction"),
            Self::Closed(_) => f.write_str("Closed(..)"),
        }
    }
}

/// Descriptive lifecycle phase of a pending Host source-custody owner.
/// Read-only inspection; grants no admission, execution, or discharge authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQuerySourceCustodyPhase {
    /// Captured input awaiting durable retention.
    Described,
    /// Retained unexecuted operation (`Initial` posture) awaiting its next advance.
    RetainedInitial,
    /// Retained operation holding a post-handler or native recovery obligation.
    RetainedObligation,
    /// Committed evidence awaiting cleanup close.
    Completed,
    /// Stopped terminal evidence awaiting cleanup close.
    Terminal,
}

impl<S, I, P, A> WorthQuerySourceCustody<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    /// Descriptive lifecycle phase of the retained Publication owner.
    /// Grants no admission, execution, or discharge authority.
    pub fn phase(&self) -> WorthQuerySourceCustodyPhase {
        match self {
            Self::Described(_) => WorthQuerySourceCustodyPhase::Described,
            Self::Retained(operation) => match operation.posture() {
                WorthQuerySourceOnlyPosture::Initial => {
                    WorthQuerySourceCustodyPhase::RetainedInitial
                }
                _ => WorthQuerySourceCustodyPhase::RetainedObligation,
            },
            Self::Completed(_) => WorthQuerySourceCustodyPhase::Completed,
            Self::Terminal(_) => WorthQuerySourceCustodyPhase::Terminal,
        }
    }

    /// Descriptive identity of the retained durable original, if one is currently held.
    /// Grants no admission or discharge authority.
    pub fn durable_original_identity(&self) -> Option<ProgramSupportObligationIdentity> {
        match self {
            Self::Described(description) => description.original_identity(),
            Self::Retained(operation) => operation.original_identity(),
            Self::Completed(completed) => completed.original_identity(),
            Self::Terminal(terminal) => terminal.original_identity(),
        }
    }

    /// Descriptive check for whether the retained Publication owner currently holds an
    /// undischarged durable original (true on Directory Homes once retained, false on MemoryOnly).
    /// Grants no admission or discharge authority.
    pub fn has_durable_original(&self) -> bool {
        self.durable_original_identity().is_some()
    }

    /// Lawfully withdraws an unexecuted command without running its handler, discharging any
    /// retained durable original. Post-handler states (including `RetryableNoEffect`),
    /// post-execution phases (`Completed`, `Terminal`), and retained native recovery obligations
    /// refuse with `NativeObligationRetained` and return `self` intact.
    pub fn close_unexecuted(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> std::result::Result<
        WorthQuerySourceCustodyWithdrawal,
        (Self, WorthQueryOperationCloseDenial),
    > {
        match self {
            Self::Described(description) => description
                .close_unexecuted(request)
                .map(|()| WorthQuerySourceCustodyWithdrawal::BeforeConstruction)
                .map_err(|(description, denial)| (Self::Described(description), denial)),
            Self::Retained(operation) => operation
                .close_unexecuted(request)
                .map(WorthQuerySourceCustodyWithdrawal::Closed)
                .map_err(|(operation, denial)| (Self::Retained(operation), denial)),
            Self::Completed(completed) => Err((
                Self::Completed(completed),
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            )),
            Self::Terminal(terminal) => Err((
                Self::Terminal(terminal),
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            )),
        }
    }

    /// Transfers one native source-only advance into Host custody, closing completed or terminal
    /// owners immediately under `request` and retaining pending custody otherwise.
    pub fn accept_advance(
        advance: WorthQuerySourceOnlyAdvance<
            WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>,
            WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
            WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
        >,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQuerySourceCustodyStep<S, I, P, A> {
        match advance {
            WorthQuerySourceOnlyAdvance::Completed(completed) => {
                close_completed(completed, request)
            }
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => close_terminal(terminal, request),
            WorthQuerySourceOnlyAdvance::Progress(operation) => pending(
                WorthQuerySourceCustody::Retained(operation),
                WorthQuerySourceCustodyPending::Progress,
            ),
            WorthQuerySourceOnlyAdvance::Refused { operation, denial } => pending(
                WorthQuerySourceCustody::Retained(operation),
                WorthQuerySourceCustodyPending::Refused(denial),
            ),
        }
    }
}

impl<S, I, P, A> WorthQuerySourceCustody<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: WorthQuerySourceMutation<S>,
    P: ApplicationProgramDefinition<S>,
    A: WorthQuerySourceCompletionAction<S, I::Binding>,
{
    /// Advances this command by its next lawful native steps under `request`.
    pub fn resume(
        self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQuerySourceCustodyStep<S, I, P, A> {
        match self {
            Self::Described(description) => retain(description, runtime, request),
            Self::Retained(operation) => advance(operation, request),
            Self::Completed(completed) => close_completed(completed, request),
            Self::Terminal(terminal) => close_terminal(terminal, request),
        }
    }
}

fn pending<S, I, P, A>(
    custody: WorthQuerySourceCustody<S, I, P, A>,
    cause: WorthQuerySourceCustodyPending,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    WorthQuerySourceCustodyStep::Pending { custody, cause }
}

fn retain<S, I, P, A>(
    description: WorthQueryOperationDescription<S, I>,
    runtime: &WorthQueryProgramApplicationRuntime<S, P>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: WorthQuerySourceMutation<S>,
    P: ApplicationProgramDefinition<S>,
    A: WorthQuerySourceCompletionAction<S, I::Binding>,
{
    match description.retain_durable_operation::<P, A>(request, runtime) {
        Ok(operation) => advance(operation, request),
        Err((description, denial)) => pending(
            WorthQuerySourceCustody::Described(description),
            WorthQuerySourceCustodyPending::Retention(denial),
        ),
    }
}

fn advance<S, I, P, A>(
    operation: WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: WorthQuerySourceMutation<S>,
    P: ApplicationProgramDefinition<S>,
    A: WorthQuerySourceCompletionAction<S, I::Binding>,
{
    WorthQuerySourceCustody::accept_advance(operation.advance(request), request)
}

fn close_completed<S, I, P, A>(
    completed: WorthQueryCompletedSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    match completed.close_with_evidence(request) {
        Ok((closed, evidence)) => WorthQuerySourceCustodyStep::Committed { closed, evidence },
        Err((completed, denial)) => pending(
            WorthQuerySourceCustody::Completed(completed),
            WorthQuerySourceCustodyPending::Close(denial),
        ),
    }
}

fn close_terminal<S, I, P, A>(
    terminal: WorthQueryTerminalSourceOperation<S, I, P, A, ApplicationSourceCompletion>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
) -> WorthQuerySourceCustodyStep<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    match terminal.close_with_evidence(request) {
        Ok((closed, evidence)) => WorthQuerySourceCustodyStep::Declined { closed, evidence },
        Err((terminal, denial)) => pending(
            WorthQuerySourceCustody::Terminal(terminal),
            WorthQuerySourceCustodyPending::Close(denial),
        ),
    }
}
