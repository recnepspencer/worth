//! Retained kernel commands borrow fresh authority and share native publication custody.
use super::operation::native_publication::{NativePublicationState, NativeRecoveredPublication};
use super::{
    WorthQueryApplicationRequest, WorthQueryClosedOperation, WorthQueryOperationCleanup,
    WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial,
};
use std::convert::Infallible;
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::{
    application_contribution::{WorthQueryAdvancementDenial, WorthQueryAdvancementPhase},
    application_installation::WorthQueryProgramApplicationRuntime,
    domain_computation::WorthQueryProductUnpublishedRecovery,
    primary_graph::{
        WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationCommitOutcome,
        WorthQueryApplicationCommitReceipt, WorthQueryApplicationIdempotencyResolution,
        WorthQueryApplicationUncommitted, WorthQueryManagedApplicationRecoveryOutcome,
        WorthQueryManagedApplicationRecoveryPerformed,
    },
    product::WorthQueryProductBranch,
};
use worth_query_installation::facade::ApplicationSchema;

mod advancement;
mod completion;
mod delegation;
mod revocation;
pub use completion::{WorthQueryCompletedCapabilityCommand, WorthQueryTerminalCapabilityCommand};

mod backend;
use backend::{NativeCommandAction, NativeCommandBackend, NativeCommandResult};

/// A kernel command owns original intent, never a request, admission or actor grant.
#[must_use = "advance or lawfully close the retained native command"]
pub struct WorthQueryCapabilityCommand<
    'runtime,
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    D,
> {
    runtime: WorthQueryProgramApplicationRuntime<S, P>,
    branch: WorthQueryProductBranch,
    backend: Box<dyn NativeCommandBackend<S, D> + 'runtime>,
    state: NativePublicationState<WorthQueryApplicationUncommitted, Infallible, Infallible>,
    cleanup: WorthQueryOperationCleanup,
    keyed_terminal: Option<WorthQueryApplicationIdempotencyResolution>,
}
impl<'r, S: ApplicationSchema, P: ApplicationProgramDefinition<S>, D>
    WorthQueryCapabilityCommand<'r, S, P, D>
{
    /// Describes this native command's current publication custody.
    pub fn posture(&self) -> super::WorthQuerySourceOnlyPosture {
        use super::WorthQuerySourceOnlyPosture as Posture;
        use NativePublicationState as State;
        match &self.state {
            State::Initial => Posture::Initial,
            State::RetryableNoEffect(_) => Posture::RetryableNoEffect,
            State::Unpublished(_) => Posture::ProductUnpublished,
            State::Performed(_) => Posture::RecoveryPerformed,
            State::Keyed(WorthQueryApplicationUncommitted::Aborted) => Posture::ProvedAborted,
            State::Keyed(_) | State::UnresolvedAdvancement(_) => Posture::Indeterminate,
            State::Stopped(_) => Posture::Terminal,
            State::AdmissionStopped(never) | State::RetainedPending(never) => match *never {},
        }
    }

    fn new(
        runtime: WorthQueryProgramApplicationRuntime<S, P>,
        branch: WorthQueryProductBranch,
        backend: impl NativeCommandBackend<S, D> + 'r,
    ) -> Self {
        Self {
            runtime,
            branch,
            backend: Box::new(backend),
            state: NativePublicationState::Initial,
            cleanup: WorthQueryOperationCleanup::default(),
            keyed_terminal: None,
        }
    }
}

/// Refusal preserves the same command and its native cause.
#[derive(Debug)]
pub enum WorthQueryCapabilityCommandDenial<D> {
    FreshRequestMismatch,
    Native(D),
    Execution(WorthQueryAdvancementDenial),
    Cleanup(WorthQueryOperationCloseDenial),
}
/// A command has no domain-handler result or manufactured output.
pub enum WorthQueryCapabilityCommandCompletion {
    Fresh {
        receipt: WorthQueryApplicationCommitReceipt,
    },
    Recovered {
        receipt: WorthQueryApplicationCommitReceipt,
        publication: Option<WorthQueryManagedApplicationRecoveryPerformed>,
        initial_posture: Option<WorthQueryApplicationUncommitted>,
        initial_advancement_failure: Option<WorthQueryAdvancementDenial>,
    },
}
impl WorthQueryCapabilityCommandCompletion {
    /// Original native evidence; borrowing cannot transfer its performed witness.
    pub fn receipt(&self) -> &WorthQueryApplicationCommitReceipt {
        match self {
            Self::Fresh { receipt } | Self::Recovered { receipt, .. } => receipt,
        }
    }
    fn recovered(value: NativeRecoveredPublication) -> Self {
        Self::Recovered {
            receipt: value.receipt,
            publication: value.publication,
            initial_posture: value.initial_posture,
            initial_advancement_failure: value.initial_advancement_failure,
        }
    }
}
/// Progress is custody, completion is native evidence, and terminality grants no retry.
pub enum WorthQueryCapabilityCommandAdvance<Command, Completed, Terminal, D> {
    Progress(Command),
    Refused {
        command: Command,
        denial: WorthQueryCapabilityCommandDenial<D>,
    },
    Completed(Completed),
    Terminal(Terminal),
}
