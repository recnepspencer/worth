//! Static admission, readmission and identity of where-filter programs.
use super::{
    ApplicationQueryWhereFilterDenial as Denial, ApplicationQueryWhereFilterOperand as Operand,
    ApplicationQueryWhereFilterProgram as Program, ApplicationQueryWhereFilterSource as Source,
};
use crate::application_query::{
    validate_portable_application_query_freshly, ApplicationQueryBasisSupport,
    ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDefinitionDenial, ApplicationQueryDisclosureContract,
    ApplicationQueryLaneEligibility, ApplicationQueryParameterDefinition,
    ApplicationQueryPredicate, ApplicationQueryRelationPredicate,
    ApplicationQueryResultRelation, ApplicationQueryResultShape,
    ApplicationQueryResultShapeBuilder, ApplicationQueryResultTraversalDirection,
    ErasedApplicationQueryDefinition, WorthQueryPortableApplicationQueryResultRelationParts,
    WorthQueryPortableApplicationQueryResultShapeParts,
};
use crate::application_schema::{ApplicationEntityRef, ApplicationFieldPresence};
use crate::portable_identity::WorthQueryPortableTypeIdentity;
use worth_foundational::expression_api::{expressions, ExpressionDenialFamily};
use worth_foundational::facade::{
    AspectValue, CanonicalDate, InternedString, ScalarAspectType, Symbol,
};

struct Schema;
struct Entity;
struct Parameters;
struct Row;
crate::worth_query_structured_value_binding!(ParametersBinding for Parameters { identity: "worth.query.test.where-filter-parameters.v1" });
crate::worth_query_structured_value_binding!(RowBinding for Row { identity: "worth.query.test.where-filter-row.v1" });
crate::worth_query_application_query!(
    Query for Schema,
    identity "worth.query.test.where-filter.v1",
    parameters ParametersBinding,
    result RowBinding,
    scope Entity => "worth.query.test.where-filter-scope.v1",
    name "where_filter_query"
);

fn field(name: &str, entity: &str, scalar: ScalarAspectType, optional: bool) -> Operand {
    Operand::from_untrusted(
        name.to_owned(),
        Source::Field {
            entity: entity.to_owned(),
            aspect: "Task".to_owned(),
            field: name.to_owned(),
            scalar_family: scalar,
            presence: if optional {
                ApplicationFieldPresence::Optional
            } else {
                ApplicationFieldPresence::Required
            },
        },
    )
}

fn parameter(name: &str, scalar: ScalarAspectType) -> Operand {
    Operand::from_untrusted(
        name.to_owned(),
        Source::Parameter {
            parameter: name.to_owned(),
            scalar_family: scalar,
        },
    )
}

fn admit(operands: Vec<Operand>, source: &str) -> Result<Program, Denial> {
    Program::admit(
        operands,
        &expressions().parse(source).expect("test source parses"),
    )
}

fn text_search() -> Program {
    admit(
        vec![
            parameter("q", ScalarAspectType::String),
            field("title", "Entity", ScalarAspectType::String, false),
            field("notes", "Entity", ScalarAspectType::String, false),
        ],
        "contains_ignore_ascii_case(title, q) || contains_ignore_ascii_case(notes, q)",
    )
    .expect("a Bool text search admits")
}

#[test]
fn an_admitted_filter_readmits_from_its_portable_draft_with_equal_identity() {
    let program = text_search();
    assert_eq!(
        program
            .operands()
            .iter()
            .map(Operand::name)
            .collect::<Vec<_>>(),
        ["notes", "q", "title"]
    );
    let readmitted = Program::from_untrusted(program.operands().to_vec(), program.draft())
        .expect("its own draft readmits");
    assert_eq!(readmitted, program);
    assert_eq!(readmitted.semantic_material(), program.semantic_material());
}

#[test]
fn static_admission_refuses_non_bool_unknown_mismatched_and_unused_operands() {
    let family = |result: Result<Program, Denial>| match result {
        Err(Denial::Expression(denial)) => Some(denial.family()),
        _ => None,
    };
    let rank = || field("rank", "Entity", ScalarAspectType::Int64, false);
    let q = || parameter("q", ScalarAspectType::String);
    assert_eq!(
        family(admit(vec![rank()], "rank + 1")),
        Some(ExpressionDenialFamily::TypeMismatch)
    );
    assert_eq!(
        family(admit(vec![rank()], "rank == missing")),
        Some(ExpressionDenialFamily::UnknownBinding)
    );
    assert_eq!(
        family(admit(vec![rank(), q()], "rank == q")),
        Some(ExpressionDenialFamily::TypeMismatch)
    );
    let due = || field("due", "Entity", ScalarAspectType::Date, true);
    let max_due = || parameter("max_due", ScalarAspectType::Date);
    assert_eq!(
        family(admit(vec![due(), max_due()], "due <= max_due")),
        Some(ExpressionDenialFamily::TypeMismatch),
        "an optional field is an Option operand, never a sentinel"
    );
    assert!(admit(
        vec![due(), max_due()],
        "is_some(due) && unwrap(due) <= max_due"
    )
    .is_ok());
    assert_eq!(
        admit(vec![rank(), q()], "rank > 1"),
        Err(Denial::UnusedOperand)
    );
    assert_eq!(
        admit(vec![rank(), rank()], "rank > 1"),
        Err(Denial::DuplicateOperand)
    );
    assert_eq!(admit(Vec::new(), "true"), Err(Denial::NoOperands));
    assert_eq!(
        admit(vec![parameter(" q", ScalarAspectType::String)], "true"),
        Err(Denial::InvalidOperandName)
    );
    assert_eq!(
        admit(
            vec![parameter("blob", ScalarAspectType::InlineBytes)],
            "true"
        ),
        Err(Denial::UnsupportedScalar)
    );
    assert!(Program::from_untrusted(vec![rank()], b"not a draft").is_err());
}

fn definition_parts() -> crate::application_query::WorthQueryPortableApplicationQueryParts {
    let entity = ApplicationEntityRef::<Schema, Entity>::from_schema_identifier("Entity");
    ApplicationQueryDefinitionBuilder::declare(Query::reference())
        .root(entity)
        .scope(entity)
        .result_shape(
            ApplicationQueryResultShapeBuilder::<Schema, Query, Entity, Row, RowBinding>::new(
                entity,
            )
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::OptionalOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("the unfiltered fixture is valid")
        .into_erased()
        .into_parts()
}

fn declared(name: &str, scalar: ScalarAspectType) -> ApplicationQueryParameterDefinition {
    ApplicationQueryParameterDefinition::from_untrusted_fields(
        name.to_owned(),
        scalar,
        WorthQueryPortableTypeIdentity::declared("worth.tests.where-filter-parameter.v1"),
        None,
        None,
    )
}

#[test]
fn the_filter_joins_canonical_identity_only_when_present() {
    let unfiltered = ErasedApplicationQueryDefinition::from_untrusted_parts(definition_parts());
    let mut parts = definition_parts();
    parts.where_filter = None;
    assert_eq!(
        ErasedApplicationQueryDefinition::from_untrusted_parts(parts.clone()).canonical_basis(),
        unfiltered.canonical_basis(),
        "an absent filter contributes no identity entry"
    );
    parts.parameters = vec![declared("q", ScalarAspectType::String)];
    let parameterized = ErasedApplicationQueryDefinition::from_untrusted_parts(parts.clone());
    parts.where_filter = Some(text_search());
    let filtered = ErasedApplicationQueryDefinition::from_untrusted_parts(parts.clone());
    assert_eq!(
        validate_portable_application_query_freshly(filtered.parts()),
        Ok(())
    );
    assert_ne!(filtered.canonical_basis(), parameterized.canonical_basis());
    parts.where_filter = Some(
        admit(
            vec![
                parameter("q", ScalarAspectType::String),
                field("title", "Entity", ScalarAspectType::String, false),
            ],
            "starts_with_ignore_ascii_case(title, q)",
        )
        .unwrap(),
    );
    let other = ErasedApplicationQueryDefinition::from_untrusted_parts(parts);
    assert_ne!(other.canonical_basis(), filtered.canonical_basis());
}

#[test]
fn definition_admission_binds_operands_to_the_root_and_declared_parameters() {
    let validate = |parameters: Vec<ApplicationQueryParameterDefinition>, filter: Program| {
        let mut parts = definition_parts();
        parts.parameters = parameters;
        parts.where_filter = Some(filter);
        validate_portable_application_query_freshly(&parts)
    };
    assert_eq!(
        validate(Vec::new(), text_search()),
        Err(ApplicationQueryDefinitionDenial::WhereFilterUnknownParameter)
    );
    assert_eq!(
        validate(vec![declared("q", ScalarAspectType::UInt32)], text_search()),
        Err(ApplicationQueryDefinitionDenial::WhereFilterParameterMismatch)
    );
    let foreign = admit(
        vec![field("title", "Other", ScalarAspectType::String, false)],
        "title == \"x\"",
    )
    .unwrap();
    assert_eq!(
        validate(Vec::new(), foreign),
        Err(ApplicationQueryDefinitionDenial::WhereFilterFieldOutsideRoot)
    );
}

mod optional_and_literal;
