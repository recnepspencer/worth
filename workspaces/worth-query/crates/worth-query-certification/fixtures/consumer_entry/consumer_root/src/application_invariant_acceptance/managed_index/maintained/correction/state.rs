//! Fresh ordinary note disclosure and final-state observation in the same Catalog.
use super::*;
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub note: String,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "maintained-catalog.note-state.row.v1" });
worth_query_application_query!(pub NoteQuery for Catalog,
    identity "maintained-catalog.note-state.v1", parameters query::ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "note_state");
pub struct NoteSlot;
worth_query_portable_type!(NoteSlot => "maintained-catalog.note-state.slot.v1");
fn note() -> ApplicationQueryResultFieldRef<
    NoteQuery,
    NoteSlot,
    Catalog,
    Directory,
    DirectoryFacts,
    DirectoryNote,
    String,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("note", DirectoryNote::reference())
}
impl WorthQueryApplicationProjection<Catalog, NoteQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, NoteQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            note: row.field(note())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub directory: String,
}
worth_query_structured_value_binding!(pub ReadBinding for Read { identity: "maintained-catalog.note-state.input.v1" });
fn scope(input: Read) -> String {
    input.directory
}
worth_query_query_binding!(pub Binding for Read, schema Catalog,
    identity "maintained-catalog.note-state.binding.v1", input ReadBinding,
    query NoteQuery, parameters query::ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope, limits results 1
);
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let query =
        ApplicationQueryDefinitionBuilder::declare(NoteQuery::reference())
            .root(Directory::reference())
            .scope(Directory::reference())
            .result_shape(
                ApplicationQueryResultShapeBuilder::<
                    Catalog,
                    NoteQuery,
                    Directory,
                    Row,
                    ResultBinding,
                >::new(Directory::reference())
                .field(note())
                .build(),
            )
            .cardinality(ApplicationQueryCardinality::ExactlyOne)
            .disclosure(ApplicationQueryDisclosureContract::public())
            .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
            .lanes(ApplicationQueryLaneEligibility::one_shot())
            .public()
            .build()
            .unwrap();
    schema
        .application_query(query)
        .application_query_binding::<Binding>()
}
