use bank_domain::{
    model::BankPrincipalId,
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityQueryBinding, EstateEmergencyAccessActivityQueryParameters,
    },
    schema::{BankSchema, EstateCase, Principal},
};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        WorthQueryAdmittedApplicationQueryPlan, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryProductQueryControls, WorthQuerySelectedProductOperation,
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
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        self.with_admitted(selected, |application, plan| {
            let result = application
                .execute_application_query_one_shot(plan)
                .map_err(BankApplicationQueryDenial::from_execution)?;
            Ok(publish_application_result(result.into_admitted_disclosed()))
        })
    }

    pub(crate) fn historical<Output>(
        self,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityHistorical<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let selected = self
            .approved
            .select_approval_product(application, self.controls.maximum_work())?;
        self.with_admitted(selected, |application, plan| {
            after_admission(BankAdmittedEstateEmergencyAccessActivityHistorical {
                application,
                plan,
            })
        })
    }

    pub(crate) fn preview<Output>(
        self,
        session: &BankPreviewSession,
        after_admission: impl for<'admitted> FnOnce(
            BankAdmittedEstateEmergencyAccessActivityPreview<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let selected = session.select(
            self.runtime.application_runtime(),
            self.controls.maximum_work(),
        )?;
        self.with_admitted(selected, |application, plan| {
            after_admission(BankAdmittedEstateEmergencyAccessActivityPreview { application, plan })
        })
    }

    pub(super) fn with_admitted<Output>(
        self,
        selected: WorthQuerySelectedProductOperation<'_, BankSchema>,
        after_admission: impl for<'admitted> FnOnce(
            &'admitted WorthQueryPrimaryGraphApplicationRuntime<BankSchema>,
            ActivityPlan<'admitted>,
        )
            -> Result<Output, BankApplicationQueryDenial>,
    ) -> Result<Output, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let query_binding = application
            .installed_schema()
            .installed_query_binding::<EstateEmergencyAccessActivityQueryBinding>()
            .map_err(BankApplicationQueryDenial::from_installation)?;

        let query = query_binding.query();
        let capability = application
            .installed_schema()
            .capability(
                bank_domain::schema::ViewEstateEmergencyProtectionCapability::reference(),
                bank_domain::schema::ViewRestrictedEstateOperation::reference(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_installation)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                self.approved.query(),
                self.principal.query(),
                &capability,
                self.request.capability_request(),
                self.controls.request(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_admission)?;
        let scope = selected
            .resolve_entity(
                bank_domain::schema::EstateCaseIdentityField::reference(),
                self.request.estate(),
                self.controls.request(),
                worth_query_host::facade::primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        let access =
            worth_query_host::facade::primary_graph::WorthQueryApplicationQueryAccessContext::<
                BankSchema,
                Principal,
                BankPrincipalId,
                EstateCase,
            >::new(self.principal.query(), &scope);
        let plan = selected
            .admit_governed_application_query(
                query,
                &access,
                capability_access,
                ApplicationQueryParameterSet::<EstateEmergencyAccessActivityQuery>::new(),
                WorthQueryProductQueryControls::new(
                    self.controls.maximum_result_count(),
                    self.controls.maximum_work(),
                    self.controls.request(),
                ),
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        after_admission(application, plan)
    }
}
