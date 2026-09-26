use crate::runtime::active::WorthUiActiveRuntimeObservation;
use crate::runtime::{
    UiAllocationFrameDispatcherCounters, UiAllocationFrameDispatcherState,
    WorthUiLastValidObservation, WorthUiRuntimeFrameEpoch, WorthUiRuntimeLifecycle,
    WorthUiRuntimeShutdownReceipt,
};

#[cfg(test)]
use super::launch_request::WorthUiRuntimeLaunchDenial;
use super::runtime_instance::WorthUiRuntime;
use crate::runtime::session::service_proposal::UiServiceProposalTeardownDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiRuntimeShutdownBlocker {
    ServiceOwnerSettlementPending,
    ServiceProposalInvariant,
}

pub struct WorthUiRuntimeShutdownRecovery {
    runtime: Box<WorthUiRuntime>,
    blocker: WorthUiRuntimeShutdownBlocker,
}

impl WorthUiRuntimeShutdownRecovery {
    pub const fn blocker(&self) -> WorthUiRuntimeShutdownBlocker {
        self.blocker
    }

    pub fn into_runtime(self) -> WorthUiRuntime {
        *self.runtime
    }

    pub fn retry(self) -> Result<WorthUiRuntimeShutdownReceipt, Self> {
        (*self.runtime).shutdown()
    }
}

impl WorthUiRuntime {
    pub fn lifecycle(&self) -> WorthUiRuntimeLifecycle {
        self.active.lifecycle()
    }

    pub fn frame_epoch(&self) -> WorthUiRuntimeFrameEpoch {
        self.active.frame_epoch()
    }

    pub fn inspect_active(&self) -> WorthUiActiveRuntimeObservation {
        self.active.observation()
    }

    pub(crate) fn active_handle_arena_identity(
        &self,
    ) -> crate::runtime::WorthUiHandleArenaIdentity {
        self.active.handle_arena_identity()
    }

    pub(crate) fn active_plan_shares_lowering_identity_with(
        &self,
        identity: &crate::runtime::planning::WorthUiExecutionPlanLoweringIdentity,
    ) -> bool {
        self.active
            .active_plan_shares_lowering_identity_with(identity)
    }

    pub fn last_valid(&self) -> WorthUiLastValidObservation {
        self.last_valid.observation()
    }

    pub fn allocation_frame_dispatcher_state(&self) -> UiAllocationFrameDispatcherState {
        self.allocation_frame_scheduler.state()
    }

    pub fn allocation_frame_dispatcher_counters(&self) -> UiAllocationFrameDispatcherCounters {
        self.allocation_frame_scheduler.counters()
    }

    pub fn durable_semantic_state(
        &self,
    ) -> Option<crate::runtime::UiAllocationDurableSemanticState> {
        self.allocation_receipt_ledger.durable_semantic_state()
    }

    pub(crate) fn allocation_truth_revision(&self) -> crate::runtime::UiAllocationTruthRevision {
        self.allocation_receipt_ledger.truth_revision()
    }

    pub fn shutdown(
        mut self,
    ) -> Result<WorthUiRuntimeShutdownReceipt, WorthUiRuntimeShutdownRecovery> {
        let service_proposals = match self.service_proposals.shutdown_all_before_effect() {
            Ok(receipt) => receipt,
            Err(denial) => {
                let blocker = match denial {
                    UiServiceProposalTeardownDenial::AwaitingOwnerSettlement(_) => {
                        WorthUiRuntimeShutdownBlocker::ServiceOwnerSettlementPending
                    }
                    UiServiceProposalTeardownDenial::Census(_)
                    | UiServiceProposalTeardownDenial::Occupancy(_)
                    | UiServiceProposalTeardownDenial::Cancellation(_)
                    | UiServiceProposalTeardownDenial::ForeignProposal
                    | UiServiceProposalTeardownDenial::NonParticipatingFamily
                    | UiServiceProposalTeardownDenial::OwnerScopeMismatch
                    | UiServiceProposalTeardownDenial::ReasonMismatch
                    | UiServiceProposalTeardownDenial::DuplicateOwnerOutcome
                    | UiServiceProposalTeardownDenial::IncompleteOwnerDiscard => {
                        WorthUiRuntimeShutdownBlocker::ServiceProposalInvariant
                    }
                };
                return Err(WorthUiRuntimeShutdownRecovery {
                    runtime: Box::new(self),
                    blocker,
                });
            }
        };
        let queue_disposition = self.shutdown_allocation_frame_dispatcher();
        let query_retirement = self.query_binding.into_operation_live_retirement();
        Ok(WorthUiRuntimeShutdownReceipt::new(
            self.active.frame_epoch(),
            queue_disposition,
            query_retirement,
            service_proposals,
        ))
    }

    #[cfg(test)]
    pub(crate) fn reject_if_pending_activation_is_stale(
        &self,
        pending_activation: crate::runtime::WorthUiPendingActivation,
    ) -> Result<(), WorthUiRuntimeLaunchDenial> {
        let active_epoch = self.active.frame_epoch();
        let pending_epoch = pending_activation.frame_epoch();
        if pending_epoch == active_epoch {
            Ok(())
        } else {
            Err(WorthUiRuntimeLaunchDenial::StalePendingActivation {
                pending_epoch,
                active_epoch,
            })
        }
    }

    #[cfg(test)]
    pub(crate) fn advance_frame_epoch_for_test(&mut self) {
        self.active.advance_frame_epoch_for_test();
    }
}
