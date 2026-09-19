#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryEntityResolutionDenialKind {
    Cancelled,
    DeadlineExceeded,
    PrimaryGraphNotInstalled,
    FieldNotInstalled,
    ValueEncodingRejected,
    EqualityIndexUnavailable,
    UnknownEntity,
    AmbiguousEntity,
    CorruptIdentityIndex,
    ProjectionWorkBudgetExceeded,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
    ForeignResolutionTruth,
}

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
