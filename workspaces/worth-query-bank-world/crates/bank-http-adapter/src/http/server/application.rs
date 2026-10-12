use std::time::Instant;

use bank_domain::model::AccountId;
use bank_domain::queries;
use bank_server::{BankReadControlDenial, BankReadControls};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationToken, WorthQueryRequestScope,
};

use super::super::protocol::{
    BankHttpAccountSummary, BankHttpAccountSummaryOutcome, BankHttpCredential, BankHttpDenial,
    BankHttpDenialKind, BankHttpNextAction, BankHttpQueryCapabilityPurpose,
    BankHttpRequestControls,
};
use super::authentication::BankHttpApplicationAuthenticator;
use super::query_denial::query_denial;
use super::query_publication::describe_query_publication;

pub(super) struct AdmittedAccountSummaryRequest {
    pub(super) request_id: String,
    pub(super) credential: BankHttpCredential,
    pub(super) controls: BankHttpRequestControls,
    pub(super) account: AccountId,
    pub(super) deadline: Instant,
}

pub(super) async fn execute_account_summary<A>(
    application: &A,
    request: AdmittedAccountSummaryRequest,
    cancellation: WorthQueryCancellationToken,
) -> BankHttpAccountSummaryOutcome
where
    A: BankHttpApplicationAuthenticator,
{
    let request_id = request.request_id;
    let scope = WorthQueryRequestScope::new(Some(request.deadline), cancellation);
    let principal = match application.authenticate(request.credential, &scope).await {
        Ok(principal) => principal,
        Err(denial) => return denied(Some(request_id), denial),
    };
    let controls = match BankReadControls::current(
        scope,
        request.controls.maximum_results,
        request.controls.maximum_work,
    ) {
        Ok(controls) => controls,
        Err(denial) => return denied(Some(request_id), control_denial(denial)),
    };
    let published = match application
        .runtime()
        .query(queries::account_summary(request.account))
        .as_principal(&principal)
        .controls(controls)
        .execute()
    {
        Ok(published) => published,
        Err(denial) => return denied(Some(request_id), query_denial(denial)),
    };
    let publication = describe_query_publication(
        published.receipt(),
        BankHttpQueryCapabilityPurpose::AccountServicing,
    );
    let mut rows = published.into_rows();
    if rows.len() != 1 {
        return denied(
            Some(request_id),
            BankHttpDenial::new(BankHttpDenialKind::InternalDenied, BankHttpNextAction::None),
        );
    }
    BankHttpAccountSummaryOutcome::Delivered {
        request_id,
        summary: BankHttpAccountSummary::from(&rows.remove(0)),
        publication,
    }
}

pub(super) fn denied(
    request_id: Option<String>,
    denial: BankHttpDenial,
) -> BankHttpAccountSummaryOutcome {
    BankHttpAccountSummaryOutcome::Denied { request_id, denial }
}

fn control_denial(denial: BankReadControlDenial) -> BankHttpDenial {
    match denial {
        BankReadControlDenial::ZeroResultLimit | BankReadControlDenial::ZeroWorkLimit => {
            BankHttpDenial::new(
                BankHttpDenialKind::MalformedRequest,
                BankHttpNextAction::CorrectRequest,
            )
        }
        BankReadControlDenial::ResultLimitTooLarge { .. } => BankHttpDenial::new(
            BankHttpDenialKind::ResourceExhausted,
            BankHttpNextAction::NarrowRequest,
        ),
    }
}
