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

    // Optional operand bound to a required parameter ref refuses declaration.
    let opt_mismatch = cause(declaration_with(|builder| {
        let req_q = parameter::<RefusedQuery, SearchText, StringApplicationValueBinding>("q");
        let opt_q = ApplicationQueryParameterRef::<
            RefusedQuery,
            SearchText,
            StringApplicationValueBinding,
            true,
        >::optional("q");
        let filter = ApplicationQueryWhereFilterOperands::new()
            .field("title", Title::at("Title"))
            .and_then(|operands| operands.parameter("q", opt_q))
            .and_then(|operands| operands.parse("!is_some(q) || title == unwrap(q)"))
            .unwrap();
        builder.application_query(
            tasks(RefusedQuery::reference())
                .parameter(req_q)
                .where_filter(filter),
        )
    }));
    assert_eq!(
        opt_mismatch,
        ApplicationQueryDefinitionDenial::WhereFilterParameterMismatch
    );

    // Non-equality field in `.where_equal_value(...)` refuses at schema closure.
    let non_eq_value = declaration_with(|builder| {
        let title_val =
            ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new("x".into())
                .unwrap();
        builder.application_query(
            tasks(RefusedQuery::reference()).where_equal_value(Title::at("Title"), title_val),
        )
    });
    assert_eq!(
        non_eq_value.err(),
        Some(ApplicationSchemaDeclarationDenial::InvalidApplicationQuery)
    );
}

#[test]
fn literal_type_mismatch_and_unit_bearing_literal_refuse_at_filter_admission() {
    let rank_lit = ApplicationEncodedScalarValue::<I64ApplicationValueBinding>::try_new(5).unwrap();
    let mismatch = ApplicationQueryWhereFilterOperands::<RefusedQuery>::new()
        .field("title", Title::at("Title"))
        .and_then(|operands| operands.literal("five", rank_lit))
        .and_then(|operands| operands.parse("title == five"))
        .err()
        .expect("string == i64 literal must refuse");
    assert_eq!(expression_family(mismatch), "TypeMismatch");

    struct Meter(i64);
    impl Meter {
        fn encode(value: &Self) -> i64 {
            value.0
        }
        fn decode(value: i64) -> Option<Self> {
            Some(Self(value))
        }
    }
    worth_query_decl::facade::worth_query_value_binding! {
        MeterBinding for Meter {
            identity: "where-filter.meter.v1",
            scalar: Int64,
            unit: "worth.units.metre.v1",
            encode: Meter::encode,
            decode: Meter::decode,
        }
    }
    let meters = ApplicationEncodedScalarValue::<MeterBinding>::try_new(Meter(10)).unwrap();
    let unit_denial = ApplicationQueryWhereFilterOperands::<RefusedQuery>::new()
        .literal("meters", meters)
        .err();
    assert_eq!(
        unit_denial,
        Some(ApplicationQueryWhereFilterDenial::UnitBearingOperand)
    );
}
