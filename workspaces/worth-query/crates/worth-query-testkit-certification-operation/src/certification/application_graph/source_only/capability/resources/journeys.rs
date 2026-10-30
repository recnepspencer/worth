//! Real two-facade resource requests, including independent provider observations.
pub use super::super::{grant_commands, host, program::*};
pub use super::{metadata, Resource};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope;
pub use std::sync::atomic::Ordering;
pub use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
pub use worth_query_host::facade::{application_entry::*, primary_graph::*};
#[path = "journeys/action_authorization.rs"]
pub mod action_authorization;
#[path = "journeys/control.rs"]
pub mod control;
#[path = "journeys/scope_replacement.rs"]
pub mod scope_replacement;
#[path = "journeys/source_binding.rs"]
pub mod source_binding;

pub fn published() -> host::Host {
    let host = host::install();
    let scope = request_scope();
    let actor = host::authenticate(&host, 1, &scope);
    let request = host.application.runtime().request(&actor, &scope);
    let retained = request
        .mutate(Original {
            input: SetRetentionInput {
                identity: DOCUMENT_IDENTITY.to_owned(),
                retention_days: 8,
            },
        })
        .without_source()
        .idempotency_owned(0x9181_8001_u64)
        .unwrap_or_else(|(_, _, cause)| panic!("source: {cause:?}"))
        .retain_operation::<Program, Action>(&host.application)
        .unwrap_or_else(|(_, cause)| panic!("source owner: {cause:?}"));
    let WorthQuerySourceOnlyAdvance::Completed(completed) = retained.advance(&request) else {
        panic!("real initial source publication")
    };
    completed
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}"));
    assert_eq!(host.contacts.resource.counts(), (0, 0, 0));
    host
}

pub fn start(
    request: &WorthQueryApplicationResourcesRequest<'_, '_, '_, CapabilitySchema>,
    input: u64,
) -> WorthQueryApplicationResource<CapabilitySchema, Resource> {
    request.start::<Resource>(input).unwrap_or_else(|stop| {
        let (_, cause) = stop.into_parts();
        panic!("resource start: {cause:?}")
    })
}
