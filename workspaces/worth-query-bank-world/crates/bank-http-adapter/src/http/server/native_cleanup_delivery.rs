//! A reserved transport slot returns unique evidence when a handoff reply is not consumed.
use super::{
    authenticated_owner::BankHttpAuthenticatedOwner, mutation_executor::WorkerCommand,
    native_cleanup_custody::BankHttpReleasedCleanup,
};
use std::sync::Arc;
use tokio::sync::mpsc::OwnedPermit;

pub(super) struct CleanupDelivery {
    id: String,
    owner: BankHttpAuthenticatedOwner,
    sequence: u64,
    evidence: Option<bank_server::BankReleasedManagedCleanup>,
    runtime: Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
    slot: Option<OwnedPermit<WorkerCommand>>,
}
impl CleanupDelivery {
    pub(super) fn new(
        id: String,
        owner: BankHttpAuthenticatedOwner,
        sequence: u64,
        evidence: bank_server::BankReleasedManagedCleanup,
        runtime: Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
        slot: OwnedPermit<WorkerCommand>,
    ) -> Self {
        Self {
            id,
            owner,
            sequence,
            evidence: Some(evidence),
            runtime,
            slot: Some(slot),
        }
    }
    pub(super) fn receive(mut self) -> BankHttpReleasedCleanup {
        let evidence = self.evidence.take().expect("unique host delivery");
        self.slot
            .take()
            .expect("reserved acknowledgement")
            .send(WorkerCommand::ConfirmHandoff {
                id: self.id.clone(),
            });
        BankHttpReleasedCleanup {
            runtime: Arc::clone(&self.runtime),
            evidence,
        }
    }
}
impl Drop for CleanupDelivery {
    fn drop(&mut self) {
        if let Some(evidence) = self.evidence.take() {
            self.slot
                .take()
                .expect("reserved return")
                .send(WorkerCommand::ReturnHandoff {
                    id: self.id.clone(),
                    owner: self.owner.clone(),
                    sequence: self.sequence,
                    evidence,
                });
        }
        // Queue ownership only: no native scope, I/O, release, publication, or business settlement.
    }
}
