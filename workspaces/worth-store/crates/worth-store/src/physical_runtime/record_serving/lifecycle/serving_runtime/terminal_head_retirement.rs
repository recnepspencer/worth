//! Serving's reach to the owners of one terminal head retired.

use worth_store_physical_format::ReleaseCustodyHeadKeyV1;

use super::ServingPhysicalRuntime;
use crate::physical_runtime::{
    durability::{
        AdmittedTerminalHeadRetirement, PhysicalBlobSessionClaim, PhysicalReclaimAttempt,
        ReleaseCertificateCapacityDenial, ReleaseHeadCapacityCharge, TerminalHeadNoRetryClaim,
        TerminalHeadPublicationExcluded, TerminalHeadRetirementAdmissionDenial,
        TerminalHeadRetryClaimDenial,
    },
    CompletedPhysicalMutation, PhysicalProtectedRootObservation,
};

impl ServingPhysicalRuntime {
    pub(in crate::physical_runtime) fn admit_terminal_head_retirement(
        &self,
        claim: &mut PhysicalBlobSessionClaim,
        inspector: PhysicalProtectedRootObservation,
        key: ReleaseCustodyHeadKeyV1,
        charge: ReleaseHeadCapacityCharge,
    ) -> Result<AdmittedTerminalHeadRetirement, TerminalHeadRetirementAdmissionDenial> {
        self.parts
            .publication
            .admit_terminal_head_retirement(claim, inspector, key, charge)
    }

    pub(in crate::physical_runtime) fn attest_no_terminal_head_retry_claim(
        &self,
        publication: &TerminalHeadPublicationExcluded,
    ) -> Result<TerminalHeadNoRetryClaim, TerminalHeadRetryClaimDenial> {
        self.parts
            .publication
            .attest_no_terminal_head_retry_claim(publication)
    }

    pub(in crate::physical_runtime) fn commit_terminal_head_retirement(
        &self,
        attempt: &PhysicalReclaimAttempt,
        completed: &CompletedPhysicalMutation,
    ) -> Result<(), ReleaseCertificateCapacityDenial> {
        self.parts
            .publication
            .commit_terminal_head_retirement(attempt, completed)
    }
}
