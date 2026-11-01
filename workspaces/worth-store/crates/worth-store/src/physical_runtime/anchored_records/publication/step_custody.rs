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
    ExecutionPanicked,
    TerminalSettlementPanicked,
    TerminalDelivery(AnchoredRecordStepDeliveryFailure),
    TerminalDeliveryPanicked,
    ProtectionReleasePanicked,
    WorkerUnavailable,
    UnstartedCancellationRefused(crate::physical_runtime::PhysicalPreSealCancellationDenial),
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
        self.step.observe_existing_terminal();
        self.step.observed_terminal()
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
    settled: bool,
    fact: Option<PhysicalMutationTerminalFact>,
    unstarted_refusal: Option<crate::physical_runtime::PhysicalPreSealCancellationDenial>,
    existing: Option<Arc<crate::physical_runtime::durability::ExistingPhysicalMutationObservation>>,
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
                settled: false,
                fact: None,
                unstarted_refusal: None,
                existing: None,
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
        let mut settlement_failure = failure;
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
            settlement_failure = settlement_failure.or(delivery_failure);
            // A destination panic is caught before releasing its owned guard,
            // so a release panic cannot become a double unwind of the callback.
            let released = catch_unwind(AssertUnwindSafe(|| drop(retention)));
            if released.is_err() {
                settlement_failure = settlement_failure.or(Some(
                    AnchoredRecordStepSettlementFailure::ProtectionReleasePanicked,
                ));
            }
        }
        let mut state = self.state();
        if settlement_failure != failure {
            state.terminal = Some(Arc::new(AnchoredRecordStepTerminal {
                mutation: self.mutation,
                outcome: fact.outcome(),
                failure: settlement_failure,
            }));
        }
        // Saved physical fact is not settled delivery. Public observation only
        // becomes terminal after both destination and protection have settled.
        state.settled = true;
    }
    pub(super) fn observed_terminal(&self) -> Option<Arc<AnchoredRecordStepTerminal>> {
        let state = self.state();
        if state.settled {
            state.terminal.clone()
        } else {
            None
        }
    }
    pub(super) fn refuse_unstarted(
        &self,
        prepared: crate::physical_runtime::PreparedPhysicalMutation,
        cause: crate::physical_runtime::PhysicalPreSealCancellationDenial,
    ) {
        let existing = prepared.observe_existing_execution().map(Arc::new);
        {
            let mut state = self.state();
            state.unstarted_refusal = Some(cause);
            state.existing = existing;
        }
        // No execution uses this duplicate input. Retire its bytes and view
        // outside every owner lock; the original attempt owns its own input.
        drop(prepared);
    }
    pub(super) fn unstarted_refusal_without_attempt(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalPreSealCancellationDenial> {
        let state = self.state();
        state.unstarted_refusal.filter(|_| state.existing.is_none())
    }
    fn observe_existing_terminal(&self) {
        let (existing, refusal) = {
            let state = self.state();
            (state.existing.clone(), state.unstarted_refusal)
        };
        if let Some(existing) = existing
            && let Some(fact) = existing.terminal()
        {
            self.finish(
                &fact,
                refusal.map(AnchoredRecordStepSettlementFailure::UnstartedCancellationRefused),
            );
            let retired = self.state().existing.take();
            drop(retired);
        }
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
        self.observe_existing_terminal();
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
        self.observed_terminal()
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
