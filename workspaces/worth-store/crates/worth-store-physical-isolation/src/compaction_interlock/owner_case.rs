//! Read-only topology projected from cases issued by physical owners.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompactionCutoverState {
    PlanAdmitted,
    RewriteLowered,
    PublicationCommitted,
    ReadPlanCutoverValidated,
    ReclaimDeferred,
    Reclaimed,
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CompactionOwnerCaseId {
    LowerRewrite,
    PublishRewrite,
    ValidateReadPlanCutover,
    DeferReclaim,
    DrainReclaimAfterReadRelease,
    InPlaceOverwriteDenied,
    EarlyReclaimDenied,
    StaleEpochReuseDenied,
    LatchHierarchyInversionDenied,
    MixedRootReadDenied,
}

impl CompactionOwnerCaseId {
    pub const fn name(self) -> &'static str {
        match self {
            Self::LowerRewrite => "physical.compaction.lower_rewrite",
            Self::PublishRewrite => "physical.compaction.publish_rewrite",
            Self::ValidateReadPlanCutover => "physical.compaction.validate_read_plan_cutover",
            Self::DeferReclaim => "physical.compaction.defer_reclaim",
            Self::DrainReclaimAfterReadRelease => {
                "physical.compaction.drain_reclaim_after_read_release"
            }
            Self::InPlaceOverwriteDenied => "physical.compaction.deny_in_place_overwrite",
            Self::EarlyReclaimDenied => "physical.compaction.deny_early_reclaim",
            Self::StaleEpochReuseDenied => "physical.compaction.deny_stale_epoch_reuse",
            Self::LatchHierarchyInversionDenied => {
                "physical.compaction.deny_latch_hierarchy_inversion"
            }
            Self::MixedRootReadDenied => "physical.compaction.deny_mixed_root_read",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactionOwnerCaseDeclaration {
    id: CompactionOwnerCaseId,
    from: CompactionCutoverState,
    to: CompactionCutoverState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CompactionOwnerCaseObservation {
    declaration: CompactionOwnerCaseDeclaration,
}

impl CompactionOwnerCaseDeclaration {
    pub(super) const fn declared_by_owner(
        id: CompactionOwnerCaseId,
        from: CompactionCutoverState,
        to: CompactionCutoverState,
    ) -> Self {
        Self { id, from, to }
    }

    pub const fn id(self) -> CompactionOwnerCaseId {
        self.id
    }
    pub const fn from(self) -> CompactionCutoverState {
        self.from
    }
    pub const fn to(self) -> CompactionCutoverState {
        self.to
    }
}

impl CompactionOwnerCaseObservation {
    pub(super) const fn issued_by_owner(declaration: CompactionOwnerCaseDeclaration) -> Self {
        Self { declaration }
    }

    pub const fn declaration(self) -> CompactionOwnerCaseDeclaration {
        self.declaration
    }

    pub const fn id(self) -> CompactionOwnerCaseId {
        self.declaration.id()
    }

    pub const fn from(self) -> CompactionCutoverState {
        self.declaration.from()
    }

    pub const fn to(self) -> CompactionCutoverState {
        self.declaration.to()
    }
}
