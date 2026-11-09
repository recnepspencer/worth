//! A generic Source whose conservative key stays constant while exact samples change.
use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use worth_query_decl::facade::application_query::*;
use worth_query_host::facade::application_invariants::*;
mod courts;
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
