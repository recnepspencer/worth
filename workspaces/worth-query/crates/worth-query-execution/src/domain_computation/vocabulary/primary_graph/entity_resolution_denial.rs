//! Moved whole from `domain_computation::primary_graph::vocabulary::entity_resolution_denial`: the items below and every impl they own.

/// Why an entity could not be resolved from its identity field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEntityResolutionDenialKind {
    Stopped(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Refused(WorthQueryEntityResolutionRefusalKind),
}

/// An ordinary resolution refusal from installed identity fields and exact truth.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEntityResolutionRefusalKind {
    /// The supplied phase belongs to another installed runtime.
    ForeignPhase,
    /// The primary graph is not installed.
    PrimaryGraphNotInstalled,
    /// The identity field is not installed.
    FieldNotInstalled,
    /// The identity value could not be encoded.
    ValueEncodingRejected,
    /// The identity index is unavailable.
    EqualityIndexUnavailable,
    /// No entity has this identity.
    UnknownEntity,
    /// More than one entity has this identity.
    AmbiguousEntity,
    /// The identity index disagrees with stored records.
    CorruptIdentityIndex,
    /// A complete selection needs a nonzero finite candidate limit.
    InvalidCandidateLimit,
    /// The complete equality result exceeds the caller's candidate limit.
    CandidateLimitExceeded { maximum: usize },
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    /// The runtime ran out of snapshot identities.
    SnapshotIdentityExhausted,
    /// No capacity remains to retain a basis.
    RetentionCapacityExhausted,
    /// The runtime ran out of basis-retention identities.
    RetentionIdentityExhausted,
    /// The identity or snapshot belongs to a different runtime or binding.
    ForeignResolutionTruth,
}
