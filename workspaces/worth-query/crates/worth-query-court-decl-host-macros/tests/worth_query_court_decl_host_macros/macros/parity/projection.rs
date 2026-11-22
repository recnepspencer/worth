//! A derived projection with a renamed output and an explicit slot identity
//! declares exactly the query hand-written slot markers declare.
use super::super::reads::ParametersBinding;
use super::super::schema::*;
use super::board;
use worth_foundational::facade::CanonicalDate;
use worth_query_decl::facade::application_identity::ApplicationIdentityNamespace;
use worth_query_decl::facade::portable_identity::WorthQueryPortableType;
use worth_query_decl::facade::{
    application_query::*, application_schema::*, worth_query_application_query,
    worth_query_identity, worth_query_identity_namespace, worth_query_structured_value_binding,
    WorthQueryProjection,
};

pub const LEGACY: ApplicationIdentityNamespace =
    worth_query_identity_namespace!("macros.tasks.legacy.v1");

#[derive(Clone, Debug, Eq, PartialEq, WorthQueryProjection)]
#[worth_query(schema = Schema, root = Task, namespace = LEGACY)]
pub struct LegacyRow {
    #[worth_query(field = TaskKey)]
    pub key: u64,
    /// Keeps the slot identity and output name already on the wire.
    #[worth_query(field = Title, name = "heading", slot = "board.row.title.v1")]
    pub title: String,
    #[worth_query(field = Due)]
    pub due: Option<CanonicalDate>,
}
worth_query_structured_value_binding!(pub LegacyRowBinding for LegacyRow {
    identity: worth_query_identity!(LEGACY, result)
});
worth_query_application_query!(pub LegacyQuery for Schema, identity worth_query_identity!(LEGACY, query),
    parameters ParametersBinding, result LegacyRowBinding, scope Project => "Project",
    name "legacy_tasks");

/// The same slots written as marker types, portable identities and selectors.
mod written {
    use super::*;
    use worth_query_decl::facade::worth_query_portable_type;

    pub struct KeySlot;
    pub struct HeadingSlot;
    pub struct DueSlot;
    worth_query_portable_type!(KeySlot => "macros.tasks.legacy.v1.slot.key");
    worth_query_portable_type!(HeadingSlot => "board.row.title.v1");
    worth_query_portable_type!(DueSlot => "macros.tasks.legacy.v1.slot.due");

    pub fn heading() -> ApplicationQueryResultFieldRef<LegacyQuery, HeadingSlot, Schema, Title> {
        ApplicationQueryResultFieldRef::new("heading", Title::at("Title"))
    }
    pub fn shape(
    ) -> TypedApplicationQueryResultShape<Schema, LegacyQuery, Task, LegacyRow, LegacyRowBinding>
    {
        ApplicationQueryResultShapeBuilder::new(Task::reference())
            .field(ApplicationQueryResultFieldRef::<
                LegacyQuery,
                KeySlot,
                Schema,
                TaskKey,
            >::new("key", TaskKey::at("TaskKey")))
            .field(heading())
            .optional_field(ApplicationQueryOptionalResultFieldRef::<
                LegacyQuery,
                DueSlot,
                Schema,
                Due,
            >::new("due", Due::at("Due")))
            .build()
    }
}

fn declared(
    shape: TypedApplicationQueryResultShape<Schema, LegacyQuery, Task, LegacyRow, LegacyRowBinding>,
) -> ApplicationSchemaDeclaration<Schema> {
    let query = ApplicationQueryDefinitionBuilder::declare(LegacyQuery::reference())
        .root(Task::reference())
        .scope(Project::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Project::reference()).forward(Contains::reference()),
        );
    board()
        .application_query(query)
        .build()
        .expect("the legacy query declares")
}

#[test]
fn a_derived_projection_keeps_overridden_slots_and_declares_the_written_query() {
    let (derived, written) = (
        declared(LegacyRow::result_shape()),
        declared(written::shape()),
    );
    assert_eq!(derived.identity(), written.identity());
    assert_eq!(derived.erased(), written.erased());
    assert_eq!(
        <legacy_row_slots::Heading as WorthQueryPortableType>::PORTABLE_TYPE_NAME,
        "board.row.title.v1"
    );
    assert_eq!(
        LegacyRow::title::<LegacyQuery>().slot_key(),
        written::heading().slot_key()
    );
}
