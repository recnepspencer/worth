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
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<BankEstateEmergencyAccountDetailsResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(phase, self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl BankAdmittedEstateEmergencyAccountDetailsPreview<'_> {
    pub fn execute(
        self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<BankEstateEmergencyAccountDetailsResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(phase, self.plan)
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
            &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            BankAdmittedEstateEmergencyAccountDetailsHistorical<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        application.with_application_advancement(self.controls.request(), |phase| {
        let application = self.runtime.application_runtime();
        let request = application.request(self.principal.external(), self.controls.request());
        let retained = request
            .at_approved_elevation(&phase, self.approved.query(), None)
            .map_err(BankApplicationQueryDenial::from_history_selection)?;
        let capability_input = self.request.capability_request();
        retained
            .query(self.request)
            .limit_results(
                self.controls.maximum_result_count(),

            )
            .admit_approved_retained(
                &phase,
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                |phase, application, plan| {
                    after_admission(phase, BankAdmittedEstateEmergencyAccountDetailsHistorical {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?

        }).map_err(|cause| BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ExecutionRequest(cause)))?
    }

    pub(crate) fn preview<Output>(
        self,
        session: &BankPreviewSession,
        after_admission: impl for<'admitted> FnOnce(
            &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            BankAdmittedEstateEmergencyAccountDetailsPreview<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        application.with_application_advancement(self.controls.request(), |phase| {
        let capability_input = self.request.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .at(session.observation())
            .query(self.request)
            .limit_results(
                self.controls.maximum_result_count(),

            )
            .admit_approved_retained(
                &phase,
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                capability_input,
                |phase, application, plan| {
                    after_admission(phase, BankAdmittedEstateEmergencyAccountDetailsPreview {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?

        }).map_err(|cause| BankApplicationQueryDenial::from_request_query(
            worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::ExecutionRequest(cause)))?
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
        .limit_results(controls.maximum_result_count())
        .execute_approved(
            approved.query(),
            ViewEstateEmergencyProtectionCapability::reference(),
            ViewRestrictedEstateOperation::reference(),
            capability_input,
        )
        .map_err(BankApplicationQueryDenial::from_request_query)
}
