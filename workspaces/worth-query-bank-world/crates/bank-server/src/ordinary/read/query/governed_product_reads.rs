use bank_domain::queries::{
    EstateGovernanceQuery, EstateGovernanceRequest, EstateLegalComplianceQuery,
    EstateLegalComplianceRequest, EstateLegalComplianceResult, EstateMandatoryReviewQuery,
    EstateMandatoryReviewRequest, EstateMandatoryReviewResult,
};
use bank_domain::reads::EstateGovernanceContext;
use worth_query_host::facade::publication::domain_computation::{
    publish_application_result, WorthQueryPublishedApplicationResult,
};

use super::BankReadyQuery;
use crate::application_query::{
    execute_estate_governance, execute_estate_legal_compliance, execute_estate_mandatory_review,
    BankApplicationQueryDenial,
};

impl BankReadyQuery<'_, '_, EstateGovernanceRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateGovernanceQuery, EstateGovernanceContext>,
        BankApplicationQueryDenial,
    > {
        let result =
            execute_estate_governance(self.runtime, self.principal, self.query, &self.controls)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
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
        let result = execute_estate_legal_compliance(
            self.runtime,
            self.principal,
            self.query,
            &self.controls,
        )?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
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
        let result = execute_estate_mandatory_review(
            self.runtime,
            self.principal,
            self.query,
            &self.controls,
        )?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}
