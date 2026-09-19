use bank_domain::queries::{
    AccountAuthorizedUsersQuery, AccountAuthorizedUsersQueryResult, AccountAuthorizedUsersRequest,
    AccountDetailQuery, AccountDetailRequest, AccountDiscoveryQuery, AccountDiscoveryRequest,
    EstateCaseOverviewQuery, EstateCaseOverviewRequest, EstateEmergencyAccountDetails,
    EstateEmergencyAccountDetailsQuery, EstateEmergencyAccountDetailsRequest,
    InstitutionAuditQuery, InstitutionAuditRequest, PaymentDetailQuery, PaymentDetailRequest,
    PendingPaymentsQuery, PendingPaymentsRequest,
};
use bank_domain::queries::{AccountSummaryQuery, AccountSummaryRequest};
use bank_domain::reads::{
    AccountDetail, AccountSummary, EstateCaseOverview, InstitutionAuditView, PaymentSummary,
    VisibleAccount,
};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    declaration::{
        application_query::{
            ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
        },
        application_schema::ApplicationStructuredValueBinding,
    },
    primary_graph::WorthQueryApplicationProjection,
    publication::domain_computation::WorthQueryPublishedApplicationResult,
};

use super::BankReadyQuery;
use crate::application_query::{
    execute_estate_emergency_account_details, BankAdmittedEstateEmergencyAccountDetailsHistorical,
    BankAdmittedEstateEmergencyAccountDetailsPreview, BankApplicationQueryDenial,
    BankEstateEmergencyAccountDetailsAdmission, BankPreviewSession,
};
use crate::BankApprovedEstateElevation;

impl<QueryInput> BankReadyQuery<'_, '_, QueryInput>
where
    QueryInput: ApplicationQueryIntent<BankSchema>,
    <QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            BankSchema,
            <QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::PrincipalIdentity,
        >,
    <<QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
        WorthQueryApplicationProjection<
            BankSchema,
            <QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::Query,
        >,
{
    fn execute_current(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            <QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::Query,
            <<QueryInput::Binding as ApplicationQueryBinding<BankSchema>>::ResultBinding as ApplicationStructuredValueBinding>::Value,
        >,
        BankApplicationQueryDenial,
    > {
        let result = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute()
            .map_err(BankApplicationQueryDenial::from_request_query)?;
        Ok(result)
    }
}

impl BankReadyQuery<'_, '_, AccountSummaryRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountSummaryQuery, AccountSummary>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, AccountDiscoveryRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountDiscoveryQuery, VisibleAccount>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, AccountDetailRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountDetailQuery, AccountDetail>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, AccountAuthorizedUsersRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            AccountAuthorizedUsersQuery,
            AccountAuthorizedUsersQueryResult,
        >,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, PaymentDetailRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<PaymentDetailQuery, PaymentSummary>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, PendingPaymentsRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<PendingPaymentsQuery, PaymentSummary>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}

impl BankReadyQuery<'_, '_, EstateCaseOverviewRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateCaseOverviewQuery, EstateCaseOverview>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }

    pub fn preview(
        self,
        session: &BankPreviewSession,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateCaseOverviewQuery, EstateCaseOverview>,
        BankApplicationQueryDenial,
    > {
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .at(session.observation())
            .query(self.query)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .execute()
            .map_err(BankApplicationQueryDenial::from_request_query)
    }
}

impl BankReadyQuery<'_, '_, EstateEmergencyAccountDetailsRequest> {
    pub fn execute_with_approved_elevation(
        self,
        approved: &BankApprovedEstateElevation,
    ) -> Result<
        WorthQueryPublishedApplicationResult<
            EstateEmergencyAccountDetailsQuery,
            EstateEmergencyAccountDetails,
        >,
        BankApplicationQueryDenial,
    > {
        execute_estate_emergency_account_details(
            self.runtime,
            self.principal,
            self.query,
            approved,
            &self.controls,
        )
    }

    pub fn admit_historical_with_approved_elevation<Output>(
        self,
        approved: &BankApprovedEstateElevation,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccountDetailsHistorical<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        BankEstateEmergencyAccountDetailsAdmission::new(
            self.runtime,
            self.principal,
            self.query,
            approved,
            &self.controls,
        )
        .historical(after_admission)
    }

    pub fn admit_preview_with_approved_elevation<Output>(
        self,
        approved: &BankApprovedEstateElevation,
        session: &BankPreviewSession,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccountDetailsPreview<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        BankEstateEmergencyAccountDetailsAdmission::new(
            self.runtime,
            self.principal,
            self.query,
            approved,
            &self.controls,
        )
        .preview(session, after_admission)
    }
}

impl BankReadyQuery<'_, '_, InstitutionAuditRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<InstitutionAuditQuery, InstitutionAuditView>,
        BankApplicationQueryDenial,
    > {
        self.execute_current()
    }
}
