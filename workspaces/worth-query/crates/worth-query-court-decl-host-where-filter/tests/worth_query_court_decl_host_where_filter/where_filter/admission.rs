//! Journey 7: typed refusals before execution, at filter admission and at
//! the schema declaration boundary.
use super::queries::*;
use super::schema::*;
use worth_query_decl::facade::{application_query::*, application_schema::*};

fn refused(source: &str) -> ApplicationQueryWhereFilterDenial {
    let q = parameter::<RefusedQuery, SearchText, StringApplicationValueBinding>("q");
    ApplicationQueryWhereFilterOperands::<RefusedQuery>::new()
        .field("title", Title::at("Title"))
        .and_then(|operands| operands.field("rank", Rank::at("Rank")))
        .and_then(|operands| operands.parameter("q", q))
        .and_then(|operands| operands.parse(source))
        .err()
        .expect("the source must be refused")
}

fn expression_family(denial: ApplicationQueryWhereFilterDenial) -> String {
    match denial {
        ApplicationQueryWhereFilterDenial::Expression(denial) => format!("{:?}", denial.family()),
        other => format!("{other:?}"),
    }
}

#[test]
fn non_bool_mismatched_unknown_and_unused_filters_refuse_statically() {
    assert_eq!(expression_family(refused("rank + 1")), "TypeMismatch");
    assert_eq!(
        expression_family(refused("title == q && rank == q")),
        "TypeMismatch"
    );
    assert_eq!(
        expression_family(refused("title == q && rank > 0 && notes == q")),
        "UnknownBinding"
    );
    assert_eq!(
        refused("title == q"),
        ApplicationQueryWhereFilterDenial::UnusedOperand
    );
    let ghost = ApplicationQueryWhereFilterOperands::<RefusedQuery>::new()
        .field("ghost", Title::at("Ghost"))
        .err();
    assert!(matches!(
        ghost,
        Some(ApplicationQueryWhereFilterDenial::InvalidRecordField { .. })
    ));
}

fn cause(
    declared: Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial>,
) -> ApplicationQueryDefinitionDenial {
    match declared {
        Err(ApplicationSchemaDeclarationDenial::InvalidQueryDefinition { cause, .. }) => cause,
        other => panic!(
            "the definition must refuse at declaration: {:?}",
            other.err()
        ),
    }
}

#[test]
fn undeclared_parameters_and_foreign_fields_refuse_at_schema_declaration() {
    assert!(Schema::declaration().is_ok());
    let undeclared = cause(declaration_with(|builder| {
        let q = parameter::<RefusedQuery, SearchText, StringApplicationValueBinding>("q");
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("title", Title::at("Title"))
            .and_then(|operands| operands.parameter("q", q))
            .and_then(|operands| operands.parse("title == q"))
            .unwrap();
        builder.application_query(tasks(RefusedQuery::reference()).where_filter(filter))
    }));
    assert_eq!(
        undeclared,
        ApplicationQueryDefinitionDenial::WhereFilterUnknownParameter
    );
    let foreign = cause(declaration_with(|builder| {
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("project", ProjectKey::at("ProjectKey"))
            .and_then(|operands| operands.parse("project >= project"))
            .unwrap();
        builder.application_query(tasks(RefusedQuery::reference()).where_filter(filter))
    }));
    assert_eq!(
        foreign,
        ApplicationQueryDefinitionDenial::WhereFilterFieldOutsideRoot
    );
}
