//! Lifetime retention and actual terminal delivery for one physical append.
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::super::AnchoredRecordName;
use crate::physical_runtime::{PhysicalMutationAttempt, PhysicalMutationTerminalFact};
use crate::physical_runtime::{PhysicalMutationIdentity, PhysicalMutationOutcome, RuntimeIdentity};

/// Correlation only: neither publication nor Native work authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnchoredRecordPublicationIdentity {
    pub(super) runtime: RuntimeIdentity,
    pub(super) name: AnchoredRecordName,
    pub(super) generation: u64,
    pub(super) content: [u8; 32],
}
impl AnchoredRecordPublicationIdentity {
    pub fn runtime_identity(self) -> RuntimeIdentity {
        self.runtime
    }
    pub fn name(self) -> AnchoredRecordName {
        self.name
    }
    pub fn generation(self) -> u64 {
        self.generation
    }
    pub fn content_sha256(self) -> [u8; 32] {
        self.content
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchoredRecordStepKind {
    Members { batch_index: usize },
    Anchor,
}

/// An owner's actual selection lifetime, not physical work authority.
pub trait AnchoredRecordSelectionProtection: Send + Sync + 'static {}

pub trait AnchoredRecordStepTerminalDestination: Send + Sync + 'static {
    fn record_terminal(
        &self,
        terminal: &AnchoredRecordStepTerminal,
    ) -> Result<(), AnchoredRecordStepDeliveryFailure>;
}

pub trait AnchoredRecordStepCustodyPort: Send + Sync {
    fn retain_step(
        &self,
        registration: AnchoredRecordStepRegistration,
    ) -> Result<AnchoredRecordStepRetention, AnchoredRecordStepCustodyRefusal>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchoredRecordStepCustodyRefusal {
    SelectionUnavailable,
    OwnerUnavailable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchoredRecordStepDeliveryFailure {
    OwnerUnavailable,
    ReceiptRefused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnchoredRecordStepSettlementFailure {
    TerminalPersistence,
    TerminalDelivery(AnchoredRecordStepDeliveryFailure),
    TerminalDeliveryPanicked,
    WorkerUnavailable,
}

pub struct AnchoredRecordStepRetention {
    _protection: Box<dyn AnchoredRecordSelectionProtection>,
    destination: Arc<dyn AnchoredRecordStepTerminalDestination>,
}
impl AnchoredRecordStepRetention {
    pub fn new(
        protection: Box<dyn AnchoredRecordSelectionProtection>,
        destination: Arc<dyn AnchoredRecordStepTerminalDestination>,
    ) -> Self {
        Self {
            _protection: protection,
            destination,
        }
    }
}

/// Strong, read-only custody installed in the caller's retained pending slot.
/// It cannot submit a step, replace controls, or mint a terminal outcome.
pub struct AnchoredRecordStepRegistration {
    pub(super) step: Arc<AcceptedStep>,
}
impl AnchoredRecordStepRegistration {
    pub fn publication_identity(&self) -> AnchoredRecordPublicationIdentity {
        self.step.publication
    }
    pub fn kind(&self) -> AnchoredRecordStepKind {
        self.step.kind
    }
    pub fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.step.mutation
    }
    pub fn terminal(&self) -> Option<Arc<AnchoredRecordStepTerminal>> {
        self.step.state().terminal.clone()
    }
    pub fn completed_member_batches(&self) -> usize {
        self.step.completed_batches
    }
}

/// Inert owner-issued physical evidence; construction is private to Store.
pub struct AnchoredRecordStepTerminal {
    mutation: PhysicalMutationIdentity,
    outcome: PhysicalMutationOutcome,
    failure: Option<AnchoredRecordStepSettlementFailure>,
}
impl AnchoredRecordStepTerminal {
    pub fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }
    pub fn outcome(&self) -> &PhysicalMutationOutcome {
        &self.outcome
    }
    pub fn settlement_failure(&self) -> Option<AnchoredRecordStepSettlementFailure> {
        self.failure
    }
}

pub(in crate::physical_runtime) struct AcceptedStep {
    publication: AnchoredRecordPublicationIdentity,
    kind: AnchoredRecordStepKind,
    mutation: PhysicalMutationIdentity,
    completed_batches: usize,
    state: Mutex<AcceptedStepState>,
    publication_owner: Weak<super::operation::PublicationShared>,
}

pub(in crate::physical_runtime) struct ManagedStepCustody {
    step: Arc<AcceptedStep>,
    _publication: Arc<super::operation::PublicationShared>,
}
impl ManagedStepCustody {
    pub(in crate::physical_runtime) fn step(&self) -> &Arc<AcceptedStep> {
        &self.step
    }
}
struct AcceptedStepState {
    attempt: Weak<PhysicalMutationAttempt>,
    retention: Option<AnchoredRecordStepRetention>,
    terminal: Option<Arc<AnchoredRecordStepTerminal>>,
    fact: Option<PhysicalMutationTerminalFact>,
    unstarted: Option<crate::physical_runtime::PreparedPhysicalMutation>,
    owner: Weak<Mutex<super::super::AnchoredRecordState>>,
}
impl AcceptedStep {
    pub(super) fn new(
        publication: AnchoredRecordPublicationIdentity,
        kind: AnchoredRecordStepKind,
        mutation: PhysicalMutationIdentity,
        completed_batches: usize,
        owner: Weak<Mutex<super::super::AnchoredRecordState>>,
        publication_owner: Weak<super::operation::PublicationShared>,
    ) -> Arc<Self> {
        Arc::new(Self {
            publication,
            kind,
            mutation,
            completed_batches,
            publication_owner,
            state: Mutex::new(AcceptedStepState {
                attempt: Weak::new(),
                retention: None,
                terminal: None,
                fact: None,
                unstarted: None,
                owner,
            }),
        })
    }
    pub(in crate::physical_runtime) fn managed_custody(self: &Arc<Self>) -> ManagedStepCustody {
        ManagedStepCustody {
            step: self.clone(),
            _publication: self
                .publication_owner
                .upgrade()
                .expect("starting endpoint retains the exact physical publication"),
        }
    }
    pub(super) fn retain(
        self: &Arc<Self>,
        port: &dyn AnchoredRecordStepCustodyPort,
    ) -> Result<(), AnchoredRecordStepCustodyRefusal> {
        let retention = port.retain_step(AnchoredRecordStepRegistration { step: self.clone() })?;
        self.state().retention = Some(retention);
        Ok(())
    }
    pub(in crate::physical_runtime) fn bind(&self, attempt: &Arc<PhysicalMutationAttempt>) {
        self.state().attempt = Arc::downgrade(attempt);
    }
    pub(in crate::physical_runtime) fn preserve_fact(&self, fact: &PhysicalMutationTerminalFact) {
        self.state().fact = Some(fact.clone());
    }
    pub(in crate::physical_runtime) fn finish(
        &self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<AnchoredRecordStepSettlementFailure>,
    ) {
        let (terminal, retention) = {
            let mut state = self.state();
            if state.terminal.is_some() {
                return;
            }
            let terminal = Arc::new(AnchoredRecordStepTerminal {
                mutation: self.mutation,
                outcome: fact.outcome(),
                failure,
            });
            // Preserve the exact physical fact before calling a foreign owner.
            state.terminal = Some(terminal.clone());
            (terminal, state.retention.take())
        };
        let uncertain_anchor = self.kind == AnchoredRecordStepKind::Anchor
            && (failure.is_some()
                || matches!(
                    terminal.outcome(),
                    PhysicalMutationOutcome::Indeterminate(_)
                )
                || matches!(terminal.outcome(), PhysicalMutationOutcome::Completed(done) if done.persisted_records().len() != 1));
        let owner = self.state().owner.upgrade();
        if uncertain_anchor && let Some(owner) = owner {
            owner
                .lock()
                .unwrap_or_else(|p| p.into_inner())
                .unresolved
                .insert(self.publication.name.anchor());
        }
        if let Some(retention) = retention {
            let delivered = catch_unwind(AssertUnwindSafe(|| {
                retention.destination.record_terminal(&terminal)
            }));
            let delivery_failure = match delivered {
                Ok(Ok(())) => None,
                Ok(Err(cause)) => {
                    Some(AnchoredRecordStepSettlementFailure::TerminalDelivery(cause))
                }
                Err(_) => Some(AnchoredRecordStepSettlementFailure::TerminalDeliveryPanicked),
            };
            if let Some(cause) = delivery_failure {
                self.state().terminal = Some(Arc::new(AnchoredRecordStepTerminal {
                    mutation: self.mutation,
                    outcome: fact.outcome(),
                    failure: failure.or(Some(cause)),
                }));
            }
        }
    }
    pub(super) fn hold_unstarted(
        &self,
        prepared: crate::physical_runtime::PreparedPhysicalMutation,
    ) {
        self.state().unstarted = Some(prepared);
    }
    pub(super) fn has_unstarted(&self) -> bool {
        self.state().unstarted.is_some()
    }
    pub(super) fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.mutation
    }
    pub(super) fn wait(&self) {
        let attempt = self.state().attempt.upgrade();
        if let Some(attempt) = attempt {
            crate::physical_runtime::PhysicalMutationHandle::new(&attempt).wait();
        }
    }
    pub(super) fn terminal(&self) -> Option<Arc<AnchoredRecordStepTerminal>> {
        let attempt = self.state().attempt.upgrade();
        if let Some(attempt) = attempt.as_ref()
            && let Some(fact) = attempt.visible_terminal_fact()
        {
            self.finish(&fact, None);
        }
        let lost = {
            let state = self.state();
            if state.terminal.is_none() && state.attempt.upgrade().is_none() {
                state.fact.clone()
            } else {
                None
            }
        };
        if let Some(fact) = lost {
            self.finish(
                &fact,
                Some(AnchoredRecordStepSettlementFailure::WorkerUnavailable),
            );
        }
        self.state().terminal.clone()
    }
    pub(super) fn cancel(&self) {
        let attempt = self.state().attempt.upgrade();
        if let Some(attempt) = attempt {
            crate::physical_runtime::PhysicalMutationHandle::new(&attempt).request_cancellation();
        }
    }
    fn state(&self) -> MutexGuard<'_, AcceptedStepState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
