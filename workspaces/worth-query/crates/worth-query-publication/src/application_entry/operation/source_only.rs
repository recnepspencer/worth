//! A no-output operation owns original meaning and every native publication obligation.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationAction;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
    WorthQueryManagedApplicationRecoveryPerformed,
};

mod advancement;
mod completion;
mod entrance;
mod native_posture;
mod retained_delivery;
pub use completion::{WorthQueryCompletedSourceOperation, WorthQueryTerminalSourceOperation};

type NativeOutcome<S, I> = WorthQueryApplicationMutationOutcome<
    <Binding<S, I> as ApplicationMutationBinding<S>>::Denial,
    <Binding<S, I> as ApplicationMutationBinding<S>>::Result,
>;

/// Move-only custody; every advancement borrows fresh request authority.
#[must_use = "retain publication custody until completion or lawful close"]
pub struct WorthQuerySourceOnlyOperation<'r, S, I, P, A, Mode>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    admission: WorthQuerySourceOperationAdmission<'r, S, I, P, A, Mode>,
    cleanup: WorthQueryOperationCleanup,
    state: SourceState<S, I>,
    keyed_terminal: Option<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
    >,
}

enum SourceState<S: ApplicationSchema, I: ApplicationMutationIntent<S>> {
    Initial,
    Unpublished(WorthQueryProductUnpublishedRecovery),
    Performed(WorthQueryManagedApplicationRecoveryPerformed),
    Keyed(WorthQueryApplicationUncommitted),
    UnresolvedAdvancement(
        worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ),
    RetryableNoEffect(NativeOutcome<S, I>),
    Stopped(NativeOutcome<S, I>),
    AdmissionStopped(WorthQueryApplicationRequestMutationDenial),
    RetainedPending(
        WorthQuerySourceCompletion<<I::Binding as ApplicationMutationBinding<S>>::Result>,
    ),
}

/// Fresh results and recovered receipt evidence never imply the same result provenance.
pub enum WorthQuerySourceCompletion<Result> {
    Fresh {
        receipt: WorthQueryApplicationCommitReceipt,
        result: Result,
        retained: Option<WorthQueryApplicationReadObservation>,
    },
    Recovered {
        retained: Option<WorthQueryApplicationReadObservation>,
        receipt: WorthQueryApplicationCommitReceipt,
        publication: Option<WorthQueryManagedApplicationRecoveryPerformed>,
        initial_posture: Option<WorthQueryApplicationUncommitted>,
        initial_advancement_failure: Option<
            worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
        >,
    },
}

/// Native progress returns custody, completion owns close, and terminal custody cannot advance.
pub enum WorthQuerySourceOnlyAdvance<Operation, Completed, Terminal> {
    Progress(Operation),
    Refused {
        operation: Operation,
        denial: WorthQuerySourceOnlyDenial,
    },
    Completed(Completed),
    Terminal(Terminal),
}

/// Native refusal returned alongside the retained operation for a fresh-authorized retry.
#[derive(Debug)]
pub enum WorthQuerySourceOnlyDenial {
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Mutation(WorthQueryApplicationRequestMutationDenial),
    Recovery(WorthQueryApplicationRecoveryRequestDenial),
    Cleanup(WorthQueryOperationCloseDenial),
    History(WorthQueryApplicationHistorySelectionDenial),
}

/// A descriptive native posture; it grants neither admission nor recovery authority.
#[derive(Debug, Eq, PartialEq)]
pub enum WorthQuerySourceOnlyPosture {
    Initial,
    ProductUnpublished,
    RecoveryPerformed,
    RetainedDeliveryPending,
    ProvedAborted,
    RetryableNoEffect,
    Indeterminate,
    Terminal,
}
impl<
        S: ApplicationSchema,
        I: ApplicationMutationIntent<S>,
        P: ApplicationProgramDefinition<S>,
        A,
        M,
    > WorthQuerySourceOnlyOperation<'_, S, I, P, A, M>
{
    /// Exact completed native no-effect evidence retained for a fresh admitted attempt.
    pub fn native_retry_outcome(&self) -> Option<&NativeOutcome<S, I>> {
        match &self.state {
            SourceState::RetryableNoEffect(outcome) => Some(outcome),
            _ => None,
        }
    }
    pub fn posture(&self) -> WorthQuerySourceOnlyPosture {
        match self.state {
            SourceState::Initial => WorthQuerySourceOnlyPosture::Initial,
            SourceState::RetryableNoEffect(_) => WorthQuerySourceOnlyPosture::RetryableNoEffect,
            SourceState::Unpublished(_) => WorthQuerySourceOnlyPosture::ProductUnpublished,
            SourceState::Performed(_) => WorthQuerySourceOnlyPosture::RecoveryPerformed,
            SourceState::Keyed(WorthQueryApplicationUncommitted::Aborted) => {
                WorthQuerySourceOnlyPosture::ProvedAborted
            }
            SourceState::Keyed(_) => WorthQuerySourceOnlyPosture::Indeterminate,
            SourceState::UnresolvedAdvancement(_) => WorthQuerySourceOnlyPosture::Indeterminate,
            SourceState::RetainedPending(_) => WorthQuerySourceOnlyPosture::RetainedDeliveryPending,
            SourceState::Stopped(_) | SourceState::AdmissionStopped(_) => {
                WorthQuerySourceOnlyPosture::Terminal
            }
        }
    }
}

impl<'r, S, I, P, A, Mode> WorthQuerySourceOnlyOperation<'r, S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
{
    pub(super) fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &'r WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        Self,
        (
            WorthQueryOperationDescription<S, I>,
            WorthQueryOperationConstructionDenial,
        ),
    > {
        WorthQuerySourceOperationAdmission::construct(description, runtime).map(|admission| Self {
            admission,
            cleanup: Default::default(),
            state: SourceState::Initial,
            keyed_terminal: None,
        })
    }
}
