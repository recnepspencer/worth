//! Typed derivation failures for external-effect identity and recovery.

/// Internal digest-preparation failure for aftermath identity derivation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryAftermathDerivationFailure {
    /// The Relational owner could not retain the exact current branch basis.
    RetentionCapacityExhausted,
    /// The Relational owner permanently exhausted retention identity space.
    RetentionIdentityExhausted,
    /// The Relational owner permanently exhausted snapshot identity space.
    SnapshotIdentityExhausted,
    /// Canonical basis sequence rejected the prepared entries.
    BasisRejected,
    /// Digest preparation rejected the ready sequence under budget.
    DigestRejected,
    /// Correlation basis carried an empty required text field.
    EmptyCorrelationBasis,
    /// A declared external effect did not carry its installed typed emission.
    MissingExternalPayload,
    /// The installed runtime clock could not supply the classification sample.
    RuntimeTimeUnavailable,
}
