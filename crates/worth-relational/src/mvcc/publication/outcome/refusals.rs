//! Typed no-movement outcomes reported by the publication owner.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalPublicationDenial {
    HomeProgramTransition(crate::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial),
    /// The selected Home root has no matching admitted execution inventory.
    HomeProgramContractsRequired,
    HomeProgramIndexDefinitionsRequired,
    StaleIndexDefinitions,
    IndexDefinitionPublicationContended,
    CanonicalArtifactEncoding(
        crate::durability::canonical_commit_artifact::encode_denial::RelationalCanonicalCommitArtifactEncodeDenial,
    ),
    ScopedCutoverDurabilityUnsupported,
    CutoverDeadlineElapsed,
    CurrentReadDependency(crate::mvcc::publication::read_dependency_denial::RelationalPublicationReadDependencyDenial),
    RecoveredCodecTransitionRequired,
    /// The coherent receiver pair changed, including same-generation replay replacement.
    StaleConfigurationEpoch,
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
    Archived,
    Deleting,
    /// The durable medium refused the commit before any durable effect; it
    /// was never installed.
    DurablePublicationRefused(crate::durability::medium::publication_refusal::RelationalDurablePublicationRefusal),
    RetainedEntitySelectionDurabilityUnsupported,
    OwnedGraphSelectionDurabilityUnsupported {
        model: crate::identity::data::EntityId,
    },
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
/// Candidate and published-snapshot capacity are checked during preparation;
/// publication and required companion stops are checked by the publication port.
/// `commit_branch_transaction` can expose either phase's typed deferral.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalPublicationDeferred {
    CompanionRegistrationPending,
    CompanionRebindRequired,
    CompanionPreflight(crate::mvcc::publication::companion::preflight_stop::CompanionPreflightStop),
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
    /// The prepared-candidate population is full at `PublicationConfig`'s
    /// `max_prepared_candidates`, echoed here as `maximum_candidates`.
    ///
    /// It is raised while registering a new candidate, so it is reachable only
    /// from preparation and never from `compare_and_publish`. Nothing was
    /// prepared: no candidate, no settlement reservation, no movement.
    ///
    /// Free a slot before preparing again by publishing a candidate, by
    /// `discard_prepared_candidate`, or by dropping it.
    CandidateCapacityExhausted {
        maximum_candidates: usize,
    },
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
    PublishedSnapshotCapacityExhausted {
        maximum_handles: usize,
    },
}
