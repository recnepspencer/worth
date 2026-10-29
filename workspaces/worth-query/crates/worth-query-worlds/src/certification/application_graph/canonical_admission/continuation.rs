//! Actual authenticated continuation pages refuse swallowed stops before handoff.
#[path = "continuation/query.rs"]
pub mod query;

pub use super::{host, opaque, schema::*};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
pub use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenial},
    primary_graph::*,
};

pub type Page = WorthQueryApplicationContinuationPageResult<
    AdmissionSchema,
    query::PageQuery,
    super::query::Parameters,
    query::PageRow,
    Document,
>;

pub fn first_page(
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
        .page(NonZeroUsize::MIN)
}

pub fn assert_funded_pages(host: &host::Host) {
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
        .resume(token, NonZeroUsize::MIN)
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
