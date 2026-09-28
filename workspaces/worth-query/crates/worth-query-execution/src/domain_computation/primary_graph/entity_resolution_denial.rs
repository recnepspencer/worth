/// Why an entity could not be resolved from its identity field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEntityResolutionDenialKind {
    /// Resolution was cancelled.
    Cancelled,
    /// Resolution reached its deadline.
    DeadlineExceeded,
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
    /// Resolution exceeded its work budget.
    ProjectionWorkBudgetExceeded,
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

/// Refusal to resolve an entity from its identity field.
///
/// Nothing was read beyond the lookup. [`Self::kind`] says why and
/// [`Self::subject`] names the entity or field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEntityResolutionDenial {
    kind: WorthQueryEntityResolutionDenialKind,
    subject: String,
}

impl WorthQueryEntityResolutionDenial {
    pub(super) fn new(
        kind: WorthQueryEntityResolutionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }

    pub const fn kind(&self) -> WorthQueryEntityResolutionDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl std::fmt::Display for WorthQueryEntityResolutionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "application entity resolution denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryEntityResolutionDenial {}
