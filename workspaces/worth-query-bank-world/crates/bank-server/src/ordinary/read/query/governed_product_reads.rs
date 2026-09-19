use bank_domain::queries::{
    EstateCustomerDisclosure, EstateCustomerDisclosureQuery, EstateCustomerDisclosureRequest,
    EstateGovernanceQuery, EstateGovernanceRequest, EstateLegalComplianceQuery,
    EstateLegalComplianceRequest, EstateLegalComplianceResult, EstateMandatoryReviewQuery,
    EstateMandatoryReviewRequest, EstateMandatoryReviewResult,
};
use bank_domain::reads::EstateGovernanceContext;
use bank_domain::schema::{
    ViewEstateAdministrationCapability, ViewEstateIdentityVerificationCapability,
    ViewEstateLegalComplianceCapability, ViewEstateMandatoryReviewCapability,
    ViewRestrictedEstateOperation,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::publication::domain_computation::WorthQueryPublishedApplicationResult;

use super::BankReadyQuery;
use crate::application_query::BankApplicationQueryDenial;

impl BankReadyQuery<'_, '_, EstateCustomerDisclosureRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            EstateCustomerDisclosureQuery,
            EstateCustomerDisclosure,
        >,
        BankApplicationQueryDenial,
    > {
        let capability_input = self.query.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute_governed(
                ViewEstateIdentityVerificationCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
            )
            .map_err(BankApplicationQueryDenial::from_request_query)
    }
}

impl BankReadyQuery<'_, '_, EstateGovernanceRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>,
        BankApplicationQueryDenial,
    > {
        let capability_input = self.query.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute_governed(
                ViewEstateAdministrationCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
            )
            .map_err(BankApplicationQueryDenial::from_request_query)
    }
}

impl BankReadyQuery<'_, '_, EstateLegalComplianceRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            EstateLegalComplianceQuery,
            EstateLegalComplianceResult,
        >,
        BankApplicationQueryDenial,
    > {
        let capability_input = self.query.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute_governed(
                ViewEstateLegalComplianceCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
            )
            .map_err(BankApplicationQueryDenial::from_request_query)
    }
}

impl BankReadyQuery<'_, '_, EstateMandatoryReviewRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            EstateMandatoryReviewQuery,
            EstateMandatoryReviewResult,
        >,
        BankApplicationQueryDenial,
    > {
        let capability_input = self.query.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute_governed(
                ViewEstateMandatoryReviewCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
            )
            .map_err(BankApplicationQueryDenial::from_request_query)
    }
}
