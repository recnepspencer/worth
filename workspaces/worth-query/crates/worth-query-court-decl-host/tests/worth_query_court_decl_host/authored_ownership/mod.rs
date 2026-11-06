//! Real installation admits only source-specific opt-in, retaining independent scopes.
use std::marker::PhantomData;
use worth_query_decl::facade::{
    application_aftermath::*, application_operation::*, application_program::*,
    application_query::*, application_schema::*, authentication::*, *,
};
use worth_query_host::facade::{
    application_contribution::*, application_entry::*, application_installation::*,
    primary_graph::*,
};
use worth_query_testkit_document_retention::facade::document_retention_model::host::host_limits;
mod readiness;
mod schema;
use schema::*;
mod correction;
mod correction_courts;
mod lifecycle;
mod lifecycle_handlers;
mod producer;
mod program;
mod query;

fn install<const MODE: u8>() -> Result<
    WorthQueryPrimaryGraphApplicationRuntime<Model<MODE>>,
    WorthQueryInMemoryApplicationDenial,
> {
    in_memory(
        Model::<MODE>::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000001").unwrap(),
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        |_, graph, installed| {
            graph.bind_principal(
                &installed.principal_binding(actor()).unwrap(),
                WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
                1,
                WorthQueryExternalPrincipalIdentity::new("https://ownership.invalid", "operator")
                    .unwrap(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            for (key, id) in [("source", 2), ("peer", 3)] {
                graph.bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Node::reference(),
                        WorthQueryApplicationEntityKey::new(key).unwrap(),
                    )
                    .field(Id::at("Id"), id)
                    .field(Value::at("Value"), 0),
                )?;
            }
            graph.bind_relation(WorthQueryApplicationRelationSeed::new(
                Peer::reference(),
                "shared-input",
                WorthQueryApplicationEntityKey::new("source").unwrap(),
                WorthQueryApplicationEntityKey::new("peer").unwrap(),
            ))?;
            Ok(())
        },
    )
}
#[test]
fn entity_source_scope_installs_and_shared_scopes_refuse_with_producer_subject() {
    let installed = install::<0>();
    assert!(installed.is_ok(), "{:?}", installed.err());
    for failure in [install::<1>().err().unwrap(), install::<2>().err().unwrap()] {
        let WorthQueryInMemoryApplicationDenial::Contributions(failure) = failure else {
            panic!("ownership must refuse at producer installation: {failure:?}")
        };
        assert_eq!(failure.kind(), WorthQueryPrimaryGraphInstallationDenialKind::ProducerOwnershipRequiresEntitySourceScope);
        assert_eq!(failure.subject(), "authored-ownership.producer.v1");
    }
}
#[test]
fn independent_producer_keeps_shared_context_installation() {
    let installed = install::<3>();
    assert!(installed.is_ok(), "{:?}", installed.err());
}
