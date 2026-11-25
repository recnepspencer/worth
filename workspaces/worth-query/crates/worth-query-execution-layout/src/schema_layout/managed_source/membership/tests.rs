//! A residual filter is selection meaning a complete Source cannot carry.
use super::is_complete_membership;
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_query::*;
use worth_query_declaration::facade::application_schema::ApplicationEntityRef;
use worth_query_declaration::*;

struct Schema;
struct Item;
struct Row;
worth_query_portable_type!(Row => "worth.tests.layout.source-row.v1");
worth_query_structured_value_binding!(ParametersBinding for () { identity: "worth.rust.unit" });
worth_query_structured_value_binding!(RowsBinding for Row { identity: "worth.tests.layout.source-row.v1" });
worth_query_application_query!(Source for Schema, identity "worth.tests.layout.source.v1",
    parameters ParametersBinding, result RowsBinding, scope Item => "Item", name "source");

fn source() -> WorthQueryPortableApplicationQueryParts {
    let item = ApplicationEntityRef::<Schema, Item>::from_schema_identifier("Item");
    let mut parts = ApplicationQueryDefinitionBuilder::declare(Source::reference())
        .root(item)
        .scope(item)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Source, Item, Row, RowsBinding>::new(item)
                .build(),
        )
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .into_erased()
        .into_parts();
    parts.root_paths = vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
        ARCHIVE_DECODE,
        WorthQueryPortableApplicationQueryRootPathParts {
            start_entity: "Item".into(),
            terminal_entity: "Item".into(),
            steps: vec![ApplicationQueryRootPathStep::from_untrusted_fields(
                ARCHIVE_DECODE,
                "Next".into(),
                "Item".into(),
                "Item".into(),
                RelationDirection::Forward,
                ApplicationQueryRootPathTraversal::OneHop,
            )],
            guards: Vec::new(),
        },
    )];
    parts
}

#[test]
fn a_parameterless_filtered_source_is_not_complete_membership() {
    let plain = source();
    assert!(is_complete_membership(
        &ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, plain.clone())
    ));
    let mut filtered = plain;
    filtered.where_filter = Some(
        ApplicationQueryWhereFilterProgram::from_untrusted(ARCHIVE_DECODE,
            vec![ApplicationQueryWhereFilterOperand::from_untrusted(ARCHIVE_DECODE,
                "rank".into(),
                ApplicationQueryWhereFilterSource::Field {
                    entity: "Item".into(),
                    aspect: "Facts".into(),
                    field: "Rank".into(),
                    scalar_family: worth_foundational::facade::ScalarAspectType::Int64,
                    presence: worth_query_declaration::facade::application_schema::ApplicationFieldPresence::Required,
                },
            )],
            &worth_foundational::expression_api::expressions()
                .parse("rank >= 5")
                .unwrap()
                .encode(),
        )
        .unwrap(),
    );
    assert!(!is_complete_membership(
        &ErasedApplicationQueryDefinition::from_untrusted_parts(ARCHIVE_DECODE, filtered)
    ));
}
