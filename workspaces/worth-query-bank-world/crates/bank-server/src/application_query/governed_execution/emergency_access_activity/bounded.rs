use bank_domain::{
    model::BankPrincipalId,
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityQueryParameters,
    },
    schema::{BankSchema, EstateCase, Principal},
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

use super::admission::BankEstateEmergencyAccessActivityAdmission;
use crate::{BankApplicationQueryDenial, BankPreviewSession};

pub type BankEstateEmergencyAccessActivityResult = WorthQueryPublishedApplicationResult<
    EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivity,
>;

pub(super) type ActivityPlan<'a> = WorthQueryAdmittedApplicationQueryPlan<
    'a,
    BankSchema,
    EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivityQueryParameters,
    EstateEmergencyAccessActivity,
    Principal,
    BankPrincipalId,
    EstateCase,
>;

pub struct BankAdmittedEstateEmergencyAccessActivityHistorical<'a> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    plan: ActivityPlan<'a>,
}

pub struct BankAdmittedEstateEmergencyAccessActivityPreview<'a> {
    application: &'a WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
    plan: ActivityPlan<'a>,
}

impl BankAdmittedEstateEmergencyAccessActivityHistorical<'_> {
    pub fn execute(
        self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(phase, self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl BankAdmittedEstateEmergencyAccessActivityPreview<'_> {
    pub fn execute(
        self,
        phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(phase, self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl BankEstateEmergencyAccessActivityAdmission<'_, '_, '_, '_> {
    pub(crate) fn one_shot(
        self,
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        let capability_input = self.request.capability_request();
        self.runtime
            .application_runtime()
            .request(self.principal.external(), self.controls.request())
            .query(self.request)
            .limit_results(self.controls.maximum_result_count())
            .execute_approved(
                self.approved.query(),
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
                capability_input,
            )
            .map_err(BankApplicationQueryDenial::from_request_query)
    }

    pub(crate) fn historical<Output>(
        self,
        after_admission: impl for<'admitted> FnOnce(
            &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
            BankAdmittedEstateEmergencyAccessActivityHistorical<'admitted>,
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
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
                capability_input,
                |phase, application, plan| {
                    after_admission(phase, BankAdmittedEstateEmergencyAccessActivityHistorical {
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
            BankAdmittedEstateEmergencyAccessActivityPreview<'admitted>,
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
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
                capability_input,
                |phase, application, plan| {
                    after_admission(phase, BankAdmittedEstateEmergencyAccessActivityPreview {
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
