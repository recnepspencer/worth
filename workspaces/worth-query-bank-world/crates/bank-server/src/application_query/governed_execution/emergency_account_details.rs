use bank_domain::{
    model::BankPrincipalId,
    queries::{
        EstateEmergencyAccountDetails, EstateEmergencyAccountDetailsQuery,
        EstateEmergencyAccountDetailsQueryParameters, EstateEmergencyAccountDetailsRequest,
    },
    schema::{
        BankSchema, EstateCase, Principal, ViewEstateEmergencyProtectionCapability,
        ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    primary_graph::{
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryPrimaryGraphApplicationRuntime,
    },
    publication::domain_computation::{
        publish_application_result, WorthQueryPublishedApplicationResult,
    },
};

use super::super::{BankApplicationQueryDenial, BankPreviewSession};
use crate::{
    BankApprovedEstateElevation, BankAuthenticatedPrincipal, BankIdentityRuntime, BankReadControls,
};

type EmergencyAccountDetailsPlan<'a> = WorthQueryAdmittedApplicationQueryPlan<
    'a,
    BankSchema,
    EstateEmergencyAccountDetailsQuery,
    EstateEmergencyAccountDetailsQueryParameters,
    EstateEmergencyAccountDetails,
    Principal,
    BankPrincipalId,
    EstateCase,
>;

pub type BankEstateEmergencyAccountDetailsResult = WorthQueryPublishedApplicationResult<
    EstateEmergencyAccountDetailsQuery,
    EstateEmergencyAccountDetails,
>;

/// Opaque historical-lane authority admitted by Query for one exact elevation.
pub struct BankAdmittedEstateEmergencyAccountDetailsHistorical<'a> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    plan: EmergencyAccountDetailsPlan<'a>,
}

/// Opaque preview-lane authority admitted by Query for one exact elevation.
pub struct BankAdmittedEstateEmergencyAccountDetailsPreview<'a> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    plan: EmergencyAccountDetailsPlan<'a>,
}

pub(crate) struct BankEstateEmergencyAccountDetailsAdmission<'a> {
    runtime: &'a BankIdentityRuntime,
    principal: &'a BankAuthenticatedPrincipal,
    request: EstateEmergencyAccountDetailsRequest,
    approved: &'a BankApprovedEstateElevation,
    controls: &'a BankReadControls,
}

impl BankAdmittedEstateEmergencyAccountDetailsHistorical<'_> {
    pub fn execute(
        self,
    ) -> Result<BankEstateEmergencyAccountDetailsResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl BankAdmittedEstateEmergencyAccountDetailsPreview<'_> {
    pub fn execute(
        self,
    ) -> Result<BankEstateEmergencyAccountDetailsResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl<'a> BankEstateEmergencyAccountDetailsAdmission<'a> {
    pub(crate) const fn new(
        runtime: &'a BankIdentityRuntime,
        principal: &'a BankAuthenticatedPrincipal,
        request: EstateEmergencyAccountDetailsRequest,
        approved: &'a BankApprovedEstateElevation,
        controls: &'a BankReadControls,
    ) -> Self {
        Self {
            runtime,
            principal,
            request,
            approved,
            controls,
        }
    }

    pub(crate) fn historical<Output>(
        self,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccountDetailsHistorical<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let request = application.request(self.principal.external(), self.controls.request());
        let retained = request
            .at_approved_elevation(self.approved.query(), self.controls.maximum_work())
            .map_err(BankApplicationQueryDenial::from_history_selection)?;
        let capability_input = self.request.capability_request();
        retained
            .query(self.request)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .admit_approved_retained(
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                |application, plan| {
                    after_admission(BankAdmittedEstateEmergencyAccountDetailsHistorical {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?
    }

    pub(crate) fn preview<Output>(
        self,
        session: &BankPreviewSession,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccountDetailsPreview<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let capability_input = self.request.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .at(session.observation())
            .query(self.request)
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
            .admit_approved_retained(
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                |application, plan| {
                    after_admission(BankAdmittedEstateEmergencyAccountDetailsPreview {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?
    }
}

pub(crate) fn execute_estate_emergency_account_details(
    runtime: &BankIdentityRuntime,
    principal: &BankAuthenticatedPrincipal,
    request: EstateEmergencyAccountDetailsRequest,
    approved: &BankApprovedEstateElevation,
    controls: &BankReadControls,
) -> Result<BankEstateEmergencyAccountDetailsResult, BankApplicationQueryDenial> {
    let capability_input = request.capability_request();
    runtime
        .application_runtime()
        .request(principal.external(), controls.request())
        .query(request)
        .limits(controls.maximum_result_count(), controls.maximum_work())
        .execute_approved(
            approved.query(),
            ViewEstateEmergencyProtectionCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
            capability_input,
        )
        .map_err(BankApplicationQueryDenial::from_request_query)
}
