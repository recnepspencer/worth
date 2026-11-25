//! A generic Source whose conservative key stays constant while exact samples change.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_decl::facade::application_query::*;
use worth_query_host::facade::application_invariants::*;
mod courts;
mod directory_home;
pub mod expectation;
pub mod optional_aspect;
mod provider;
pub mod query;
mod rule;
pub use provider::Provider;
pub use rule::{definition, resolve, CheckSamples};
#[derive(Default)]
pub struct Calls {
    pub optional_updates: AtomicUsize,
    pub optional_changes: Mutex<
        Vec<(
            Option<optional_aspect::query::Row>,
            Option<optional_aspect::query::Row>,
        )>,
    >,
    pub reader_impostor: AtomicUsize,
    pub reader_bytes: Mutex<Vec<Vec<u8>>>,
    pub cancel_content: Mutex<Option<WorthQueryCancellationSource>>,
    pub content_reads: AtomicUsize,
    pub exact_reads: AtomicUsize,
    pub updates: AtomicUsize,
    pub swallowed_projection: AtomicUsize,
    pub records: Mutex<Vec<(Option<Vec<String>>, Vec<String>)>>,
}
worth_query_value_binding!(pub ProbeBinding for u64 { identity:"content-source.minimum.v1", scalar:String, canonical:ValueDecodeBounds::new(32,16,2,2), });
pub struct Index;
impl ApplicationManagedIndexBinding<Schema> for Index {
    type SourceBinding = query::SourceBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = Record;
    const NAME: &'static str = "content_sample_index";
    const PROVIDER: &'static str = "content-source.provider.v1";
    const PROVIDER_VERSION: u32 = 1;
    const STRUCTURE: ApplicationManagedIndexSourceStructure =
        ApplicationManagedIndexSourceStructure::BinarySourceTree;
    const MEMBERSHIP: ApplicationManagedIndexMembership =
        ApplicationManagedIndexMembership::ExhaustiveSourceTree;
    const REFINEMENT: Option<ApplicationManagedIndexRefinement> = Some(
        ApplicationManagedIndexRefinement::row_local("content-source.minimum-sample", 1),
    );
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    optional_aspect::declare(expectation::declare(schema))
        .application_query(query::state())
        .application_query_binding::<query::StateBinding>()
        .application_query(query::source())
        .application_query_binding::<query::SourceBinding>()
        .application_query(query::exact())
        .application_query_binding::<query::FindBinding>()
        .managed_index(ApplicationManagedIndexDefinition::<Schema, Index>::new())
        .invariant(definition())
}

/// A recovered or reopened Home opens its managed index `Created` (public
/// #359). Recovered bytes serve no live Source image until the owner's explicit
/// rebuild places one at the readmitted head; `rebuilt` names the rows that
/// rebuild must then serve. Neither the open, the refused read nor the rebuild
/// moves the World.
pub fn cold_index(host: &world::Host, rebuilt: Option<&[&str]>) {
    let selected = world::pin(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let find = || query::Find {
        root: "record".into(),
        minimum: 10,
    };
    let index = request
        .query(find())
        .open_index::<Index>()
        .expect("a recovered Home opens its managed index Created");
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Created);
    let refused = match request.query(find()).execute() {
        Err(cause) => cause,
        Ok(_) => panic!("recovered bytes serve no live managed Source image before a rebuild"),
    };
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = refused else {
        panic!("{refused:?}")
    };
    assert!(matches!(
        cause.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::RootUnavailable)
    ));
    if let Some(expected) = rebuilt {
        request.query(find()).rebuild_index(&index).unwrap();
        assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        let rows = request.query(find()).execute().unwrap();
        let keys = rows.rows().iter().map(|row| row.key.as_str());
        assert_eq!(keys.collect::<Vec<_>>(), expected);
    }
    assert_eq!(
        world::pin(host).selected_commit(),
        selected.selected_commit()
    );
    index.close();
}
