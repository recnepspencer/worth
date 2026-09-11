use bank_domain::queries::{
    AccountAuthorizedUsersQuery, AccountAuthorizedUsersQueryResult, AccountAuthorizedUsersRequest,
    AccountDetailQuery, AccountDetailRequest, AccountDiscoveryQuery, AccountDiscoveryRequest,
    EstateCaseOverviewQuery, EstateCaseOverviewRequest, EstateCustomerDisclosure,
    EstateCustomerDisclosureQuery, EstateCustomerDisclosureRequest, EstateEmergencyAccountDetails,
    EstateEmergencyAccountDetailsQuery, EstateEmergencyAccountDetailsRequest,
    InstitutionAuditQuery, InstitutionAuditRequest, PaymentDetailQuery, PaymentDetailRequest,
    PendingPaymentsQuery, PendingPaymentsRequest,
};
use bank_domain::queries::{AccountSummaryQuery, AccountSummaryRequest};
use bank_domain::reads::{
    AccountDetail, AccountSummary, EstateCaseOverview, InstitutionAuditView, PaymentSummary,
    VisibleAccount,
};
use bank_domain::schema::{
    AccountIdentity, EstateCaseIdentityField, InstitutionIdentityField, PaymentIdentityField,
    PrincipalIdentityField,
};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    publication::domain_computation::WorthQueryPublishedApplicationResult,
};

use super::BankReadyQuery;
use crate::application_query::{
    execute_estate_customer_disclosure, execute_estate_emergency_account_details, execute_one_shot,
    BankAdmittedEstateEmergencyAccountDetailsHistorical,
    BankAdmittedEstateEmergencyAccountDetailsPreview, BankApplicationQueryDenial,
    BankApplicationQueryInvocation, BankEstateEmergencyAccountDetailsAdmission, BankPreviewSession,
};
use crate::BankApprovedEstateElevation;

impl BankReadyQuery<'_, '_, AccountSummaryRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountSummaryQuery, AccountSummary>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                AccountSummaryQuery::reference(),
                AccountIdentity::reference(),
                self.query.account(),
                ApplicationQueryParameterSet::<AccountSummaryQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}

impl BankReadyQuery<'_, '_, AccountDiscoveryRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountDiscoveryQuery, VisibleAccount>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                AccountDiscoveryQuery::reference(),
                PrincipalIdentityField::reference(),
                self.principal.principal_id(),
                ApplicationQueryParameterSet::<AccountDiscoveryQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}

impl BankReadyQuery<'_, '_, AccountDetailRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<AccountDetailQuery, AccountDetail>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                AccountDetailQuery::reference(),
                AccountIdentity::reference(),
                self.query.account(),
                ApplicationQueryParameterSet::<AccountDetailQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
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
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                AccountAuthorizedUsersQuery::reference(),
                AccountIdentity::reference(),
                self.query.account(),
                ApplicationQueryParameterSet::<AccountAuthorizedUsersQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}

impl BankReadyQuery<'_, '_, PaymentDetailRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<PaymentDetailQuery, PaymentSummary>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                PaymentDetailQuery::reference(),
                PaymentIdentityField::reference(),
                self.query.payment(),
                ApplicationQueryParameterSet::<PaymentDetailQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}

impl BankReadyQuery<'_, '_, PendingPaymentsRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<PendingPaymentsQuery, PaymentSummary>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                PendingPaymentsQuery::reference(),
                PrincipalIdentityField::reference(),
                self.principal.principal_id(),
                ApplicationQueryParameterSet::<PendingPaymentsQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}

impl BankReadyQuery<'_, '_, EstateCaseOverviewRequest> {
    pub fn execute(
        self,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateCaseOverviewQuery, EstateCaseOverview>,
        BankApplicationQueryDenial,
    > {
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                EstateCaseOverviewQuery::reference(),
                EstateCaseIdentityField::reference(),
                self.query.estate(),
                ApplicationQueryParameterSet::<EstateCaseOverviewQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }

    pub fn preview(
        self,
        session: &BankPreviewSession,
    ) -> Result<
        WorthQueryPublishedApplicationResult<EstateCaseOverviewQuery, EstateCaseOverview>,
        BankApplicationQueryDenial,
    > {
        let application = self.runtime.application_runtime();
        let selected = session.select(application, self.controls.maximum_work())?;
        let query = application
            .installed_schema()
            .application_query(EstateCaseOverviewQuery::reference())
            .map_err(BankApplicationQueryDenial::from_installation)?;
        let scope = selected
            .resolve_entity(
                EstateCaseIdentityField::reference(),
                self.query.estate(),
                self.controls.request(),
                worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        let access =
            worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAccessContext::<
                bank_domain::schema::BankSchema,
                bank_domain::schema::Principal,
                bank_domain::model::BankPrincipalId,
                bank_domain::schema::EstateCase,
            >::new(self.principal.query(), &scope);
        let plan = selected
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::<EstateCaseOverviewQuery>::new(),
                self.controls.application_query_controls(),
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        let result = application
            .execute_application_query_one_shot(plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(
            worth_query_host::facade::publication::domain_computation::publish_application_result(
                result.into_admitted_disclosed(),
            ),
        )
    }
}

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
        execute_estate_customer_disclosure(self.runtime, self.principal, self.query, &self.controls)
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
        let controls = self.controls.application_query_controls();
        execute_one_shot(
            self.runtime,
            self.principal,
            BankApplicationQueryInvocation::new(
                InstitutionAuditQuery::reference(),
                InstitutionIdentityField::reference(),
                self.query.institution(),
                ApplicationQueryParameterSet::<InstitutionAuditQuery>::new(),
                controls,
                self.controls.request(),
            ),
        )
    }
}
