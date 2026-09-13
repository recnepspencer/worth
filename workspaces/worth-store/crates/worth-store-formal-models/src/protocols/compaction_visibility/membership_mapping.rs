use worth_store_lsm_authority::{
    LsmMembershipDenial as OwnerDenial, LsmMembershipDisposition as OwnerDisposition,
    LsmMembershipOperation as OwnerOperation, LsmMembershipOwnerCaseId,
    LsmMembershipOwnerCaseObservation,
};

use super::action::{
    CompactionVisibilityAction, LsmMembershipAction, LsmMembershipDenial, ModeledOutcome,
};
pub fn map_lsm_membership_observation(
    observation: LsmMembershipOwnerCaseObservation,
) -> CompactionVisibilityAction {
    map_lsm_membership_case(observation.id())
}

pub(crate) fn map_lsm_membership_case(
    owner_case: LsmMembershipOwnerCaseId,
) -> CompactionVisibilityAction {
    let operation = match owner_case.operation() {
        OwnerOperation::Open => LsmMembershipAction::Open,
        OwnerOperation::PersistRecord => LsmMembershipAction::PersistRecord,
        OwnerOperation::SelectCompaction => LsmMembershipAction::SelectCompaction,
        OwnerOperation::ReplaceMembership => LsmMembershipAction::ReplaceMembership,
        OwnerOperation::LookupPublishedReplacement => {
            LsmMembershipAction::LookupPublishedReplacement
        }
    };
    let outcome = match owner_case.disposition() {
        OwnerDisposition::Admitted => ModeledOutcome::Admitted,
        OwnerDisposition::Denied(denial) => ModeledOutcome::Denied(map_denial(denial)),
    };
    CompactionVisibilityAction::LsmMembership { operation, outcome }
}

const fn map_denial(denial: OwnerDenial) -> LsmMembershipDenial {
    match denial {
        OwnerDenial::CanonicalKeyRequired => LsmMembershipDenial::CanonicalKeyRequired,
        OwnerDenial::DurableRecordBindingMismatch => {
            LsmMembershipDenial::DurableRecordBindingMismatch
        }
        OwnerDenial::StoreBindingMismatch => LsmMembershipDenial::StoreBindingMismatch,
        OwnerDenial::UnsupportedRecordKind => LsmMembershipDenial::UnsupportedRecordKind,
        OwnerDenial::MembershipAmbiguous => LsmMembershipDenial::MembershipAmbiguous,
        OwnerDenial::MembershipIncomplete => LsmMembershipDenial::MembershipIncomplete,
        OwnerDenial::ValueRecordRequired => LsmMembershipDenial::ValueRecordRequired,
        OwnerDenial::GenerationRecordRequired => LsmMembershipDenial::GenerationRecordRequired,
        OwnerDenial::TombstoneRecordRequired => LsmMembershipDenial::TombstoneRecordRequired,
        OwnerDenial::MembershipStale => LsmMembershipDenial::MembershipStale,
        OwnerDenial::ManifestMembershipMismatch => LsmMembershipDenial::ManifestMembershipMismatch,
        OwnerDenial::ReplacementOutputMismatch => LsmMembershipDenial::ReplacementOutputMismatch,
        OwnerDenial::PhysicalPublicationBindingMismatch => {
            LsmMembershipDenial::PhysicalPublicationBindingMismatch
        }
        OwnerDenial::PersistedMembershipArtifactInvalid => {
            LsmMembershipDenial::PersistedMembershipArtifactInvalid
        }
        OwnerDenial::Io => LsmMembershipDenial::Io,
    }
}
