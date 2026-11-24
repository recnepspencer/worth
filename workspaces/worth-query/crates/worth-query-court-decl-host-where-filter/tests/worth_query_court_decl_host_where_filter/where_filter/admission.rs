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

    // Typed `.where_equal_value(...)` on a unit-bearing equality-queryable field
    // refuses at schema declaration with `InvalidPredicateValue`.
    #[derive(Clone, Copy)]
    struct UnitSchema;
    #[derive(Clone, Copy)]
    struct UnitModel;
    struct UnitBindings;
    #[derive(Clone, Copy)]
    struct Beam;
    struct BeamFacts;
    struct BeamRow {
        span: Meter,
    }
    impl worth_query_decl::facade::application_schema::fragment::BindingSet for UnitBindings {
        type Work<'a> = ValueDecodeAdmission<'a>;
        fn with_admission<T>(
            work: &mut Self::Work<'_>,
            run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
        ) -> T {
            run(work)
        }
    }
    impl worth_query_decl::facade::application_schema::fragment::Scalar<Meter> for UnitBindings {
        type Binding = MeterBinding;
        const WRITABLE: bool = true;
    }
    impl worth_query_decl::facade::application_schema::fragment::Entity for Beam {
        const NAME: &'static str = "Beam";
        type Aspects = (BeamFacts,);
        type Relations = ();
    }
    impl worth_query_decl::facade::application_schema::fragment::Aspect for BeamFacts {
        const NAME: &'static str = "BeamFacts";
        type Row = BeamRow;
    }
    impl worth_query_decl::facade::application_schema::fragment::Record<BeamRow> for UnitBindings {
        fn record(
        ) -> impl worth_query_decl::facade::application_schema::fragment::Shape<BeamRow, Self>
        {
            worth_query_decl::facade::application_schema::fragment::field(
                "Span",
                |row: &BeamRow| Meter(row.span.0),
            )
            .equality_queryable(true)
            .build(|span, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(BeamRow { span })
            })
        }
    }
    type SpanField = worth_query_decl::facade::application_schema::fragment::FieldSelection<
        UnitModel,
        Beam,
        BeamFacts,
        Meter,
    >;
    struct BeamParams;
    struct BeamResult;
    struct SpanSlot;
    worth_query_decl::facade::worth_query_portable_type!(SpanSlot => "where-filter.unit-span-slot.v1");
    worth_query_decl::facade::worth_query_structured_value_binding!(BeamParamsBinding for BeamParams { identity: "where-filter.unit-params.v1" });
    worth_query_decl::facade::worth_query_structured_value_binding!(BeamResultBinding for BeamResult { identity: "where-filter.unit-result.v1" });
    worth_query_decl::facade::worth_query_application_query!(
        UnitLiteralQuery for UnitSchema,
        identity "where-filter.unit-literal.v1",
        parameters BeamParamsBinding,
        result BeamResultBinding,
        scope Beam => "Beam",
        name "unit_literal"
    );
    struct UnitQueries;
    impl worth_query_decl::facade::application_schema::fragment::Registration<UnitSchema>
        for UnitQueries
    {
        fn register(
            builder: ApplicationSchemaDeclarationBuilder<UnitSchema>,
        ) -> Result<ApplicationSchemaDeclarationBuilder<UnitSchema>, ApplicationSchemaDeclarationDenial>
        {
            let beam = worth_query_decl::facade::application_schema::fragment::entity(UnitModel, Beam);
            let ten_meters =
                ApplicationEncodedScalarValue::<MeterBinding>::try_new(Meter(10)).unwrap();
            let shape = ApplicationQueryResultShapeBuilder::<
                UnitSchema,
                UnitLiteralQuery,
                Beam,
                BeamResult,
                BeamResultBinding,
            >::new(beam)
            .field(ApplicationQueryResultFieldRef::<
                UnitLiteralQuery,
                SpanSlot,
                UnitSchema,
                SpanField,
            >::new("span", SpanField::at("Span")))
            .build();
            Ok(builder.application_query(
                ApplicationQueryDefinitionBuilder::declare(UnitLiteralQuery::reference())
                    .root(beam)
                    .scope(beam)
                    .result_shape(shape)
                    .cardinality(ApplicationQueryCardinality::OptionalOne)
                    .disclosure(ApplicationQueryDisclosureContract::public())
                    .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
                    .lanes(ApplicationQueryLaneEligibility::one_shot())
                    .public()
                    .where_equal_value(SpanField::at("Span"), ten_meters),
            ))
        }
    }
    impl worth_query_decl::facade::application_schema::fragment::Fragment for UnitModel {
        const NAME: &'static str = "unit_model";
        type Bindings = UnitBindings;
        type Members = (Beam,);
    }
    impl worth_query_decl::facade::application_schema::fragment::Fragment for UnitSchema {
        const NAME: &'static str = "unit_schema";
        type Bindings = worth_query_decl::facade::application_schema::fragment::NoBindings;
        type Members = (UnitModel, UnitQueries);
    }
    impl ApplicationSchema for UnitSchema {
        const OWNER: &'static str = "worth.query.where-filter";
        const NAME: &'static str = "unit_schema";
        fn declaration(
        ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
            worth_query_decl::facade::application_schema::fragment::declare(Self)
        }
    }
    match UnitSchema::declaration() {
        Err(ApplicationSchemaDeclarationDenial::InvalidQueryDefinition { cause, .. }) => {
            assert_eq!(cause, ApplicationQueryDefinitionDenial::InvalidPredicateValue);
        }
        other => panic!("unit-bearing where_equal_value must refuse: {:?}", other.err()),
    }
}
