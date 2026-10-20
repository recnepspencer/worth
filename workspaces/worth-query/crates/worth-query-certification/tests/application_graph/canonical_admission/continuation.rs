//! Actual authenticated continuation pages refuse swallowed stops before handoff.
#[path = "continuation/query.rs"]
pub(super) mod query;

use super::{host, opaque, schema::*};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial},
    primary_graph::*,
};

type Page = WorthQueryApplicationContinuationPageResult<
    AdmissionSchema,
    query::PageQuery,
    super::query::Parameters,
    query::PageRow,
    Document,
>;

fn first_page(
    host: &host::Host,
    scope: &WorthQueryRequestScope,
) -> Result<Page, WorthQueryApplicationRequestQueryDenial> {
    let principal = authenticate_operator(host.installed_schema(), scope);
    host.runtime()
        .request(&principal, scope)
        .on_branch(host.current_world())
        .query(query::ReadPage {
            identity: "root".to_owned(),
        })
        .page(NonZeroUsize::MIN, Some(NonZeroUsize::new(5_120).unwrap()))
}

fn assert_funded_pages(host: &host::Host) {
    query::reset(None, false);
    opaque::reset();
    let scope = request_scope();
    let page = first_page(host, &scope).unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(
        page.rows()[0].children,
        [("child".to_owned(), super::value())]
    );
    assert_eq!(query::returned(), 1);
    assert_eq!(opaque::ignored_refusals(), 0);
    let (_, token, receipt) = page.into_parts();
    assert!(receipt.basis_released());
    let token = token.expect("a real second ordered child must issue a native token");
    assert_eq!(token.page_ordinal(), 2);

    // A continuation requires a fresh authenticated request, preserving its
    // exact installed ordered native basis rather than rerunning the first page.
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let page = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .query(query::ReadPage {
            identity: "root".to_owned(),
        })
        .resume(
            token,
            NonZeroUsize::MIN,
            Some(NonZeroUsize::new(5_120).unwrap()),
        )
        .unwrap();
    assert_eq!(page.rows().len(), 1);
    assert_eq!(
        page.rows()[0].children,
        [("root".to_owned(), super::value())]
    );
    assert_eq!(query::returned(), 2);
    assert_eq!(opaque::ignored_refusals(), 0);
    assert!(page.continuation().is_none());
    assert!(page.receipt().basis_released());
}

#[test]
fn continuation_nested_decode_refusal_never_returns_a_page_or_successor() {
    let host = host::install_page(super::FUNDED_WORK);
    assert_funded_pages(&host);
    drop(host);

    let host = host::install_page(super::ONE_VALUE_WORK);
    query::reset(None, false);
    opaque::reset();
    let denial = match first_page(&host, &request_scope()) {
        Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
        Err(other) => panic!("the admitted page must fail at decode completion: {other:?}"),
        Ok(_) => panic!("swallowed decode refusal must return neither page nor successor"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationContinuationDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::ValueDecode(
                WorthQueryValueDecodeDenial::Checkpoint(
                    worth_execution::MapKernelStop::WorkCeiling
                )
            )
        )
    );
    assert_eq!(
        query::returned(),
        1,
        "the actual projector returned apparent success"
    );
    assert!(
        opaque::ignored_refusals() > 0,
        "the real nested seed swallowed native refusal"
    );
    let checkpoint = host::capture(&host);
    drop(host);
    let fresh = host::restore(checkpoint, super::FUNDED_WORK);
    assert_funded_pages(&fresh);
}

#[test]
fn continuation_late_cancellation_refuses_the_page_and_fresh_request_succeeds() {
    let host = host::install_page(super::FUNDED_WORK);
    // Both apparent success and a domain rejection must preserve cancellation
    // of the ORIGINAL request after the actual nondefault value was decoded.
    for reject in [false, true] {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(30)),
            cancellation.token(),
        );
        query::reset(Some(cancellation), reject);
        opaque::reset();
        let denial = match first_page(&host, &scope) {
            Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
            Err(other) => panic!("late cancellation must veto the admitted page: {other:?}"),
            Ok(_) => panic!("late cancellation must return neither page nor successor"),
        };
        assert_eq!(
            query::returned(),
            1,
            "cancellation follows actual decoded projection"
        );
        assert_eq!(opaque::ignored_refusals(), 0);
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationContinuationDenialKind::Projection(
                WorthQueryApplicationProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            )
        );
    }
    // With a live original request, an actual application domain rejection
    // still propagates as such; cancellation is not a manufactured error map.
    query::reset(None, true);
    let denial = match first_page(&host, &request_scope()) {
        Err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution(denial)) => denial,
        Err(other) => panic!("the domain rejection must remain a projection denial: {other:?}"),
        Ok(_) => panic!("a rejecting projector must not publish a page or successor"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationContinuationDenialKind::Projection(
            WorthQueryApplicationProjectionDenialKind::DomainProjectionRejected,
        )
    );
    assert_funded_pages(&host);
}
