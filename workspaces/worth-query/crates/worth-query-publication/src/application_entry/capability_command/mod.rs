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
        WorthQueryApplicationSettlementDeferred, WorthQueryApplicationUncommitted,
        WorthQueryManagedApplicationRecoveryOutcome, WorthQueryManagedApplicationRecoveryPerformed,
    },
    product::WorthQueryProductBranch,
};
use worth_query_installation::facade::ApplicationSchema;

mod advancement;
mod completion;
mod delegation;
mod revocation;
pub use completion::{WorthQueryCompletedCapabilityCommand, WorthQueryTerminalCapabilityCommand};

pub(super) enum NativeCommandAction<'a> {
    Validate,
    Execute,
    Resolve,
    Recover(&'a WorthQueryProductUnpublishedRecovery),
    Settle(&'a WorthQueryApplicationSettlementDeferred),
}
pub(super) enum NativeCommandResult {
    Validated,
    Commit(WorthQueryApplicationCommitOutcome),
    Resolved(WorthQueryAdmittedIdempotencyRead),
    Recovered(WorthQueryManagedApplicationRecoveryOutcome),
}
// This contract and every implementer are private to Publication.
pub(super) trait NativeCommandBackend<S: ApplicationSchema, D> {
    fn run(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        action: NativeCommandAction<'_>,
    ) -> Result<NativeCommandResult, D>;
}
impl<S: ApplicationSchema, D, F> NativeCommandBackend<S, D> for F
where
    F: for<'p, 'e, 'q, 'a, 'b, 'c, 'x> FnMut(
        &'p WorthQueryAdvancementPhase<'e>,
        &'q WorthQueryApplicationRequest<'a, 'b, 'c, S>,
        NativeCommandAction<'x>,
    ) -> Result<NativeCommandResult, D>,
{
    fn run(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        action: NativeCommandAction<'_>,
    ) -> Result<NativeCommandResult, D> {
        self(phase, request, action)
    }
}

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
    pub(super) fn new(
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
