pub(crate) use worth_store_physical_backend::MediaOperationRole;

#[derive(Clone, Copy)]
pub(crate) struct FaultCase {
    pub(crate) id: &'static str,
    pub(crate) expected_outcome: &'static str,
    pub(crate) manifest: ManifestPosture,
    pub(crate) abrupt: bool,
    pub(crate) expected_fault: Option<FaultExpectation>,
}

#[derive(Clone, Copy)]
pub(crate) enum ManifestPosture {
    Absent,
    ScaffoldOnly,
    ScaffoldWithLock,
    EmptyStagedIdentity,
    FullStagedIdentity,
    Published,
    PublishedWithQualificationResidue,
}

impl ManifestPosture {
    pub(crate) const fn identity_visible(self) -> bool {
        matches!(
            self,
            Self::Published | Self::PublishedWithQualificationResidue
        )
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FaultExpectation {
    pub(crate) role: MediaOperationRole,
    pub(crate) ordinal: u64,
    pub(crate) terminal: &'static str,
    pub(crate) operation_bound: bool,
}

pub(crate) const fn fault(
    id: &'static str,
    expected_outcome: &'static str,
    manifest: ManifestPosture,
    role: MediaOperationRole,
    ordinal: u64,
    terminal: &'static str,
    operation_bound: bool,
) -> FaultCase {
    FaultCase {
        id,
        expected_outcome,
        manifest,
        abrupt: false,
        expected_fault: Some(FaultExpectation {
            role,
            ordinal,
            terminal,
            operation_bound,
        }),
    }
}

pub(crate) const fn abrupt(
    id: &'static str,
    manifest: ManifestPosture,
    role: MediaOperationRole,
    ordinal: u64,
    operation_bound: bool,
) -> FaultCase {
    FaultCase {
        id,
        expected_outcome: "abrupt-death",
        manifest,
        abrupt: true,
        expected_fault: Some(FaultExpectation {
            role,
            ordinal,
            terminal: "abrupt",
            operation_bound,
        }),
    }
}

pub(crate) const CI_CASES: [FaultCase; 5] = [
    FaultCase {
        id: "pass-through",
        expected_outcome: "success",
        manifest: crate::partial_effects::fault_cases::ManifestPosture::Published,
        abrupt: false,
        expected_fault: None,
    },
    fault(
        "before-root-creation",
        "denied",
        crate::partial_effects::fault_cases::ManifestPosture::Absent,
        worth_store_physical_backend::MediaOperationRole::CreateDirectory,
        1,
        "denied",
        false,
    ),
    fault(
        "short-identity-prefix",
        "success",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
        1,
        "partial",
        true,
    ),
    fault(
        "directory-barrier-indeterminate",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
        1,
        "denied",
        true,
    ),
    abrupt(
        "abrupt-after-replacement",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::AtomicReplace,
        1,
        true,
    ),
];

pub(crate) const RELEASE_CASES: [FaultCase; 17] = [
    fault(
        "after-fixed-directories",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::ScaffoldOnly,
        worth_store_physical_backend::MediaOperationRole::CreateDirectory,
        4,
        "indeterminate",
        false,
    ),
    fault(
        "before-staged-identity-create",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::ScaffoldWithLock,
        worth_store_physical_backend::MediaOperationRole::CreateNew,
        1,
        "denied",
        true,
    ),
    fault(
        "after-staged-identity-create",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::EmptyStagedIdentity,
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
        1,
        "denied",
        true,
    ),
    fault(
        "after-complete-identity-write",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::FullStagedIdentity,
        worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        1,
        "denied",
        true,
    ),
    fault(
        "file-barrier-denial",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::FullStagedIdentity,
        worth_store_physical_backend::MediaOperationRole::SynchronizeFileState,
        1,
        "denied",
        true,
    ),
    fault(
        "after-identity-file-sync",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::FullStagedIdentity,
        worth_store_physical_backend::MediaOperationRole::AtomicReplace,
        1,
        "denied",
        true,
    ),
    abrupt(
        "after-directory-sync-before-observation",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::SynchronizeRootParentPublication,
        1,
        true,
    ),
    fault(
        "qualification-positioned-write",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::PositionedWrite,
        2,
        "partial",
        true,
    ),
    fault(
        "qualification-append",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::Append,
        1,
        "denied",
        true,
    ),
    fault(
        "qualification-truncate",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::Truncate,
        1,
        "denied",
        true,
    ),
    fault(
        "qualification-allocation",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::Allocate,
        1,
        "denied",
        true,
    ),
    fault(
        "qualification-metadata",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::ReadMetadata,
        1,
        "denied",
        true,
    ),
    fault(
        "qualification-list",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::ListDirectory,
        1,
        "denied",
        true,
    ),
    fault(
        "cleanup-delete",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::PublishedWithQualificationResidue,
        worth_store_physical_backend::MediaOperationRole::Delete,
        1,
        "denied",
        true,
    ),
    fault(
        "cleanup-directory-barrier",
        "failed",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::SynchronizeDirectoryPublication,
        3,
        "denied",
        true,
    ),
    abrupt(
        "before-lock-release",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::ReleaseMutationLease,
        1,
        false,
    ),
    abrupt(
        "after-lock-release",
        crate::partial_effects::fault_cases::ManifestPosture::Published,
        worth_store_physical_backend::MediaOperationRole::ReleaseMutationLease,
        1,
        false,
    ),
];
