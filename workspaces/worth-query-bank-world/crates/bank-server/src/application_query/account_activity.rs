use std::num::NonZeroUsize;

use bank_domain::model::{AccountId, BankPrincipalId};
use bank_domain::queries::{
    account_activity, AccountActivityLiveCause, AccountActivityQuery, AccountActivityQueryBinding,
    AccountActivityQueryParameters, AccountActivityQueryResult,
};
use bank_domain::schema::{Account, AccountIdentity, BankSchema, Posting, Principal};
use worth_query_host::facade::{
    application_entry::WorthQueryApplicationRequestExt,
    declaration::application_query::ApplicationQueryParameterSet,
    domain::WorthQueryInstalledApplicationQuery,
    primary_graph::{
        WorthQueryApplicationEntityIdentity, WorthQueryApplicationLiveControls,
        WorthQueryApplicationLiveLease, WorthQueryApplicationQueryAccessContext,
        WorthQueryApplicationQueryResumeControls, WorthQueryPrincipalResolutionMode,
        WorthQueryProductQueryControls, WorthQuerySelectedProductOperation,
    },
    publication::domain_computation::publish_application_result,
};

mod output;

pub use output::{
    BankAccountActivityContinuation, BankAccountActivityHistoricalResult,
    BankAccountActivityLiveOutcome, BankAccountActivityLiveUpdate, BankAccountActivityPageResult,
    BankAccountActivityQueryResult,
};

use super::BankApplicationQueryDenial;
use crate::{
    BankApplicationLiveCloseOutcome, BankAuthenticatedPrincipal, BankCommitReceipt,
    BankIdentityRuntime, BankReadControls,
};

type QueryAccountActivityLiveLease<'runtime, 'principal> = WorthQueryApplicationLiveLease<
    'runtime,
    'principal,
    BankSchema,
    AccountActivityQuery,
    AccountActivityQueryParameters,
    AccountActivityQueryResult,
    Principal,
    BankPrincipalId,
    Account,
    Posting,
    AccountActivityLiveCause,
>;

pub struct BankAccountActivityRequest<'runtime> {
    runtime: &'runtime BankIdentityRuntime,
    account: AccountId,
}

pub struct BankAccountActivityRequestForPrincipal<'runtime, 'principal> {
    runtime: &'runtime BankIdentityRuntime,
    principal: &'principal BankAuthenticatedPrincipal,
    account: AccountId,
}

pub struct BankAccountActivityLiveLease<'runtime, 'principal> {
    query: QueryAccountActivityLiveLease<'runtime, 'principal>,
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
        commit: &BankCommitReceipt,
        maximum_result_count: NonZeroUsize,
        maximum_work: NonZeroUsize,
        request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<BankAccountActivityHistoricalResult, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let history = application
            .branches()
            .history(application.current_world(), maximum_work)
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let selected_commit = commit.publication().product_commit().composite_commit();
        let entry = history
            .entries()
            .find(|entry| entry.selected_commit() == selected_commit)
            .ok_or(BankApplicationQueryDenial::HistoricalCommitUnavailable)?;
        let selected = history
            .select(&entry)
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let prepared = self.prepare(selected, request)?;
        let PreparedAccountActivity {
            runtime,
            principal,
            query,
            scope,
            selected,
        } = prepared;
        let access = WorthQueryApplicationQueryAccessContext::new(principal.query(), &scope);
        let plan = selected
            .admit_application_query(
                &query,
                &access,
                ApplicationQueryParameterSet::<AccountActivityQuery>::new(),
                WorthQueryProductQueryControls::new(maximum_result_count, maximum_work, request),
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        let result = runtime
            .application_runtime()
            .execute_application_query_one_shot(plan)
            .map_err(BankApplicationQueryDenial::from_execution)?;
        Ok(publish_application_result(result.into_admitted_disclosed()))
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
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let prepared = self.prepare(selected, controls.request())?;
        let PreparedAccountActivity {
            runtime,
            principal,
            query,
            scope,
            selected,
        } = prepared;
        let access = WorthQueryApplicationQueryAccessContext::new(principal.query(), &scope);
        let plan = selected
            .admit_application_query_continuation(
                &query,
                &access,
                ApplicationQueryParameterSet::<AccountActivityQuery>::new(),
                controls.application_query_controls(),
            )
            .map_err(BankApplicationQueryDenial::from_admission)?;
        let page = runtime
            .application_runtime()
            .execute_application_query_continuation_page(plan)
            .map_err(BankApplicationQueryDenial::from_continuation_execution)?;
        Ok(output::publish_page(page))
    }

    pub fn resume(
        self,
        continuation: BankAccountActivityContinuation,
        controls: WorthQueryApplicationQueryResumeControls<'_>,
    ) -> Result<BankAccountActivityPageResult, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let prepared = self.prepare(selected, controls.request_scope())?;
        continuation.resume(prepared, controls)
    }

    pub fn subscribe(
        self,
        controls: WorthQueryApplicationLiveControls,
    ) -> Result<BankAccountActivityLiveLease<'runtime, 'principal>, BankApplicationQueryDenial>
    {
        let application = self.runtime.application_runtime();
        let selected = application
            .on_branch(application.current_world())
            .select()
            .map_err(BankApplicationQueryDenial::from_product_selection)?;
        let prepared = self.prepare(selected, controls.request())?;
        let PreparedAccountActivity {
            principal,
            query,
            scope,
            selected,
            ..
        } = prepared;
        let query = selected
            .open_application_query_live::<
                AccountActivityQuery,
                AccountActivityQueryParameters,
                AccountActivityQueryResult,
                Principal,
                BankPrincipalId,
                Account,
                Posting,
                AccountActivityLiveCause,
            >(
                query,
                principal.query(),
                scope,
                ApplicationQueryParameterSet::<AccountActivityQuery>::new(),
                controls,
            )
            .map_err(BankApplicationQueryDenial::from_live_open)?;
        Ok(BankAccountActivityLiveLease { query })
    }

    fn prepare(
        self,
        selected: WorthQuerySelectedProductOperation<'runtime, BankSchema>,
        request: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<PreparedAccountActivity<'runtime, 'principal>, BankApplicationQueryDenial> {
        let application = self.runtime.application_runtime();
        let query_binding = application
            .installed_schema()
            .installed_query_binding::<AccountActivityQueryBinding>()
            .map_err(BankApplicationQueryDenial::from_installation)?;

        let query = query_binding.into_query();
        let scope = selected
            .resolve_entity(
                AccountIdentity::reference(),
                self.account,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(BankApplicationQueryDenial::from_scope_resolution)?;
        Ok(PreparedAccountActivity {
            runtime: self.runtime,
            principal: self.principal,
            query,
            scope,
            selected,
        })
    }
}

impl BankAccountActivityLiveLease<'_, '_> {
    pub fn buffered_cause_count(&self) -> usize {
        self.query.buffered_cause_count()
    }

    pub fn poll(&mut self) -> BankAccountActivityLiveOutcome {
        output::publish_live_outcome(self.query.poll())
    }

    pub fn close(self) -> BankApplicationLiveCloseOutcome {
        output::publish_close(self.query.close())
    }
}

struct PreparedAccountActivity<'runtime, 'principal> {
    runtime: &'runtime BankIdentityRuntime,
    principal: &'principal BankAuthenticatedPrincipal,
    query: WorthQueryInstalledApplicationQuery<
        BankSchema,
        AccountActivityQuery,
        AccountActivityQueryParameters,
        AccountActivityQueryResult,
        Account,
    >,
    scope: WorthQueryApplicationEntityIdentity<BankSchema, Account>,
    selected: WorthQuerySelectedProductOperation<'runtime, BankSchema>,
}

impl PreparedAccountActivity<'_, '_> {
    fn access(
        &self,
    ) -> WorthQueryApplicationQueryAccessContext<'_, BankSchema, Principal, BankPrincipalId, Account>
    {
        WorthQueryApplicationQueryAccessContext::new(self.principal.query(), &self.scope)
    }
}
