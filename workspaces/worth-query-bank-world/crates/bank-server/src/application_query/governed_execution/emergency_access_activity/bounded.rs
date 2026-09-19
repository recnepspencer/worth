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
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(self.plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
    }
}

impl BankAdmittedEstateEmergencyAccessActivityPreview<'_> {
    pub fn execute(
        self,
    ) -> Result<BankEstateEmergencyAccessActivityResult, BankApplicationQueryDenial> {
        let result = self
            .application
            .execute_application_query_one_shot(self.plan)
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
            .limits(
                self.controls.maximum_result_count(),
                self.controls.maximum_work(),
            )
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
            BankAdmittedEstateEmergencyAccessActivityHistorical<'admitted>,
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
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
                capability_input,
                |application, plan| {
                    after_admission(BankAdmittedEstateEmergencyAccessActivityHistorical {
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
            BankAdmittedEstateEmergencyAccessActivityPreview<'admitted>,
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
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
                capability_input,
                |application, plan| {
                    after_admission(BankAdmittedEstateEmergencyAccessActivityPreview {
                        application,
                        plan,
                    })
                },
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?
    }
}
