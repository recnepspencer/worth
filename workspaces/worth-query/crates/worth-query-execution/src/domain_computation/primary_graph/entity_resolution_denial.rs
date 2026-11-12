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

/// Refusal to resolve an entity from its identity field.
///
/// Nothing was read beyond the lookup. [`Self::kind`] says why and
/// [`Self::subject`] names the entity or field.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryEntityResolutionDenial {
    kind: WorthQueryEntityResolutionDenialKind,
    subject: String,
    native_lookup: Option<(
        worth_relational::facade::indexes::DerivedIndexId,
        worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind,
    )>,
}

impl WorthQueryEntityResolutionDenial {
    pub(super) fn new(
        kind: WorthQueryEntityResolutionDenialKind,
        subject: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            subject: subject.into(),
            native_lookup: None,
        }
    }

    pub(super) fn with_native_lookup(
        mut self,
        index: worth_relational::facade::indexes::DerivedIndexId,
        cause: worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind,
    ) -> Self {
        self.native_lookup = Some((index, cause));
        self
    }
    /// The original exact index failure, when Native refused the identity lookup.
    pub const fn native_lookup(
        &self,
    ) -> Option<(
        worth_relational::facade::indexes::DerivedIndexId,
        worth_relational::facade::indexes::BoundedEntityFieldLookupDenialKind,
    )> {
        self.native_lookup
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

impl worth_execution::ChargedBytes for WorthQueryEntityResolutionDenial {
    fn additional_charged_bytes(&self) -> u64 {
        self.subject.capacity() as u64
    }
}
