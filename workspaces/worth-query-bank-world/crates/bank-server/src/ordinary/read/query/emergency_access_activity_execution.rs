use bank_domain::queries::EstateEmergencyAccessActivityRequest;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationLiveControls, WorthQueryApplicationQueryResumeControls,
};

use super::BankReadyQuery;
use crate::application_query::{
    BankAdmittedEstateEmergencyAccessActivityContinuation,
    BankAdmittedEstateEmergencyAccessActivityHistorical,
    BankAdmittedEstateEmergencyAccessActivityPreview, BankApplicationQueryDenial,
    BankEstateEmergencyAccessActivityAdmission, BankEstateEmergencyAccessActivityContinuation,
    BankEstateEmergencyAccessActivityLiveLease, BankEstateEmergencyAccessActivityPageResult,
    BankEstateEmergencyAccessActivityResult, BankPreviewSession,
};
use crate::BankApprovedEstateElevation;

impl<'runtime, 'principal>
    BankReadyQuery<'runtime, 'principal, EstateEmergencyAccessActivityRequest>
{
    pub fn execute_with_approved_elevation(
        self,
        approved: &BankApprovedEstateElevation,
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        self.admission(approved).one_shot()
    }

    pub fn admit_historical_with_approved_elevation<Output>(
        self,
        approved: &BankApprovedEstateElevation,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityHistorical<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        self.admission(approved).historical(after_admission)
    }

    pub fn admit_preview_with_approved_elevation<Output>(
        self,
        approved: &BankApprovedEstateElevation,
        session: &BankPreviewSession,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityPreview<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        self.admission(approved).preview(session, after_admission)
    }

    pub fn page_with_approved_elevation(
        self,
        approved: &BankApprovedEstateElevation,
    ) -> Result<BankEstateEmergencyAccessActivityPageResult, BankApplicationQueryDenial> {
        self.admission(approved).page()
    }

    pub fn resume_with_approved_elevation(
        self,
        approved: &BankApprovedEstateElevation,
        continuation: BankEstateEmergencyAccessActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
    ) -> Result<BankEstateEmergencyAccessActivityPageResult, BankApplicationQueryDenial> {
        self.admission(approved).resume(continuation, controls)
    }

    pub fn readmit_resume_with_approved_elevation<Output>(
        self,
        approved: &BankApprovedEstateElevation,
        continuation: BankEstateEmergencyAccessActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
        after_readmission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityContinuation<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        self.admission(approved)
            .readmit_resume(continuation, controls, after_readmission)
    }

    pub fn subscribe_with_approved_elevation(
        self,
        approved: &BankApprovedEstateElevation,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<BankEstateEmergencyAccessActivityLiveLease<'runtime>, BankApplicationQueryDenial>
    {
        self.admission(approved).subscribe(controls)
    }

    fn admission<'approved, 'controls>(
        &'controls self,
        approved: &'approved BankApprovedEstateElevation,
    ) -> BankEstateEmergencyAccessActivityAdmission<'runtime, 'principal, 'approved, 'controls>
    {
        BankEstateEmergencyAccessActivityAdmission::new(
            self.runtime,
            self.principal,
            self.query,
            approved,
            &self.controls,
        )
    }
}
