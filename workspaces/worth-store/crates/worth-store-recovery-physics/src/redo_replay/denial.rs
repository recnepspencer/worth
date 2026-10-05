#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoPlanningDenial {
    MalformedMember,
    WrongDomain,
    /// The admitted members name more redo targets than were admitted for
    /// all of them: `observed` is the count that first passed `admitted`.
    TargetLimit {
        observed: u64,
        admitted: u64,
    },
    /// One more distinct target than was admitted for all members.
    DistinctTargetLimit {
        observed: u64,
        admitted: u64,
    },
    RecoveryMemoryLimit {
        observed: u64,
        admitted: u64,
    },
    InvalidRecordOrder,
    NonCanonicalTargetOrder,
    LsnRangeMismatch,
    InvalidTarget,
    InvalidRecoveryProjection,
    /// The admitted members' projections hold more of one kind of entry than
    /// was admitted for all of them. The members themselves are well formed.
    ProjectionLimit {
        limit: PhysicalRedoProjectionLimit,
        observed: u64,
        admitted: u64,
    },
    UnsupportedRecoveryProjectionVersion(u16),
    MissingPageObservation,
    GenerationMismatch,
    PageDigestMismatch,
    ProvenNoEffectHasWalAttempt,
    CounterOverflow,
    /// A terminal head retirement member was admitted by C.9 but cannot yet
    /// be planned.
    TerminalHeadRetirementUnsupported,
}

impl PhysicalRedoPlanningDenial {
    /// A member is decoded under the targets its predecessors left. This
    /// restates a target limit it ran past in terms of all members.
    pub(crate) const fn after_targets(self, already: u64) -> Self {
        match self {
            Self::TargetLimit { observed, admitted } => Self::TargetLimit {
                observed: observed.saturating_add(already),
                admitted: admitted.saturating_add(already),
            },
            other => other,
        }
    }
}

/// The projection allowance a set of admitted members can exhaust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRedoProjectionLimit {
    Frames,
    RecordIdentities,
    Placements,
    SegmentUpdates,
    Manifests,
    TotalEntries,
    InlineAllocations,
}
