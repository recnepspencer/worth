use bank_domain::{
    model::BankPrincipalId,
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityLiveCause,
        EstateEmergencyAccessActivityQuery, EstateEmergencyAccessActivityQueryBinding,
        EstateEmergencyAccessActivityQueryParameters,
    },
    schema::{
        BankSchema, EmergencyAccess, EstateCase, EstateCaseIdentityField, Principal,
        ViewEstateEmergencyProtectionCapability, ViewRestrictedEstateOperation,
    },
};
use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        WorthQueryApplicationLiveControls, WorthQueryApplicationLiveLease,
        WorthQueryApplicationLiveOutcome, WorthQueryPrincipalResolutionMode,
    },
    publication::domain_computation::{
        publish_application_result, WorthQueryPublishedApplicationResult,
    },
};

use super::admission::BankEstateEmergencyAccessActivityAdmission;
use crate::{
    BankApplicationLiveCauseDenial, BankApplicationLiveCloseOutcome, BankApplicationLiveOverflow,
    BankApplicationLiveProjectionDenial, BankApplicationQueryDenial, BankAuthorizationDenial,
};

type ActivityLiveLease<'runtime, 'principal> = WorthQueryApplicationLiveLease<
    'runtime,
    'principal,
    BankSchema,
    EstateEmergencyAccessActivityQuery,
    EstateEmergencyAccessActivityQueryParameters,
    EstateEmergencyAccessActivity,
    Principal,
    BankPrincipalId,
    EstateCase,
    EmergencyAccess,
    EstateEmergencyAccessActivityLiveCause,
>;

pub struct BankEstateEmergencyAccessActivityLiveLease<'runtime, 'principal> {
    query: ActivityLiveLease<'runtime, 'principal>,
}

pub struct BankEstateEmergencyAccessActivityLiveUpdate {
    published: WorthQueryPublishedApplicationResult<
        EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivity,
    >,
}

#[derive(Debug)]
pub enum BankEstateEmergencyAccessActivityLiveOutcome {
    Delivered(BankEstateEmergencyAccessActivityLiveUpdate),
    Pending,
    Overflow(BankApplicationLiveOverflow),
    AuthorizationDenied(BankAuthorizationDenial),
    StalePrincipal,
    StaleScope,
    ProjectionDenied(BankApplicationLiveProjectionDenial),
    CauseDenied(BankApplicationLiveCauseDenial),
    Cancelled,
    DeadlineExceeded,
    Closed,
    Unavailable,
}

impl std::fmt::Debug for BankEstateEmergencyAccessActivityLiveUpdate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BankEstateEmergencyAccessActivityLiveUpdate")
            .field("row_count", &self.published.rows().len())
            .field("receipt", self.published.receipt())
            .finish()
    }
}

impl BankEstateEmergencyAccessActivityLiveUpdate {
    pub fn rows(&self) -> &[EstateEmergencyAccessActivity] {
        self.published.rows()
    }

    pub fn receipt(
        &self,
    ) -> &worth_query_host::facade::publication::domain_computation::WorthQueryApplicationQueryPublicationReceipt
    {
        self.published.receipt()
    }
}

impl BankEstateEmergencyAccessActivityLiveLease<'_, '_> {
    pub fn buffered_cause_count(&self) -> usize {
        self.query.buffered_cause_count()
    }

    pub fn poll(&mut self) -> BankEstateEmergencyAccessActivityLiveOutcome {
        match self.query.poll() {
            WorthQueryApplicationLiveOutcome::Delivered(update) => {
                let (_, admitted) = update.into_admitted_disclosed();
                BankEstateEmergencyAccessActivityLiveOutcome::Delivered(
                    BankEstateEmergencyAccessActivityLiveUpdate {
                        published: publish_application_result(admitted),
                    },
                )
            }
            WorthQueryApplicationLiveOutcome::Pending => {
                BankEstateEmergencyAccessActivityLiveOutcome::Pending
            }
            WorthQueryApplicationLiveOutcome::Overflow(overflow) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Overflow(
                    BankApplicationLiveOverflow::from_query(overflow),
                )
            }
            WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial) => {
                BankEstateEmergencyAccessActivityLiveOutcome::AuthorizationDenied(
                    BankAuthorizationDenial::from_query(*denial),
                )
            }
            WorthQueryApplicationLiveOutcome::StalePrincipal => {
                BankEstateEmergencyAccessActivityLiveOutcome::StalePrincipal
            }
            WorthQueryApplicationLiveOutcome::StaleScope => {
                BankEstateEmergencyAccessActivityLiveOutcome::StaleScope
            }
            WorthQueryApplicationLiveOutcome::ProjectionDenied(kind) => {
                BankEstateEmergencyAccessActivityLiveOutcome::ProjectionDenied(
                    BankApplicationLiveProjectionDenial::from_query(kind),
                )
            }
            WorthQueryApplicationLiveOutcome::CauseDenied(kind) => {
                BankEstateEmergencyAccessActivityLiveOutcome::CauseDenied(
                    BankApplicationLiveCauseDenial::from_query(kind),
                )
            }
            WorthQueryApplicationLiveOutcome::Cancelled => {
                BankEstateEmergencyAccessActivityLiveOutcome::Cancelled
            }
            WorthQueryApplicationLiveOutcome::DeadlineExceeded => {
                BankEstateEmergencyAccessActivityLiveOutcome::DeadlineExceeded
            }
            WorthQueryApplicationLiveOutcome::Closed => {
                BankEstateEmergencyAccessActivityLiveOutcome::Closed
            }
            WorthQueryApplicationLiveOutcome::Unavailable => {
                BankEstateEmergencyAccessActivityLiveOutcome::Unavailable
            }
        }
    }

    pub fn close(self) -> BankApplicationLiveCloseOutcome {
        BankApplicationLiveCloseOutcome::from_query(self.query.close())
    }
}

impl<'runtime, 'principal>
    BankEstateEmergencyAccessActivityAdmission<'runtime, 'principal, '_, '_>
{
    pub(crate) fn subscribe(
        self,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<
        BankEstateEmergencyAccessActivityLiveLease<'runtime, 'principal>,
        BankApplicationQueryDenial,
    > {
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let query_binding = application
            .installed_schema()
            .installed_query_binding::<EstateEmergencyAccessActivityQueryBinding>()
            .map_err(BankApplicationQueryDenial::from_installation)?;

        let query = query_binding.into_query();
        let capability = application
            .installed_schema()
            .capability(
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_installation)?;
        let capability_access = selected
            .admit_approved_elevation_access(
                self.approved.query(),
                self.principal.query(),
                &capability,
                self.request.capability_request(),
                controls.request(),
            )
            .map_err(BankApplicationQueryDenial::from_capability_admission)?;
        let scope = selected
            .resolve_entity(
                EstateCaseIdentityField::reference(),
                self.request.estate(),
                controls.request(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        let query = selected
            .open_governed_application_query_live::<
                EstateEmergencyAccessActivityQuery,
                EstateEmergencyAccessActivityQueryParameters,
                EstateEmergencyAccessActivity,
                Principal,
                BankPrincipalId,
                EstateCase,
                EmergencyAccess,
                EstateEmergencyAccessActivityLiveCause,
                _,
                _,
                _,
            >(
                query,
                self.principal.query(),
                scope,
                capability_access,
                ApplicationQueryParameterSet::<EstateEmergencyAccessActivityQuery>::new(),
                controls,
            )
            .map_err(BankApplicationQueryDenial::from_live_open)?;
        Ok(BankEstateEmergencyAccessActivityLiveLease { query })
    }
}
