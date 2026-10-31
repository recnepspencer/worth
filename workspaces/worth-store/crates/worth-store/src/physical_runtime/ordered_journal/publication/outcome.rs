//! Owner-issued correlation and actual terminal evidence; neither starts work.
use super::super::*;
use crate::physical_runtime::{
    AnchoredRecordStepSettlementFailure, PhysicalMutationIdentity, PhysicalMutationOutcome,
    RuntimeIdentity,
};
use std::sync::Arc;
/// In-memory owner correlation. It grants no work and is not durable identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrderedJournalPublicationSource {
    pub(super) runtime: RuntimeIdentity,
    pub(super) journal: OrderedJournalName,
    pub(super) position: u64,
    pub(super) digest: [u8; 32],
    pub(super) bytes: u64,
}
impl OrderedJournalPublicationSource {
    pub fn runtime_identity(self) -> RuntimeIdentity {
        self.runtime
    }
    pub fn journal(self) -> OrderedJournalName {
        self.journal
    }
    pub fn position(self) -> u64 {
        self.position
    }
    pub fn payload_sha256(self) -> [u8; 32] {
        self.digest
    }
    pub fn payload_bytes(self) -> u64 {
        self.bytes
    }
}
pub struct OrderedJournalPublicationTerminal {
    pub(super) source: OrderedJournalPublicationSource,
    pub(super) mutation: Option<PhysicalMutationIdentity>,
    pub(super) outcome: Option<PhysicalMutationOutcome>,
    pub(super) published: Option<OrderedJournalPublished>,
    pub(super) failure: Option<OrderedJournalTerminalFailure>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderedJournalTerminalFailure {
    UnresolvedDuplicate,
    Settlement(AnchoredRecordStepSettlementFailure),
    CompletedWithoutRecord,
}
impl OrderedJournalPublicationTerminal {
    pub fn source(&self) -> OrderedJournalPublicationSource {
        self.source
    }
    pub fn mutation_identity(&self) -> Option<PhysicalMutationIdentity> {
        self.mutation
    }
    pub fn physical_outcome(&self) -> Option<&PhysicalMutationOutcome> {
        self.outcome.as_ref()
    }
    pub fn published(&self) -> Option<OrderedJournalPublished> {
        self.published
    }
    pub fn failure(&self) -> Option<OrderedJournalTerminalFailure> {
        self.failure
    }
}
/// Sealed actual journal completion. Descriptive entries cannot construct it.
#[derive(Debug)]
pub struct OrderedJournalCommitted {
    pub(super) terminal: Arc<OrderedJournalPublicationTerminal>,
}
impl OrderedJournalCommitted {
    pub fn source(&self) -> OrderedJournalPublicationSource {
        self.terminal.source
    }
    pub fn entry(&self) -> OrderedJournalEntry {
        self.published().entry()
    }
    pub fn published(&self) -> OrderedJournalPublished {
        self.terminal
            .published
            .expect("only actual selected journal entries issue committed custody")
    }
    pub fn terminal(&self) -> &OrderedJournalPublicationTerminal {
        &self.terminal
    }
}
#[derive(Debug, Clone)]
pub struct OrderedJournalPublicationObservation {
    pub(super) source: OrderedJournalPublicationSource,
    pub(super) mutation: Option<PhysicalMutationIdentity>,
    pub(super) stopped: bool,
    pub(super) terminal: Option<Arc<OrderedJournalPublicationTerminal>>,
}
impl OrderedJournalPublicationObservation {
    pub fn source(&self) -> OrderedJournalPublicationSource {
        self.source
    }
    pub fn mutation_identity(&self) -> Option<PhysicalMutationIdentity> {
        self.mutation
    }
    pub fn stopped(&self) -> bool {
        self.stopped
    }
    pub fn terminal(&self) -> Option<&Arc<OrderedJournalPublicationTerminal>> {
        self.terminal.as_ref()
    }
}
#[derive(Debug)]
pub enum OrderedJournalOperationFailure {
    CompletionReadmissionMismatch,
    UnresolvedDuplicate,
    Publication(OrderedJournalPublicationFailure),
    Busy,
    ForeignReceiver,
    OwnerReleased,
    Stopped,
    OriginalRequestStopped(worth_execution::MapKernelStop),
    Panicked,
    Settlement(AnchoredRecordStepSettlementFailure),
}
impl From<OrderedJournalPublicationFailure> for OrderedJournalOperationFailure {
    fn from(failure: OrderedJournalPublicationFailure) -> Self {
        Self::Publication(failure)
    }
}
#[must_use]
#[derive(Debug)]
pub struct OrderedJournalPublicationFailureCustody {
    pub(super) stopped: super::recovery::OrderedJournalStoppedPublication,
    pub(super) cause: OrderedJournalOperationFailure,
}
impl OrderedJournalPublicationFailureCustody {
    pub fn cause(&self) -> &OrderedJournalOperationFailure {
        &self.cause
    }
    pub fn observe(&self) -> OrderedJournalPublicationObservation {
        self.stopped.observe()
    }
    pub fn into_stopped(self) -> super::recovery::OrderedJournalStoppedPublication {
        self.stopped
    }
    pub(super) fn into_legacy(self) -> OrderedJournalPublicationFailure {
        match self {
            Self {
                cause: OrderedJournalOperationFailure::Publication(cause),
                stopped: _,
            } => cause,
            other => OrderedJournalPublicationFailure::OwnedPublication(Box::new(other)),
        }
    }
}

impl std::fmt::Debug for OrderedJournalPublicationTerminal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fate = self.outcome.as_ref().map(|o| match o {
            PhysicalMutationOutcome::Completed(_) => "Completed",
            PhysicalMutationOutcome::ProvenNoEffect(_) => "ProvenNoEffect",
            PhysicalMutationOutcome::Indeterminate(_) => "Indeterminate",
        });
        f.debug_struct("OrderedJournalPublicationTerminal")
            .field("source", &self.source)
            .field("mutation", &self.mutation)
            .field("physical_fate", &fate)
            .field("published", &self.published)
            .field("failure", &self.failure)
            .finish()
    }
}
