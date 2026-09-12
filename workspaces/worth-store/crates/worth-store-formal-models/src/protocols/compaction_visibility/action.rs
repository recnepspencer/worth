#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModeledOutcome<D> {
    Admitted,
    Denied(D),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompactionVisibilityAction {
    LsmMembership {
        operation: LsmMembershipAction,
        outcome: ModeledOutcome<LsmMembershipDenial>,
    },
    LsmExecution {
        operation: LsmExecutionAction,
        outcome: ModeledOutcome<LsmExecutionDenial>,
    },
    LsmMaintenance {
        operation: LsmMaintenanceAction,
        outcome: ModeledOutcome<LsmMaintenanceDenial>,
    },
    LowerRewrite,
    PublishRewrite,
    ValidateReadPlanCutover,
    DeferReclaim,
    DrainReclaimAfterReadRelease,
    DenyInPlaceOverwrite,
    DenyEarlyReclaim,
    DenyStaleEpochReuse,
    DenyLatchHierarchyInversion,
    DenyMixedRootRead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmMembershipAction {
    Open,
    PersistRecord,
    SelectCompaction,
    ReplaceMembership,
    LookupPublishedReplacement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmMembershipDenial {
    CanonicalKeyRequired,
    DurableRecordBindingMismatch,
    StoreBindingMismatch,
    UnsupportedRecordKind,
    MembershipAmbiguous,
    MembershipIncomplete,
    ValueRecordRequired,
    GenerationRecordRequired,
    TombstoneRecordRequired,
    MembershipStale,
    ManifestMembershipMismatch,
    ReplacementOutputMismatch,
    PhysicalPublicationBindingMismatch,
    PersistedMembershipArtifactInvalid,
    Io,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmExecutionAction {
    PrepareCompaction,
    BindPhysicalCompaction,
    PrepareMembershipActivation,
    PublishCompaction,
    ExecuteReplay,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmExecutionDenial {
    StrategyInvariant,
    CanonicalKeyRequired,
    MemtableDoesNotFollowSortedRuns,
    SortedRunsNotCanonical,
    ReplayTailNotCanonical,
    ReplayBindingMismatch,
    TombstoneRecordRequired,
    ValueRecordRequired,
    GenerationRecordRequired,
    OutputGenerationOverflow,
    OutputPublicationMismatch,
    ManifestPublicationRequired,
    ManifestDoesNotCoverCompaction,
    ManifestMembershipMismatch,
    PersistedMembershipAmbiguous,
    PersistedMembershipIncomplete,
    PersistedMembershipStale,
    PersistedArtifactInvalid,
    PersistedIndexIo,
    PhysicalTargetEpochRequired,
    DurableRecordBindingMismatch,
    RecordKeyScopeMismatch,
    PhysicalPublicationBindingMismatch,
    SelectedOperationKeyMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmMaintenanceAction {
    AdmitRunPublication,
    AdmitReplay,
    AdmitCompaction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LsmMaintenanceDenial {
    ArtifactFamily,
    SecurityScope,
    KeyDomain,
    ConcreteKey,
    Shape,
    RequestAdmission,
    NoEligibleLayout,
    Cost,
    Budget,
    UnexpectedSelectedOperation,
}
