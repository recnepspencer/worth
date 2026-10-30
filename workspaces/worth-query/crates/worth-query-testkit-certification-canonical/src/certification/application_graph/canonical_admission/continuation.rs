//! Actual authenticated continuation pages refuse swallowed stops before handoff.
#[path = "continuation/query.rs"]
pub mod query;

pub use super::{host, opaque, schema::*};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
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
    let runtime = host.program.runtime();
    let external = authenticate_operator(host.program.installed_schema(), scope);
    super::request_funding::with_request(host, scope, |phase| {
        let binding = runtime
            .installed_schema()
            .installed_query_binding::<query::PageBinding>()
            .map_err(WorthQueryApplicationRequestQueryDenial::BindingInstallation)?;
        let selected = runtime
            .on_branch(host.program.current_world())
            .select(&phase)
            .map_err(WorthQueryApplicationRequestQueryDenial::ProductSelection)?;
        let principal = selected
            .resolve_authenticated_principal(
                &phase,
                binding.principal_binding(),
                &external,
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution)?;
        let root = selected
            .resolve_entity(
                &phase,
                DocumentIdentityField::reference(),
                "root".to_owned(),
                scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::ScopeResolution)?;
        let query = binding.into_query();
        let access = WorthQueryApplicationQueryAccessContext::new(&principal, &root);
        let controls = WorthQueryProductQueryControls::new(NonZeroUsize::MIN, scope);
        let plan = selected
            .admit_application_query_continuation(
                &phase,
                &query,
                &access,
                worth_query_host::facade::declaration::application_query::ApplicationQueryParameterSet::new(),
                controls,
            )
            .map_err(WorthQueryApplicationRequestQueryDenial::Admission)?;
        runtime
            .execute_application_query_continuation_page(&phase, plan)
            .map_err(WorthQueryApplicationRequestQueryDenial::ContinuationExecution)
    })
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
    let principal = authenticate_operator(host.program.installed_schema(), &scope);
    let page = host
        .program
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.program.current_world())
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
