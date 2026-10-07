use super::owner_case::{sealed, ObserveOwnerCase, OwnerCaseObservation};
use crate::materialization::{
    BTreePublicationMaterializationAdmissionCaseId,
    BTreePublicationMaterializationAdmissionOutcome, CatalogRootMaterializationAdmissionCaseId,
    CatalogRootMaterializationAdmissionOutcome, ImportedBlobMaterializationAdmissionCaseId,
    ImportedBlobMaterializationAdmissionOutcome, LsmLookupMaterializationAdmissionCaseId,
    LsmLookupMaterializationAdmissionOutcome, LsmPublicationMaterializationAdmissionCaseId,
    LsmPublicationMaterializationAdmissionOutcome, LsmReplayMaterializationAdmissionCaseId,
    LsmReplayMaterializationAdmissionOutcome,
};

macro_rules! observe_owner_case {
    ($outcome:ty => $case_id:ty) => {
        impl sealed::Sealed for $outcome {}

        impl ObserveOwnerCase for $outcome {
            type CaseId = $case_id;

            fn owner_case_observation(&self) -> OwnerCaseObservation<Self::CaseId> {
                OwnerCaseObservation::issued(self.case_id())
            }
        }
    };
}

observe_owner_case!(CatalogRootMaterializationAdmissionOutcome => CatalogRootMaterializationAdmissionCaseId);
observe_owner_case!(BTreePublicationMaterializationAdmissionOutcome => BTreePublicationMaterializationAdmissionCaseId);
observe_owner_case!(LsmLookupMaterializationAdmissionOutcome => LsmLookupMaterializationAdmissionCaseId);
observe_owner_case!(LsmPublicationMaterializationAdmissionOutcome => LsmPublicationMaterializationAdmissionCaseId);
observe_owner_case!(LsmReplayMaterializationAdmissionOutcome => LsmReplayMaterializationAdmissionCaseId);
observe_owner_case!(ImportedBlobMaterializationAdmissionOutcome => ImportedBlobMaterializationAdmissionCaseId);
