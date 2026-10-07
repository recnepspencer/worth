/// Why an entity could not be resolved from its identity field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEntityResolutionDenialKind {
    Handle(crate::facade::primary_graph::WorthQueryHandleDenial),
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
    /// The exact selected lookup could not reserve its preparation backing.
    ProjectionPreparationMemoryExhausted,
    /// A complete selection needs a nonzero finite candidate limit.
    InvalidCandidateLimit,
    /// The complete equality result exceeds the caller's candidate limit.
    CandidateLimitExceeded {
        maximum: usize,
    },
    /// The installed limit on concurrently active snapshots was reached.
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
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

impl From<crate::facade::primary_graph::WorthQueryHandleDenial>
    for WorthQueryEntityResolutionDenial
{
    fn from(denial: crate::facade::primary_graph::WorthQueryHandleDenial) -> Self {
        Self::new(
            WorthQueryEntityResolutionDenialKind::Handle(denial),
            "application handle",
        )
    }
}
