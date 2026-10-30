pub use std::sync::OnceLock;

pub use worth_query::facade::{domain, read};

pub use super::super::{canonical_ordered_collection_bundle, identity_contract};
pub use super::collection_semantics;

pub fn routing_collection_semantics() -> domain::WorthQueryDomainOperationSemanticClosure {
    let mut semantics = collection_semantics();
    semantics.canonical_query =
        canonical_ordered_collection_bundle("Vertex", "ordering", "position");
    let domain::WorthQueryOperationCollectionContract::Collection {
        ordering_fields,
        grouping,
        ..
    } = &mut semantics.collection
    else {
        unreachable!("collection semantics retain their collection contract")
    };
    *ordering_fields =
        vec![
            domain::WorthQueryOperationCollectionField::from_dotted("ordering.position")
                .expect("valid installed-only ordering field"),
        ];
    *grouping = domain::WorthQueryOperationGroupingContract::Ungrouped;
    semantics
}

pub fn routing_collection_read_declaration() -> &'static read::WorthQueryReadDeclaration {
    static DECLARATION: OnceLock<read::WorthQueryReadDeclaration> = OnceLock::new();
    DECLARATION.get_or_init(|| {
        read::declare(|builder| {
            builder.local_collection(
                "Vertex",
                read::QuerySchemaView::new(
                    "impact-routing-collection",
                    [
                        read::SchemaFieldView::new(
                            read::AspectName::new("identity").unwrap(),
                            read::FieldName::new("id").unwrap(),
                            read::ScalarAspectType::String,
                        ),
                        read::SchemaFieldView::new(
                            read::AspectName::new("ordering").unwrap(),
                            read::FieldName::new("position").unwrap(),
                            read::ScalarAspectType::String,
                        ),
                    ],
                    [],
                ),
                |query| {
                    query
                        .project(read::AspectFieldSelector::new("identity", "id").unwrap())
                        .project(read::AspectFieldSelector::new("ordering", "position").unwrap())
                        .order_by(
                            read::OrderingSelector::ascending("ordering", "position").unwrap(),
                        )
                },
                |shape| {
                    shape
                        .field(read::AuthoredResultShapeField::new("identity", "id", "id").unwrap())
                },
            )
        })
        .expect("routing collection declaration is canonical")
    })
}

pub fn routing_collection_schema() -> worth_query::facade::consumer_kit::WorthQueryTestBackendSchema
{
    use worth_foundational::facade::{
        AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
        AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
        StructAspectShape,
    };

    let position = FieldDeclaration::new(
        FieldKey::new("position").unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    let ordering = AspectContract::struct_aspect(
        AspectKey::new("ordering").unwrap(),
        AspectIdentity(0x5751_9018),
        AspectContractRevision(1),
        StructAspectShape::new([position]).unwrap(),
    );
    worth_query::facade::consumer_kit::WorthQueryTestBackendSchema::single_collection("Vertex")
        .aspect_contract(identity_contract())
        .unwrap()
        .aspect_contract(ordering)
        .unwrap()
        .aspect("identity.id", "identity.id")
        .unwrap()
        .aspect("ordering.position", "ordering.position")
        .unwrap()
}
