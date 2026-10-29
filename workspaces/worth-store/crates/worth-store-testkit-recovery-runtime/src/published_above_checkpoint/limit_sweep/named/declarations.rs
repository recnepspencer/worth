//! Whether the denial a block carries says only that a limit ran out. A
//! phase that keeps its reader's failure names a limit only where that
//! reader ran out of recovery's own observation bytes.

pub use {worth_store::physical_runtime::FilesystemObservationBound, worth_store::physical_runtime::RecoveryDiscoveryFailure};
pub use {worth_store_recovery_physics::PhysicalRedoPlanningDenial};
pub use {worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial as Planning, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial as Control, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadReadDenial as HeadRead, worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial as HeadWalk, worth_store_recovery_runtime::PhysicalRecoverySelectedRecordReadDenial as RecordRead, worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial as Head, worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial as Candidate};

pub fn reader(failure: &RecoveryDiscoveryFailure) -> bool {
    matches!(
        failure,
        worth_store::physical_runtime::RecoveryDiscoveryFailure::Limit(past)
            if past.dimension() == worth_store::physical_runtime::FilesystemObservationBound::ObservationBytes
    )
}

pub fn record(denial: &RecordRead) -> bool {
    match denial {
        worth_store_recovery_runtime::PhysicalRecoverySelectedRecordReadDenial::ManifestEntryLimit => true,
        worth_store_recovery_runtime::PhysicalRecoverySelectedRecordReadDenial::ManifestRead(failure) | worth_store_recovery_runtime::PhysicalRecoverySelectedRecordReadDenial::ChunkRead { failure, .. } => {
            reader(failure)
        }
        _ => false,
    }
}

pub fn head(denial: &Head) -> bool {
    match denial {
        worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::ManifestEntryLimit
        | worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::RosterEntryLimit
        | worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::ObservationByteLimit
        | worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::HeadWalk(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial::Read(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadReadDenial::ManifestEntryLimit { .. }))
        | worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::Control(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial::ManifestEntryLimit) => true,
        worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::SourceRootRead { failure, .. }
        | worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::HeadWalk(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadWalkDenial::Read(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadReadDenial::Media { failure, .. })) => reader(failure),
        worth_store_recovery_runtime::PhysicalRecoverySelectedReleaseHeadDenial::Control(worth_store_recovery_runtime::PhysicalRecoveryReleaseHeadControlDenial::ControlRead { denial, .. }) => record(denial),
        _ => false,
    }
}

pub fn only_a_limit(denial: &Planning) -> bool {
    match denial {
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Redo(worth_store_recovery_physics::PhysicalRedoPlanningDenial::ProjectionLimit { .. }) => true,
        // Every cost denial is a limit the plan's cost ran past.
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::Cost(_) => true,
        // The successor root is charged its entry before it is probed.
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::ManifestEntryLimit { .. }) => true,
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SuccessorCandidate(worth_store_recovery_runtime::PhysicalRecoverySuccessorCandidateDenial::Discovery { failure, .. }) => reader(failure),
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::SelectedReleaseHead(denial) => head(denial),
        // A pending tier release reads the released directory's records.
        worth_store_recovery_runtime::PhysicalRecoveryPlanningDenial::ReleasedDirectorySource(denial) => record(denial),
        _ => false,
    }
}