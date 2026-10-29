//! Real Decl/Host expectations; no caller supplies physical dependency facts.
use super::super::authentication;
use super::{host, mutation, prefix, query, schema::Catalog};
use std::sync::Arc;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
mod affinity;
mod membership;
fn run(
    test: fn(
        &host::Host,
        &WorthQueryAuthenticatedExternalPrincipal<Catalog>,
        &WorthQueryRequestScope,
    ),
) {
    std::thread::Builder::new()
        .name("managed-source-expectations".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            let world = host::install(Arc::new(prefix::Calls::default()));
            let scope = authentication::request_scope();
            let adapter = authentication::admit(world.installed_schema());
            let principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ))
            .unwrap();
            test(&world, &principal, &scope);
        })
        .unwrap()
        .join()
        .unwrap();
}
fn find(prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: "catalog".into(),
        prefix: prefix.into(),
    }
}
fn rename(prefix: &str, item: &str, replacement: &str) -> mutation::Rename {
    mutation::Rename {
        directory: "catalog".into(),
        prefix: prefix.into(),
        item: item.into(),
        replacement: replacement.into(),
    }
}
fn source_changed(
    result: Result<
        WorthQueryApplicationMutationOutcome<mutation::Denial, mutation::Renamed>,
        WorthQueryApplicationRequestMutationDenial,
    >,
) {
    let cause = result.expect_err("stale complete source refuses before effects");
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Attempt(cause),
    ) = cause
    else {
        panic!("actual stale-source attempt: {cause:?}")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryApplicationAttemptDenialKind::SourceChanged
    );
}
