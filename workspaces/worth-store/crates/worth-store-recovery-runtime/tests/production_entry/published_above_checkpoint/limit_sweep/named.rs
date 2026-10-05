//! Whether the denial a block carries says only that a limit ran out. A
//! phase that keeps its reader's failure names a limit only where that
//! reader ran out of recovery's own observation bytes.

use worth_store::physical_runtime::{RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure};
use worth_store_recovery_physics::PhysicalRedoPlanningDenial;
use worth_store_recovery_runtime::{
    PhysicalRecoveryPageAdmissionDenial as Page, PhysicalRecoveryPlanningDenial as Planning,
    PhysicalRecoveryReleaseHeadControlDenial as Control,
    PhysicalRecoveryReleaseHeadReadDenial as HeadRead,
    PhysicalRecoveryReleaseHeadWalkDenial as HeadWalk,
    PhysicalRecoverySelectedRecordReadDenial as RecordRead,
    PhysicalRecoverySelectedReleaseHeadDenial as Head,
    PhysicalRecoverySuccessorCandidateDenial as Candidate,
};

fn reader(failure: &RecoveryDiscoveryFailure) -> bool {
    matches!(
        failure,
        RecoveryDiscoveryFailure::ByteLimitExceeded {
            scope: RecoveryDiscoveryByteLimitScope::Observation,
            ..
        }
    )
}

fn page(denial: &Page) -> bool {
    matches!(
        denial,
        Page::ManifestEntryLimit | Page::ObservationByteLimit | Page::StagingByteLimit
    )
}

fn record(denial: &RecordRead) -> bool {
    match denial {
        RecordRead::ManifestEntryLimit => true,
        RecordRead::ManifestRead(failure) | RecordRead::ChunkRead { failure, .. } => {
            reader(failure)
        }
        _ => false,
    }
}

fn head(denial: &Head) -> bool {
    match denial {
        Head::ManifestEntryLimit
        | Head::RosterEntryLimit { .. }
        | Head::ObservationByteLimit
        | Head::HeadWalk(HeadWalk::Read(HeadRead::ManifestEntryLimit { .. }))
        | Head::Control(Control::ManifestEntryLimit) => true,
        Head::SourceRoutes(denial) => page(denial),
        Head::SourceRootRead { failure, .. }
        | Head::HeadWalk(HeadWalk::Read(HeadRead::Media { failure, .. })) => reader(failure),
        Head::Control(Control::ControlRead { denial, .. }) => record(denial),
        _ => false,
    }
}

pub(super) fn only_a_limit(denial: &Planning) -> bool {
    match denial {
        Planning::Page(denial) => page(denial),
        Planning::Redo(PhysicalRedoPlanningDenial::ProjectionLimit { .. }) => true,
        // Every cost denial is a limit the plan's cost ran past.
        Planning::Cost(_) => true,
        Planning::SuccessorCandidate(Candidate::Discovery { failure, .. }) => reader(failure),
        Planning::SelectedReleaseHead(denial) => head(denial),
        _ => false,
    }
}
