use bank_domain::{
    queries::{
        EstateEmergencyAccessActivity, EstateEmergencyAccessActivityQuery,
        EstateEmergencyAccessActivityRequest,
    },
    schema::{BankSchema, ViewEstateEmergencyProtectionCapability, ViewRestrictedEstateOperation},
};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveSubscription,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{WorthQueryApplicationLiveControls, WorthQueryApplicationLiveOutcome},
    publication::domain_computation::{
        publish_application_result, WorthQueryPublishedApplicationResult,
    },
};

use super::admission::BankEstateEmergencyAccessActivityAdmission;
use crate::{
    BankApplicationLiveCauseDenial, BankApplicationLiveCloseOutcome, BankApplicationLiveOverflow,
    BankApplicationLiveProjectionDenial, BankApplicationQueryDenial, BankAuthenticatedPrincipal,
    BankAuthorizationDenial, BankIdentityRuntime,
};

type ActivityLiveLease<'runtime> = WorthQueryApplicationLiveSubscription<
    'runtime,
    BankSchema,
    EstateEmergencyAccessActivityRequest,
>;

pub struct BankEstateEmergencyAccessActivityLiveLease<'runtime> {
    runtime: &'runtime BankIdentityRuntime,
    query: ActivityLiveLease<'runtime>,
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

impl BankEstateEmergencyAccessActivityLiveLease<'_> {
    pub fn buffered_cause_count(&self) -> usize {
        self.query.buffered_cause_count()
    }

    pub fn poll(
        &mut self,
        principal: &BankAuthenticatedPrincipal,
        request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> BankEstateEmergencyAccessActivityLiveOutcome {
        let fresh = self
            .runtime
            .application_runtime()
            .request(principal.external(), request);
        match self.query.next(&fresh) {
            Ok(WorthQueryApplicationLiveOutcome::Delivered(update)) => {
                let (_, admitted) = update.into_admitted_disclosed();
                BankEstateEmergencyAccessActivityLiveOutcome::Delivered(
                    BankEstateEmergencyAccessActivityLiveUpdate {
                        published: publish_application_result(admitted),
                    },
                )
            }
            Ok(WorthQueryApplicationLiveOutcome::Pending) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Pending
            }
            Ok(WorthQueryApplicationLiveOutcome::Overflow(overflow)) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Overflow(
                    BankApplicationLiveOverflow::from_query(overflow),
                )
            }
            Ok(WorthQueryApplicationLiveOutcome::AuthorizationDenied(denial)) => {
                BankEstateEmergencyAccessActivityLiveOutcome::AuthorizationDenied(
                    BankAuthorizationDenial::from_query(*denial),
                )
            }
            Ok(WorthQueryApplicationLiveOutcome::StalePrincipal) => {
                BankEstateEmergencyAccessActivityLiveOutcome::StalePrincipal
            }
            Ok(WorthQueryApplicationLiveOutcome::StaleScope) => {
                BankEstateEmergencyAccessActivityLiveOutcome::StaleScope
            }
            Ok(WorthQueryApplicationLiveOutcome::ProjectionDenied(kind)) => {
                BankEstateEmergencyAccessActivityLiveOutcome::ProjectionDenied(
                    BankApplicationLiveProjectionDenial::from_query(kind),
                )
            }
            Ok(WorthQueryApplicationLiveOutcome::CauseDenied(kind)) => {
                BankEstateEmergencyAccessActivityLiveOutcome::CauseDenied(
                    BankApplicationLiveCauseDenial::from_query(kind),
                )
            }
            Ok(WorthQueryApplicationLiveOutcome::Cancelled) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Cancelled
            }
            Ok(WorthQueryApplicationLiveOutcome::DeadlineExceeded) => {
                BankEstateEmergencyAccessActivityLiveOutcome::DeadlineExceeded
            }
            Ok(WorthQueryApplicationLiveOutcome::Closed) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Closed
            }
            Ok(WorthQueryApplicationLiveOutcome::Unavailable) => {
                BankEstateEmergencyAccessActivityLiveOutcome::Unavailable
            }
            Err(_) => BankEstateEmergencyAccessActivityLiveOutcome::Unavailable,
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
    ) -> Result<BankEstateEmergencyAccessActivityLiveLease<'runtime>, BankApplicationQueryDenial>
    {
        let limits = WorthQueryApplicationLiveLimits::bounded(
            controls.buffer_capacity(),
            controls.maximum_materialized_record_count().get(),
            controls.maximum_work_per_delivery().get(),
        );
        let input = self.request.capability_request();
        let query = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), controls.request())
            .query(self.request)
            .subscribe_approved(
                self.approved.query(),
                ViewEstateEmergencyProtectionCapability::reference(),
                ViewRestrictedEstateOperation::reference(),
                input,
                limits,
            )
            .map_err(BankApplicationQueryDenial::from_live_request_open)?;
        Ok(BankEstateEmergencyAccessActivityLiveLease {
            runtime: self.runtime,
            query,
        })
    }
}
