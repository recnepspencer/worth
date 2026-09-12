use std::sync::Arc;

use crate::branch::{AdmittedRelationalBranchBasis, RelationalBranchBasisDescriptor};
use crate::history::data::{CanonicalCommitEnvelope, PositionedCanonicalCommit};
use crate::history::RelationalCommitIdentity;

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
        crate::branch::RelationalBranchPublicationAuthorityMarker,
        PerformedRelationalCommitData,
    >,
    capability: PerformedSettlementCapability,
    late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
}

/// Borrowed view of the runtime-owned pending settlement record.
///
/// Its `Drop` is the whole reason this is a separate value: abandoning the
/// witness must be observable without letting a partial move of the proof
/// suppress that accounting.
struct PerformedSettlementCapability {
    record: Arc<crate::runtime::PendingRelationalPublicationSettlement>,
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
        record: Arc<crate::runtime::PendingRelationalPublicationSettlement>,
        late_interruption: Option<crate::runtime::RelationalInterruptionEvent>,
    ) -> Self {
        Self {
            performed: worth_proof::Performed::record(
                &crate::branch::issue_relational_branch_publication_authority(),
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
        Arc<crate::runtime::PendingRelationalPublicationSettlement>,
    ) {
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

    pub const fn late_interruption(&self) -> Option<crate::runtime::RelationalInterruptionEvent> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalPublicationDenial {
    StaleInvariantGeneration {
        expected_generation: u64,
        actual_generation: u64,
    },
    OwnerUnavailable {
        runtime_instance_id: u64,
    },
    ForeignRuntime {
        expected_runtime_instance_id: u64,
        actual_runtime_instance_id: u64,
    },
    OwnerMismatch,
    BranchUnavailable,
    Archived,
    Deleting,
}

/// A bounded resource turned this publication attempt away without moving
/// anything.
///
/// Every variant is a typed no-movement answer rather than an error: the branch
/// reference is exactly where it started, no publication route survives, and
/// every resource the attempt took is given back before it returns. What
/// differs between variants is which bound was met, and therefore what the
/// caller must do next.
///
/// The variants do not share one entry surface. A
/// `RelationalPublicationOutcome::Deferred` from
/// `RelationalPublicationPort::compare_and_publish` can only ever be
/// `PatchPositionReservationContended`, `RetentionBackpressure`, or
/// `CandidateLifetimeExpired`. `CandidateCapacityExhausted` and
/// `PublishedSnapshotCapacityExhausted` are raised only while a candidate is
/// being prepared, so they reach a caller as
/// `TransactionCommitError::PublicationDeferred` and never as a publication
/// outcome. All five can surface from `commit_branch_transaction`, which
/// prepares and publishes in one call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationDeferred {
    /// Another publisher holds the runtime's single patch-position reservation.
    ///
    /// The reservation is one runtime-wide, nonblocking slot, held for the
    /// duration of one cutover. Contention is therefore global: a deferred
    /// publisher is turned away by that slot, never by a wait on its own
    /// branch. It is not capacity exhaustion either; the reservation counters
    /// record a deferral and no overflow.
    ///
    /// The slot is tested after the exact branch-cell comparison and before
    /// the cutover body, so this deferral is only ever reached with that
    /// comparison already matched: the branch was exactly where this candidate
    /// expected it at that instant, and only the reference movement did not
    /// happen. That match is an observation, not a durable admission grant. The
    /// candidate is consumed and its candidate slot returned, and the
    /// pre-effect pending-settlement reservation is installed and then released
    /// on the way out, leaving no record behind.
    ///
    /// To retry, begin a fresh transaction and prepare a new candidate; this
    /// one is spent, and is neither reusable nor a rebase authority. The same
    /// still-admitted expected basis may be reused, since a contended
    /// mechanical reservation is no reason to pay for a new observation.
    /// Ordinary comparison still returns stale if the branch moved meanwhile.
    PatchPositionReservationContended,
    /// A retention obligation could not be acquired because the owner's live
    /// root capacity or its retired branch-root capacity is full.
    ///
    /// These are owner capacities rather than `PublicationConfig` policy, so no
    /// configuration field raises or relaxes them. Three points report it:
    /// acquiring the candidate's retention during preparation, admitting the
    /// next basis during publication preflight, and reserving the head
    /// retirement. All three run before the pending-settlement reservation is
    /// installed, so nothing was reserved and no route survives.
    ///
    /// Release a live obligation before retrying: drop an admitted basis, or
    /// `release_component_basis` for one that was retained externally, then
    /// prepare and publish again.
    RetentionBackpressure,
    /// The prepared candidate outlived `PublicationConfig`'s
    /// `candidate_max_lifetime_millis`, echoed here as
    /// `maximum_lifetime_millis`.
    ///
    /// Expiry is checked before the candidate is consumed and again after the
    /// wait for branch coordination, so a candidate can expire while queued
    /// behind another publisher on its own branch. A candidate whose registry
    /// entry was already reaped reports the same variant.
    ///
    /// Either way the candidate is spent, and its candidate slot and retention
    /// are returned; a candidate that is never consumed discards itself when it
    /// drops, so no path keeps the slot.
    ///
    /// Prepare again. An expired candidate cannot be renewed.
    CandidateLifetimeExpired { maximum_lifetime_millis: u64 },
    /// The prepared-candidate population is full at `PublicationConfig`'s
    /// `max_prepared_candidates`, echoed here as `maximum_candidates`.
    ///
    /// It is raised while registering a new candidate, so it is reachable only
    /// from preparation and never from `compare_and_publish`. Nothing was
    /// prepared: no candidate, no settlement reservation, no movement.
    ///
    /// Free a slot before preparing again by publishing a candidate, by
    /// `discard_prepared_candidate`, or by `reap_expired_prepared_candidates`.
    CandidateCapacityExhausted { maximum_candidates: usize },
    /// The published snapshot handle population is full at
    /// `PublicationConfig`'s `max_published_snapshot_handles`, echoed here as
    /// `maximum_handles`.
    ///
    /// One bound covers three populations at once: prepared candidates holding
    /// a reserved slot, performed commits that are not yet settled, and
    /// published snapshots that have not been released. That is why a caller
    /// who never releases handles can meet this bound with no transaction in
    /// flight, and why settlement admission needs no second capacity check.
    ///
    /// The slot is reserved during preparation, before any candidate or
    /// settlement reservation exists, so this deferral never reaches settlement
    /// and moves nothing.
    ///
    /// Release a published snapshot before preparing again, by calling
    /// `release_snapshot` on `snapshots()` with the handle named by the commit
    /// result. A settlement record that is never claimed also closes its handle
    /// when it drops, but that is not on a schedule the caller controls.
    PublishedSnapshotCapacityExhausted { maximum_handles: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalPublicationFailureKind {
    SnapshotIdentityExhausted,
    CandidateIdentityExhausted,
    PreparedRootBudgetExhausted {
        maximum_bytes: u64,
        required_bytes: u64,
    },
    PreparedRootMismatch,
    PreparedBasisDescriptor(crate::branch::RelationalBranchBasisDenial),
    NextBasisAdmission(crate::branch::RelationalBranchBasisDenial),
    /// The selected branch root was already gone when the critical section
    /// looked for it, so publication stopped before any comparison or any
    /// reference movement.
    SelectedRootUnavailable,
    BranchObservation(crate::branch::RelationalBranchBasisDenial),
    PatchPositionCapacityExhausted,
    RetentionIdentityExhausted,
    RetentionOwner,
    /// A pending settlement record already exists for this candidate's
    /// owner-issued commit identity, so the pre-effect reservation would have
    /// aliased another attempt's recovery state.
    PendingSettlementIdentityConflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelationalPublicationFailure {
    kind: RelationalPublicationFailureKind,
    detail: String,
}

impl RelationalPublicationFailure {
    pub(crate) fn new(kind: RelationalPublicationFailureKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
        }
    }

    pub fn kind(&self) -> &RelationalPublicationFailureKind {
        &self.kind
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[derive(Debug)]
#[must_use = "publication outcomes carry performed work or a typed terminal posture"]
pub enum RelationalPublicationOutcome {
    Performed(PerformedRelationalCommit),
    Stale(StaleRelationalBranchObservation),
    Denied(RelationalPublicationDenial),
    Interrupted(crate::runtime::RelationalInterruptionEvent),
    Deferred(RelationalPublicationDeferred),
    Failed(RelationalPublicationFailure),
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

    pub(crate) fn interrupted(interruption: crate::runtime::RelationalInterruptionEvent) -> Self {
        Self::Interrupted(interruption)
    }

    pub(crate) fn deferred(deferred: RelationalPublicationDeferred) -> Self {
        Self::Deferred(deferred)
    }

    pub(crate) fn failed(failure: RelationalPublicationFailure) -> Self {
        Self::Failed(failure)
    }
}
