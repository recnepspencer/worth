//! Real captured source and ordinary corrections produce every native page used here.
use super::*;
use crate::http::server::mutation_custody::{
    CorrectionCustody, ReapplicationCustody, SendMoneyCustody, SourceMutationCustody,
};
use crate::http::server::tests::fixture::{application, CausalHttpApplication};
use bank_domain::{
    model::{AccountId, BankPrincipalId, Money},
    proposals::BankIdempotencyKey,
    schema::SendMoney,
};
use bank_server::{BankAuthenticatedPrincipal, BankMutationControls};
use worth_query_host::facade::{application_entry::*, primary_graph::*};

pub(super) struct Fixture {
    pub application: CausalHttpApplication,
    pub principal: BankAuthenticatedPrincipal,
    pub source: BankHttpCorrectionSource,
    pub page: Arc<WorthQueryCorrectionAlternativesPage>,
}
pub(super) fn scope(
) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope {
    use worth_query_host::facade::admission::authenticated_principal::*;
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(30)),
        WorthQueryCancellationSource::new().token(),
    )
}
pub(super) fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(
        std::num::NonZeroUsize::new(128),
        std::num::NonZeroUsize::new(16).unwrap(),
    )
}
pub(super) async fn fixture() -> Fixture {
    build(false).await.0
}
/// Each retained completion owns its own World read lease throughout the journey.
pub(super) async fn retained_fixture() -> (Fixture, Vec<SourceMutationCustody>) {
    build(true).await
}
pub(super) fn close_owners(
    application: &CausalHttpApplication,
    principal: &BankAuthenticatedPrincipal,
    owners: Vec<SourceMutationCustody>,
) {
    let request_scope = scope();
    let request = application.runtime.request(principal, &request_scope);
    for owner in owners {
        assert!(owner.close(&request).is_ok());
    }
}
async fn build(retain: bool) -> (Fixture, Vec<SourceMutationCustody>) {
    let application = application(AccountId::new(100).unwrap());
    let request_scope = scope();
    let principal = application.authenticate(&request_scope).await;
    let mut owners = Vec::new();
    let (source, page) = {
        let request = application.runtime.request(&principal, &request_scope);
        let operation = application
            .runtime
            .mutate(bank_server::mutations::send_money(SendMoney {
                from: AccountId::new(100).unwrap(),
                recipient: BankPrincipalId::new(2).unwrap(),
                amount: Money::from_minor(10).unwrap(),
            }))
            .as_principal(&principal)
            .controls(BankMutationControls::new(
                request_scope.clone(),
                BankIdempotencyKey::new("registry-source").unwrap(),
            ))
            .retain_operation()
            .unwrap_or_else(|_| panic!("source owner"));
        let WorthQuerySourceOnlyAdvance::Completed(completed) = operation.advance(&request, None)
        else {
            panic!("source completion");
        };
        let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
            panic!("fresh source");
        };
        let source = receipt.correction_source().coordinates();
        retain_or_close(
            SourceMutationCustody::SendMoney(SendMoneyCustody::Completed(completed)),
            retain,
            &request,
            &mut owners,
        );
        retain_or_close(
            reverse(&application, &principal, &source, "registry-u"),
            retain,
            &request,
            &mut owners,
        );
        let selected = request
            .corrections()
            .select_coordinates(&source, limits())
            .unwrap();
        let page = request
            .corrections()
            .alternatives(&selected, limits(), std::num::NonZeroUsize::MIN)
            .unwrap();
        let alternative = page.select(&page.entries()[0], limits()).unwrap();
        let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = application
            .runtime
            .prepare_send_money_reapplication(
                &principal,
                &request_scope,
                &alternative,
                "registry-r",
                limits(),
            )
            .unwrap()
        else {
            panic!("fresh reapplication");
        };
        let completed = match prepared.publish().unwrap() {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            WorthQuerySourceOnlyAdvance::Progress(owner) => match owner.advance(&request, None) {
                WorthQuerySourceOnlyAdvance::Completed(done) => done,
                _ => panic!("reapplication delivery"),
            },
            _ => panic!("reapplication publication"),
        };
        let WorthQuerySourceCompletion::Fresh { receipt, .. } = completed.evidence() else {
            panic!("fresh reapplication receipt");
        };
        let reapplied = receipt.correction_source().coordinates();
        retain_or_close(
            SourceMutationCustody::Reapplication(ReapplicationCustody::Completed(completed)),
            retain,
            &request,
            &mut owners,
        );
        retain_or_close(
            reverse(&application, &principal, &reapplied, "registry-v"),
            retain,
            &request,
            &mut owners,
        );
        let selected = request
            .corrections()
            .select_coordinates(&source, limits())
            .unwrap();
        let page = Arc::new(
            request
                .corrections()
                .alternatives(&selected, limits(), std::num::NonZeroUsize::MIN)
                .unwrap()
                .into_page(),
        );
        assert!(page.continuation().is_some());
        (source, page)
    };
    (
        Fixture {
            application,
            principal,
            source: source.into(),
            page,
        },
        owners,
    )
}
fn retain_or_close(
    owner: SourceMutationCustody,
    retain: bool,
    request: &WorthQueryApplicationRequest<'_, '_, '_, bank_domain::schema::BankSchema>,
    owners: &mut Vec<SourceMutationCustody>,
) {
    if retain {
        owners.push(owner);
    } else {
        assert!(owner.close(request).is_ok());
    }
}
fn reverse(
    application: &CausalHttpApplication,
    principal: &BankAuthenticatedPrincipal,
    source: &WorthQueryCorrectionSourceCoordinates,
    key: &str,
) -> SourceMutationCustody {
    let request_scope = scope();
    let request = application.runtime.request(principal, &request_scope);
    let WorthQueryOperationCorrectionPreparation::Prepared(prepared) = application
        .runtime
        .prepare_send_money_correction(principal, &request_scope, source, key, limits())
        .unwrap()
    else {
        panic!("fresh inverse");
    };
    let completed = match prepared.publish().unwrap() {
        WorthQuerySourceOnlyAdvance::Completed(done) => done,
        WorthQuerySourceOnlyAdvance::Progress(owner) => match owner.advance(&request, None) {
            WorthQuerySourceOnlyAdvance::Completed(done) => done,
            _ => panic!("inverse delivery"),
        },
        _ => panic!("inverse publication"),
    };
    SourceMutationCustody::Correction(CorrectionCustody::Completed(completed))
}
