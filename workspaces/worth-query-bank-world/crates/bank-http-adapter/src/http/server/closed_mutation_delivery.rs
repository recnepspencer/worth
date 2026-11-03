//! A reserved return returns the exact closed record if a reply is lost.
use super::{
    authenticated_owner::BankHttpAuthenticatedOwner,
    closed_mutation_custody::BankHttpClosedMutation, closed_mutation_custody::ClosedMutationRecord,
    mutation_executor::WorkerCommand,
};
use std::sync::Arc;
use tokio::sync::mpsc::OwnedPermit;
/// Unaccepted delivery returns the exact closed owner to its worker.
#[must_use = "receive the native owner or drop this delivery to return it"]
pub struct BankHttpClosedMutationDelivery {
    id: String,
    record: Option<ClosedMutationRecord>,
    runtime: Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
    slot: Option<OwnedPermit<WorkerCommand>>,
}
impl BankHttpClosedMutationDelivery {
    pub(super) fn new(
        id: String,
        record: ClosedMutationRecord,
        runtime: Arc<dyn super::inbound_completion::BankRailCompletionRuntime>,
        slot: OwnedPermit<WorkerCommand>,
    ) -> Self {
        Self {
            id,
            record: Some(record),
            runtime,
            slot: Some(slot),
        }
    }
    pub fn receive(mut self) -> BankHttpClosedMutation {
        let record = self.record.take().expect("unique closed host delivery");
        self.slot
            .take()
            .expect("reserved close acknowledgement")
            .send(WorkerCommand::ConfirmClosedHandoff {
                id: self.id.clone(),
            });
        BankHttpClosedMutation {
            runtime: Arc::clone(&self.runtime),
            evidence: record.evidence,
        }
    }
}
impl Drop for BankHttpClosedMutationDelivery {
    fn drop(&mut self) {
        if let Some(record) = self.record.take() {
            self.slot.take().expect("reserved closed return").send(
                WorkerCommand::ReturnClosedHandoff {
                    id: self.id.clone(),
                    record,
                },
            );
        }
        // No native action, I/O or logical settlement in Drop.
    }
}
pub(super) fn eligible(
    record: &ClosedMutationRecord,
    owner: &BankHttpAuthenticatedOwner,
    sequence: u64,
) -> bool {
    &record.owner == owner && record.sequence.checked_add(1) == Some(sequence)
}
