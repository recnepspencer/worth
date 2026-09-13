use worth_store_physical_isolation::{CompactionOwnerCaseId, CompactionOwnerCaseObservation};

use super::CompactionVisibilityAction;

pub fn map_compaction_observation(
    observation: CompactionOwnerCaseObservation,
) -> CompactionVisibilityAction {
    map_compaction_case(observation.id())
}

pub(crate) const fn map_compaction_case(
    owner_case: CompactionOwnerCaseId,
) -> CompactionVisibilityAction {
    match owner_case {
        CompactionOwnerCaseId::LowerRewrite => CompactionVisibilityAction::LowerRewrite,
        CompactionOwnerCaseId::PublishRewrite => CompactionVisibilityAction::PublishRewrite,
        CompactionOwnerCaseId::AdmitRecoveryVisibility => {
            CompactionVisibilityAction::AdmitRecoveryVisibility
        }
        CompactionOwnerCaseId::DeferReclaim => CompactionVisibilityAction::DeferReclaim,
        CompactionOwnerCaseId::DrainReclaimAfterReadRelease => {
            CompactionVisibilityAction::DrainReclaimAfterReadRelease
        }
        CompactionOwnerCaseId::InPlaceOverwriteDenied => {
            CompactionVisibilityAction::DenyInPlaceOverwrite
        }
        CompactionOwnerCaseId::EarlyReclaimDenied => CompactionVisibilityAction::DenyEarlyReclaim,
        CompactionOwnerCaseId::StaleEpochReuseDenied => {
            CompactionVisibilityAction::DenyStaleEpochReuse
        }
        CompactionOwnerCaseId::BackendResidueCandidateSelectionDenied => {
            CompactionVisibilityAction::DenyBackendResidueCandidateSelection
        }
        CompactionOwnerCaseId::LatchHierarchyInversionDenied => {
            CompactionVisibilityAction::DenyLatchHierarchyInversion
        }
        CompactionOwnerCaseId::MixedRootReadDenied => CompactionVisibilityAction::DenyMixedRootRead,
    }
}
