use std::num::NonZeroUsize;

use bank_domain::model::AccountId;
use bank_domain::queries::{account_activity, AccountActivityRequest};
use bank_domain::schema::BankSchema;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationLiveLimits, WorthQueryApplicationLiveSubscription,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::{WorthQueryApplicationLiveControls, WorthQueryApplicationQueryResumeControls},
};

mod output;

pub use output::{
    BankAccountActivityContinuation, BankAccountActivityHistoricalResult,
    BankAccountActivityLiveOutcome, BankAccountActivityLiveUpdate, BankAccountActivityPageResult,
    BankAccountActivityQueryResult,
};

use super::BankApplicationQueryDenial;
use crate::{
    BankApplicationLiveCloseOutcome, BankAuthenticatedPrincipal, BankIdentityRuntime,
    BankReadControls,
};

type QueryAccountActivityLiveLease<'runtime> =
    WorthQueryApplicationLiveSubscription<'runtime, BankSchema, AccountActivityRequest>;

/// Reusable query selection; principal, product and request scope are supplied on each use.
#[derive(Clone, Copy)]
pub struct BankAccountActivityRequest<'runtime> {
    runtime: &'runtime BankIdentityRuntime,
    account: AccountId,
}

pub struct BankAccountActivityRequestForPrincipal<'runtime, 'principal> {
    runtime: &'runtime BankIdentityRuntime,
    principal: &'principal BankAuthenticatedPrincipal,
    account: AccountId,
}

pub struct BankAccountActivityLiveLease<'runtime> {
    runtime: &'runtime BankIdentityRuntime,
    query: QueryAccountActivityLiveLease<'runtime>,
}

impl BankIdentityRuntime {
    pub const fn account_activity(&self, account: AccountId) -> BankAccountActivityRequest<'_> {
        BankAccountActivityRequest {
            runtime: self,
            account,
        }
    }
}

impl<'runtime> BankAccountActivityRequest<'runtime> {
    pub const fn as_principal<'principal>(
        self,
        principal: &'principal BankAuthenticatedPrincipal,
    ) -> BankAccountActivityRequestForPrincipal<'runtime, 'principal> {
        BankAccountActivityRequestForPrincipal {
            runtime: self.runtime,
            principal,
            account: self.account,
        }
    }
}

impl<'runtime, 'principal> BankAccountActivityRequestForPrincipal<'runtime, 'principal> {
    pub fn historical(
        self,
        commit: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<BankAccountActivityHistoricalResult, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let ordinary = application.request(self.principal.external(), request);
        let selected = ordinary
            .at_commit(commit, maximum_work)
            .map_err(BankApplicationQueryDenial::from_history_selection)?;
        selected
            .query(account_activity(self.account))
            .limits(maximum_result_count, maximum_work)
            .execute()
            .map_err(BankApplicationQueryDenial::from_request_query)
    }

    pub fn retained(
        self,
        observation: &worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation,
        controls: BankReadControls,
    ) -> Result<BankAccountActivityQueryResult, BankApplicationQueryDenial> {
        self.runtime
            .application_runtime()
            .request(self.principal.external(), controls.request())
            .at(observation)
            .query(account_activity(self.account))
            .limits(controls.maximum_result_count(), controls.maximum_work())
            .execute()
            .map_err(BankApplicationQueryDenial::from_request_query)
    }

    pub fn execute(
        self,
        controls: BankReadControls,
    ) -> Result<BankAccountActivityQueryResult, BankApplicationQueryDenial> {
        let result = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), controls.request())
            .query(account_activity(self.account))
            .limits(controls.maximum_result_count(), controls.maximum_work())
            .execute()
            .map_err(BankApplicationQueryDenial::from_request_query)?;
        Ok(result)
    }

    pub fn page(
        self,
        controls: BankReadControls,
    ) -> Result<BankAccountActivityPageResult, BankApplicationQueryDenial> {
        let page = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), controls.request())
            .query(account_activity(self.account))
            .page(controls.maximum_result_count(), controls.maximum_work())
            .map_err(BankApplicationQueryDenial::from_request_query)?;
        Ok(output::publish_page(page))
    }

    pub fn resume(
        self,
        continuation: BankAccountActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
    ) -> Result<BankAccountActivityPageResult, BankApplicationQueryDenial> {
        let page = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), controls.request_scope())
            .query(account_activity(self.account))
            .resume(
                continuation.into_query(),
                controls.maximum_page_width(),
                controls.maximum_work(),
            )
            .map_err(BankApplicationQueryDenial::from_request_query)?;
        Ok(output::publish_page(page))
    }

    pub fn subscribe(
        self,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<BankAccountActivityLiveLease<'runtime>, BankApplicationQueryDenial> {
        let limits = WorthQueryApplicationLiveLimits::bounded(
            controls.buffer_capacity(),
            controls.maximum_materialized_record_count().get(),
            controls.maximum_work_per_delivery().get(),
        );
        let request = self
            .runtime
            .application_runtime()
            .request(self.principal.external(), controls.request());
        let query = request
            .query(account_activity(self.account))
            .subscribe(limits)
            .map_err(BankApplicationQueryDenial::from_live_request_open)?;
        Ok(BankAccountActivityLiveLease {
            runtime: self.runtime,
            query,
        })
    }
}

impl BankAccountActivityLiveLease<'_> {
    pub fn buffered_cause_count(&self) -> usize {
        self.query.buffered_cause_count()
    }

    pub fn poll(
        &mut self,
        principal: &BankAuthenticatedPrincipal,
        request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> BankAccountActivityLiveOutcome {
        let fresh = self
            .runtime
            .application_runtime()
            .request(principal.external(), request);
        match self.query.next(&fresh) {
            Ok(outcome) => output::publish_live_outcome(outcome),
            Err(_) => BankAccountActivityLiveOutcome::Unavailable,
        }
    }

    pub fn close(self) -> BankApplicationLiveCloseOutcome {
        output::publish_close(self.query.close())
    }
}
