//! Native cleanup evidence retains the original operation's remaining custody.
use super::WorthQueryPendingApplicationCommitReceipt as Original;
use crate::domain_computation::application_aftermath::WorthQueryExternalEffectDispatch;
use crate::domain_computation::{
    WorthQueryDirectRunCleanupInspection, WorthQueryDirectRunCleanupReceipt,
};

// The concrete slot is funded before effects and filled only by its active original.
pub(in crate::domain_computation) struct RetainedOriginalCleanupCommit(
    std::sync::Arc<dyn crate::domain_computation::primary_graph::provider::OriginalCompletionSlot>,
);
impl RetainedOriginalCleanupCommit {
    pub(in crate::domain_computation::primary_graph) fn new(
        slot: std::sync::Arc<
            dyn crate::domain_computation::primary_graph::provider::OriginalCompletionSlot,
        >,
    ) -> Self {
        Self(slot)
    }
    pub(in crate::domain_computation) fn original(&self) -> &Original {
        self.0.original_cleanup()
    }
}

/// First-send disposition is separate from business completion and recovered data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCleanupDispatchPosture {
    NotDeclared,
    Pending,
    AttemptedUnresolved,
    Observed,
    TerminalObserved,
    DiagnosticCopy,
}
/// Actual cleanup proof plus any original outbox/recovery obligation.
/// A copy describes cleanup but cannot resume the original operation.
#[must_use = "retain remaining original dispatch/recovery custody"]
pub struct WorthQueryApplicationManagedCleanupReceipt {
    native: WorthQueryDirectRunCleanupReceipt,
    obligation: Option<worth_foundational::facade::ManagedCleanupObligationIdentity>,
    pub(in crate::domain_computation) original: Option<RetainedOriginalCleanupCommit>,
    pub(in crate::domain_computation) posture: WorthQueryCleanupDispatchPosture,
    pub(in crate::domain_computation) dispatch: Option<WorthQueryExternalEffectDispatch>,
}
impl WorthQueryApplicationManagedCleanupReceipt {
    pub(super) fn new(
        native: WorthQueryDirectRunCleanupReceipt,
        original: Option<RetainedOriginalCleanupCommit>,
        obligation: Option<worth_foundational::facade::ManagedCleanupObligationIdentity>,
    ) -> Self {
        let posture = if original
            .as_ref()
            .is_some_and(|original| original.original().dispatch_binding().is_some())
        {
            WorthQueryCleanupDispatchPosture::Pending
        } else {
            WorthQueryCleanupDispatchPosture::NotDeclared
        };
        Self {
            native,
            obligation,
            original,
            posture,
            dispatch: None,
        }
    }
    pub fn inspection(&self) -> &WorthQueryDirectRunCleanupInspection {
        self.native.inspection()
    }
    pub fn obligation_identity(
        &self,
    ) -> Option<worth_foundational::facade::ManagedCleanupObligationIdentity> {
        self.obligation
    }
    pub fn dispatch_posture(&self) -> WorthQueryCleanupDispatchPosture {
        self.posture
    }
    pub fn original_dispatch(&self) -> Option<&WorthQueryExternalEffectDispatch> {
        self.dispatch.as_ref()
    }
}
impl Clone for WorthQueryApplicationManagedCleanupReceipt {
    fn clone(&self) -> Self {
        Self {
            native: self.native.clone(),
            obligation: self.obligation,
            original: None,
            posture: WorthQueryCleanupDispatchPosture::DiagnosticCopy,
            dispatch: self.dispatch.clone(),
        }
    }
}
impl std::fmt::Debug for WorthQueryApplicationManagedCleanupReceipt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationManagedCleanupReceipt")
            .field("native", &self.native)
            .field("dispatch_posture", &self.posture)
            .finish()
    }
}
impl PartialEq for WorthQueryApplicationManagedCleanupReceipt {
    fn eq(&self, other: &Self) -> bool {
        self.native == other.native
            && self.obligation == other.obligation
            && self.posture == other.posture
            && self.dispatch == other.dispatch
    }
}

/// Exact refusal before or after attempting the original outstanding delivery.
#[derive(Debug)]
pub enum WorthQueryCleanupDispatchDenial {
    OriginalNotOwned,
    ForeignRuntime,
    AlreadyAttempted,
    AlreadyCompleted,
    TransportNotInstalled,
    Cancelled,
    TimedOut,
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    OwnerReadDenied(
        crate::domain_computation::primary_graph::WorthQueryCommittedDispatchOutboxReadDenial,
    ),
    DispatchDenied(
        crate::domain_computation::primary_graph::WorthQueryExternalDispatchPreparationDenial,
    ),
}
