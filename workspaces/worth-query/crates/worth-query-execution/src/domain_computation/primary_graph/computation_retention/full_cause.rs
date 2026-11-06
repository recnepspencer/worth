//! Why a run of a partitioned computation recomputed every partition.
//!
//! The cause is data: the partitioned computation's observer of a full
//! retained reduction stays with that computation
//! (`application_contribution::partitioned_computation::incremental::retained`).

/// Why a run recomputed every partition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPartitionedComputationFullCause {
    /// No output record has been published at this address.
    FirstRun,
    /// Checkpoint restoration retained no computation state.
    Restored,
    /// Republication retained facts but no computation state.
    Republished,
    /// The decision ran no partitioned computation.
    NotProduced,
    /// The run's charge or retained byte sum could not be measured.
    Unmeasured,
    /// A successor took this record's computation reservation.
    Moved,
    /// The lineage ledger refused the computation reservation.
    Evicted,
    /// The input value's canonical encoding differs.
    InputChanged,
    /// The installed owner differs, even when its types match.
    OtherInstallation,
    /// The installed producer edition differs.
    OtherEdition,
    /// The observation bound exceeds the request's work policy.
    ObservationOverBudget,
    /// Complete structural reads require fresh issuance under this admission;
    /// their live carried-value handoff is not yet qualified for reuse.
    StructuralWalkRead,
    /// Aggregate decision inputs require fresh Native identity and membership reads.
    AggregateRead,
    /// A prior was already consumed by an earlier invocation in this handler.
    NoPriorHanded,
    /// This reader belongs to an operation outside a producer invocation.
    NoProducerPrior,
    /// Policy suppressed the completed computation's retention.
    RetentionPolicy,
    /// The handler ran several computations and has no single attribution.
    SeveralComputations,
    /// A collision restart computed without a reusable retention basis.
    CollisionSuppressed,
    /// The computation stopped before leaving completed state.
    Stopped,
    /// Execution policy chose not to retain this computation.
    Unretained,
    /// Incremental routing met a compact partition identity collision.
    IdentityCollision,
}
