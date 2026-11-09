//! Producer input is an ordinary admitted Directory read, independent of payload.
use super::super::schema::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
pub(in super::super) mod operation;

#[derive(Clone, Debug, serde::Serialize)]
pub struct ReadDirectory {
    pub directory: String,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub coordinate: (u32, u64, u32),
    pub directory: String,
    pub note: String,
}
pub struct Parameters;
worth_query_structured_value_binding!(pub InputBinding for ReadDirectory { identity: "maintained-generated.source-input.v1" });
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity: "maintained-generated.source-parameters.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "maintained-generated.source-result.v1" });
worth_query_application_query!(pub Query for Catalog, identity "maintained-generated.directory-source.v1", parameters ParametersBinding, result ResultBinding, scope Directory => "Directory", name "generated_catalog_source");
pub struct IdentitySlot;
pub struct NoteSlot;
worth_query_portable_type!(IdentitySlot => "maintained-generated.directory-identity.v1");
worth_query_portable_type!(NoteSlot => "maintained-generated.directory-note.v1");
fn identity() -> ApplicationQueryResultFieldRef<Query, IdentitySlot, Catalog, DirectoryIdentity> {
    ApplicationQueryResultFieldRef::new("directory", DirectoryIdentity::at("DirectoryIdentity"))
}
fn note() -> ApplicationQueryResultFieldRef<Query, NoteSlot, Catalog, DirectoryNote> {
    ApplicationQueryResultFieldRef::new("note", DirectoryNote::at("DirectoryNote"))
}
pub fn definition() -> ApplicationQueryDefinitionBuilder<Catalog, Query, Parameters, Row, Directory>
{
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(Directory::reference())
        .scope(Directory::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::new(Directory::reference())
                .field(identity())
                .field(note())
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}
impl WorthQueryApplicationProjection<Catalog, Query> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, Query>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            coordinate: (
                row.entity_id().partition_value(),
                row.entity_id().local_slot_value(),
                row.entity_id().generation_value(),
            ),
            directory: row.field(identity())?,
            note: row.field(note())?,
        })
    }
}
fn scope(input: ReadDirectory) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for ReadDirectory, schema Catalog,
    identity "maintained-generated.source-binding.v1", input InputBinding,
    query Query, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity,
    field DirectoryIdentity::at("DirectoryIdentity"), value scope, limits results 1
);
