use super::owner_case::{sealed, ObserveOwnerCase, OwnerCaseObservation};
use crate::access::execution::{DegradedScanReadinessCaseId, DegradedScanReadinessOutcome};
use crate::planning::{ImportedBlobReadAdmissionCaseId, ImportedBlobReadAdmissionOutcome};
use crate::strategy::registry::{LayoutAdmissionCaseId, LayoutAdmissionOutcome};
use crate::{
    AccessPlanSelectionCaseId, AccessPlanSelectionOutcome, ArtifactFamilyAdmissionCaseId,
    ArtifactFamilyAdmissionOutcome, BaselineLsmLookupAdmissionCaseId,
    BaselineLsmLookupAdmissionOutcome, BaselineLsmLookupCaseId, BaselineLsmLookupExecution,
    BootstrapCatalogReadCaseId, BootstrapCatalogReadOutcome, FullDeclaredScanCaseId,
    FullDeclaredScanOutcome, PhysicalKeyDomainAdmissionCaseId, PhysicalKeyDomainAdmissionOutcome,
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

observe_owner_case!(DegradedScanReadinessOutcome => DegradedScanReadinessCaseId);
observe_owner_case!(ImportedBlobReadAdmissionOutcome => ImportedBlobReadAdmissionCaseId);
observe_owner_case!(BaselineLsmLookupAdmissionOutcome => BaselineLsmLookupAdmissionCaseId);
observe_owner_case!(BaselineLsmLookupExecution => BaselineLsmLookupCaseId);
observe_owner_case!(ArtifactFamilyAdmissionOutcome => ArtifactFamilyAdmissionCaseId);
observe_owner_case!(PhysicalKeyDomainAdmissionOutcome => PhysicalKeyDomainAdmissionCaseId);
observe_owner_case!(BootstrapCatalogReadOutcome => BootstrapCatalogReadCaseId);
observe_owner_case!(LayoutAdmissionOutcome => LayoutAdmissionCaseId);
observe_owner_case!(AccessPlanSelectionOutcome => AccessPlanSelectionCaseId);
observe_owner_case!(FullDeclaredScanOutcome => FullDeclaredScanCaseId);
