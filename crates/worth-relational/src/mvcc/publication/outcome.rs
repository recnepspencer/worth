use std::sync::Arc;
use worth_relational_branch::internal_port::history::commit::identity::RelationalCommitIdentityOwnerPort as _;

use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::basis::RelationalBranchBasisDescriptor;
use crate::history::commit::RelationalCommitIdentity;
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};

pub use failure::{RelationalPublicationFailure, RelationalPublicationFailureKind};
pub(crate) use worth_relational_contracts::internal_port::mvcc::publication::outcome::failure;

pub struct PublishRelationalCommit;

impl worth_proof::ActionMarker for PublishRelationalCommit {}

/// Posture after the owner reports performed publication. The branch root and
/// exact basis are current; history, patch, and replay inputs resolve from
/// that root immediately. Optional accelerators and diagnostic projections
/// may be refreshed later and never decide publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationProjectionPosture {
    CanonicalRootCurrentOptionalProjectionsDeferred,
}

/// Durability posture of the independently borrowable Phase 9 movement port.
/// The port proves in-process owner movement only; the ordinary commit facade
/// reports success only after its durability barrier acknowledges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationDurabilityPosture {
    OwnerAcknowledgementDeferred,
}

#[derive(Debug)]
struct PerformedRelationalCommitData {
    positioned_commit: Arc<PositionedCanonicalCommit>,
    next_basis: AdmittedRelationalBranchBasis,
}

/// Witness that one prepared candidate crossed its branch linearization point.
///
/// This value is deliberately non-cloneable, but it is not the owner of the
/// remaining settlement work. The runtime installed that work in its pending
/// settlement registry before the movement this value reports, so dropping the
/// witness records capability abandonment and nothing else: the obligation,
/// the route's settled marking, and repair availability all stay with the
/// runtime.
#[must_use = "performed publication must be settled by its owning runtime"]
pub struct PerformedRelationalCommit {
    performed: worth_proof::Performed<
        PublishRelationalCommit,
        crate::branch::authority::RelationalBranchPublicationAuthorityMarker,
        PerformedRelationalCommitData,
    >,
    capability: PerformedSettlementCapability,
    late_interruption:
        Option<crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent>,
}

/// Borrowed view of the runtime-owned pending settlement record.
///
/// Its `Drop` is the whole reason this is a separate value: abandoning the
/// witness must be observable without letting a partial move of the proof
/// suppress that accounting.
struct PerformedSettlementCapability {
    record: Arc<crate::runtime::state::runtime_state::publication_settlement_registry::record::PendingRelationalPublicationSettlement>,
    consumed: bool,
}

impl Drop for PerformedSettlementCapability {
    fn drop(&mut self) {
        if !self.consumed {
            self.record.record_capability_abandonment();
        }
    }
}

impl std::fmt::Debug for PerformedRelationalCommit {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PerformedRelationalCommit")
            .field(
                "commit_id",
                &self
                    .performed
                    .outcome()
                    .positioned_commit
                    .envelope()
                    .commit
                    .commit_id,
            )
            .field(
                "branch_id",
                self.performed.outcome().next_basis.identity().branch_id(),
            )
            .finish_non_exhaustive()
    }
}

impl PerformedRelationalCommit {
    pub(crate) fn record(
        positioned_commit: Arc<PositionedCanonicalCommit>,
        next_basis: AdmittedRelationalBranchBasis,
        record: Arc<crate::runtime::state::runtime_state::publication_settlement_registry::record::PendingRelationalPublicationSettlement>,
        late_interruption: Option<
            crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent,
        >,
    ) -> Self {
        Self {
            performed: worth_proof::Performed::record(
                &crate::branch::authority::issue_relational_branch_publication_authority(),
                PerformedRelationalCommitData {
                    positioned_commit,
                    next_basis,
                },
            ),
            capability: PerformedSettlementCapability {
                record,
                consumed: false,
            },
            late_interruption,
        }
    }

    pub fn canonical_commit(&self) -> &CanonicalCommitEnvelope {
        self.performed.outcome().positioned_commit.envelope()
    }

    /// Derive the immutable commit occurrence from the owner-issued performed
    /// result. Callers cannot manufacture this identity or select it
    /// independently from the performed canonical commit.
    pub fn commit_identity(&self) -> RelationalCommitIdentity {
        let envelope = self.canonical_commit();
        RelationalCommitIdentity::new(
            envelope.commit.commit_id,
            envelope.commit.version_id,
            envelope.branch_context.clone(),
        )
    }

    pub fn patch_position(&self) -> crate::publication::patch::data::PatchStreamPosition {
        self.performed.outcome().positioned_commit.position()
    }

    pub fn next_basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.performed.outcome().next_basis
    }

    /// Surrender the witness to its owning runtime. The registry record is the
    /// authority for what remains, so this hands back only the exact route and
    /// the record that already holds the work.
    pub(crate) fn into_settlement_parts(
        mut self,
    ) -> (
        Arc<PositionedCanonicalCommit>,
        Arc<crate::runtime::state::runtime_state::publication_settlement_registry::record::PendingRelationalPublicationSettlement>,
    ){
        self.capability.consumed = true;
        let record = Arc::clone(&self.capability.record);
        let outcome = self.performed.into_outcome();
        (outcome.positioned_commit, record)
    }

    pub const fn projection_posture(&self) -> RelationalPublicationProjectionPosture {
        RelationalPublicationProjectionPosture::CanonicalRootCurrentOptionalProjectionsDeferred
    }

    pub const fn durability_posture(&self) -> RelationalPublicationDurabilityPosture {
        RelationalPublicationDurabilityPosture::OwnerAcknowledgementDeferred
    }

    pub const fn late_interruption(
        &self,
    ) -> Option<crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent>
    {
        self.late_interruption
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaleRelationalBranchObservation {
    expected: RelationalBranchBasisDescriptor,
    observed: RelationalBranchBasisDescriptor,
}

impl StaleRelationalBranchObservation {
    pub(crate) fn new(
        expected: RelationalBranchBasisDescriptor,
        observed: RelationalBranchBasisDescriptor,
    ) -> Self {
        Self { expected, observed }
    }

    pub fn expected(&self) -> &RelationalBranchBasisDescriptor {
        &self.expected
    }

    pub fn observed(&self) -> &RelationalBranchBasisDescriptor {
        &self.observed
    }
}

pub use refusals::{RelationalPublicationDeferred, RelationalPublicationDenial};
pub(crate) use worth_relational_contracts::internal_port::mvcc::publication::outcome::refusals;

#[derive(Debug)]
#[must_use = "publication outcomes carry performed work or a typed terminal posture"]
pub enum RelationalPublicationOutcome {
    RequestStopped(worth_execution::WorkCeilingDenial),
    Performed(PerformedRelationalCommit),
    Stale(StaleRelationalBranchObservation),
    Denied(RelationalPublicationDenial),
    Interrupted(crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent),
    Deferred(RelationalPublicationDeferred),
    Failed(RelationalPublicationFailure),
    /// The medium could not say whether the commit became durable. It was not
    /// installed; reopening the runtime resolves its fate.
    DurabilityUnresolved(crate::durability::medium::RelationalDurabilityUnresolved),
}

impl RelationalPublicationOutcome {
    pub(crate) fn performed(performed: PerformedRelationalCommit) -> Self {
        Self::Performed(performed)
    }

    pub(crate) fn stale(stale: StaleRelationalBranchObservation) -> Self {
        Self::Stale(stale)
    }

    pub(crate) fn denied(denial: RelationalPublicationDenial) -> Self {
        Self::Denied(denial)
    }

    pub(crate) fn interrupted(
        interruption: crate::runtime::operation_control::interruption_event::RelationalInterruptionEvent,
    ) -> Self {
        Self::Interrupted(interruption)
    }

    pub(crate) fn deferred(deferred: RelationalPublicationDeferred) -> Self {
        Self::Deferred(deferred)
    }

    pub(crate) fn failed(failure: RelationalPublicationFailure) -> Self {
        Self::Failed(failure)
    }
}
