//! A no-output operation owns original meaning and every native publication obligation.
use super::*;
use worth_query_declaration::facade::application_program::ApplicationAction;
use worth_query_execution::facade::domain_computation::WorthQueryProductUnpublishedRecovery;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryApplicationUncommitted,
    WorthQueryManagedApplicationRecoveryPerformed,
};

mod advancement;
pub(super) use advancement::Step as SourceStep;
mod completion;
mod entrance;
mod native_posture;
mod retained_delivery;
pub use completion::{WorthQueryCompletedSourceOperation, WorthQueryTerminalSourceOperation};

type NativeOutcome<S, I> = WorthQueryApplicationMutationOutcome<
    <Binding<S, I> as ApplicationMutationBinding<S>>::Denial,
    <Binding<S, I> as ApplicationMutationBinding<S>>::Result,
>;

/// Move-only custody with an owning keepalive to the exact installed runtime.
/// Every advancement borrows fresh request authority; no request or actor grant is retained.
/// Construction can leave the initiating runtime borrow's frame.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_schema::ApplicationSchema, application_operation::ApplicationMutationIntent, application_program::*};
/// use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
/// fn transfer<S, I, P, A>(description: WorthQueryOperationDescription<S, I>, runtime: &WorthQueryProgramApplicationRuntime<S, P>) -> Result<WorthQuerySourceOnlyOperation<S, I, P, A, ApplicationSourceCompletion>, (WorthQueryOperationDescription<S, I>, WorthQueryOperationConstructionDenial)>
/// where S: ApplicationSchema + 'static, I: ApplicationMutationIntent<S>, I::Binding: 'static, P: ApplicationProgramDefinition<S>, A: ApplicationAction<S, I::Binding>, A::Delivery: ApplicationActionDelivery<S, I::Binding, Mode = ApplicationSourceCompletion> {
///     description.retain_operation::<P, A>(runtime)
/// }
/// ```
#[must_use = "retain publication custody until completion or lawful close"]
pub struct WorthQuerySourceOnlyOperation<S, I, P, A, Mode>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
{
    cleanup: WorthQueryOperationCleanup,
    state: SourceState<S, I>,
    guarded_abort: Option<WorthQueryApplicationUncommitted>,
    keyed_terminal: Option<
        worth_query_execution::facade::primary_graph::WorthQueryApplicationIdempotencyResolution,
    >,
    // Keep the authentic runtime alive while every bounded native carrier drops.
    pub(in crate::application_entry) admission:
        WorthQuerySourceOperationAdmission<S, I, P, A, Mode>,
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
> WorthQuerySourceOnlyOperation<S, I, P, A, M>
{
    /// Abandons ordinary retry meaning while keeping the exact native cleanup authority alive.
    ///
    /// This performs no release, settlement, publication, or request admission. Native recovery
    /// records remain discoverable on the returned runtime; the returned cleanup collection
    /// retains actual failed managed-run custody that has no publication recovery record.
    /// The original ordinary operation cannot be resumed after this explicit handoff. Refused
    /// native cleanup retains its own recovery/cleanup handle. Final runtime destruction and
    /// crash durability are not guaranteed by this handoff.
    pub fn into_runtime_for_cleanup(
        self,
    ) -> (
        WorthQueryProgramApplicationRuntime<S, P>,
        WorthQueryOperationCleanup,
    ) {
        (self.admission.runtime, self.cleanup)
    }

    /// Exact managed-run cleanup custody retained independently of business evidence.
    pub fn managed_cleanup_failure(
        &self,
    ) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryMutationCleanupFailure>
    {
        self.cleanup.managed_failure()
    }
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

impl<S, I, P, A, Mode> WorthQuerySourceOnlyOperation<S, I, P, A, Mode>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode: WorthQuerySourceDeliveryMode,
{
    pub(super) fn construct(
        description: WorthQueryOperationDescription<S, I>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
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
            guarded_abort: None,
        })
    }
}

impl<
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A,
    M,
> WorthQuerySourceOnlyOperation<S, I, P, A, M>
{
    pub(super) fn guarded_abort(&self) -> Option<&WorthQueryApplicationUncommitted> {
        self.guarded_abort.as_ref()
    }
    pub(super) fn record_unresolved_advancement(
        &mut self,
        denial:worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial,
    ) {
        if matches!(denial,worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Panicked)
            && matches!(self.state,SourceState::Initial|SourceState::RetryableNoEffect(_)) {self.state=SourceState::UnresolvedAdvancement(denial);}
    }
}
