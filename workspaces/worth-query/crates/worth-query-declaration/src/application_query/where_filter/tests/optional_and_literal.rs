//! Static admission, validation, and canonical identity for optional
//! parameters (`G7`) and authored literal constants (`DX-2`).
use super::*;

fn optional_parameter(name: &str, scalar: ScalarAspectType) -> Operand {
    Operand::from_untrusted(
        name.to_owned(),
        Source::OptionalParameter {
            parameter: name.to_owned(),
            scalar_family: scalar,
        },
    )
}

fn literal(name: &str, scalar: ScalarAspectType, value: AspectValue) -> Operand {
    Operand::from_untrusted(
        name.to_owned(),
        Source::Literal {
            scalar_family: scalar,
            value,
        },
    )
}

#[test]
fn optional_parameters_validate_presence_type_and_predicate_restrictions() {
    let opt_due = optional_parameter("max_due", ScalarAspectType::Date);
    let due = field("due", "Entity", ScalarAspectType::Date, true);
    assert!(admit(
        vec![due.clone(), opt_due.clone()],
        "!is_some(max_due) || (is_some(due) && unwrap(due) <= unwrap(max_due))"
    )
    .is_ok());
    assert!(matches!(
        admit(
            vec![due, opt_due.clone()],
            "is_some(due) && unwrap(due) <= max_due"
        ),
        Err(Denial::Expression(d)) if d.family() == ExpressionDenialFamily::TypeMismatch
    ));

    let opt_filter = admit(vec![opt_due], "!is_some(max_due)").unwrap();
    let req_filter = admit(
        vec![
            field("due", "Entity", ScalarAspectType::Date, false),
            parameter("max_due", ScalarAspectType::Date),
        ],
        "due <= max_due",
    )
    .unwrap();
    let validate = |parameters: Vec<ApplicationQueryParameterDefinition>, filter: Program| {
        let mut parts = definition_parts();
        parts.parameters = parameters;
        parts.where_filter = Some(filter);
        validate_portable_application_query_freshly(&parts)
    };
    assert_eq!(
        validate(
            vec![declared("max_due", ScalarAspectType::Date)],
            opt_filter.clone()
        ),
        Err(ApplicationQueryDefinitionDenial::WhereFilterParameterMismatch),
        "optional operand requires optional parameter declaration"
    );
    assert_eq!(
        validate(
            vec![declared("max_due", ScalarAspectType::Date).with_optional_presence()],
            req_filter
        ),
        Err(ApplicationQueryDefinitionDenial::WhereFilterParameterMismatch),
        "required operand refuses optional parameter declaration"
    );
    assert_eq!(
        validate(
            vec![declared("max_due", ScalarAspectType::Date).with_optional_presence()],
            opt_filter
        ),
        Ok(())
    );

    let mut null_param = definition_parts();
    null_param.parameters = vec![declared("n", ScalarAspectType::Null).with_optional_presence()];
    assert_eq!(
        validate_portable_application_query_freshly(&null_param),
        Err(ApplicationQueryDefinitionDenial::InvalidOptionalParameter)
    );

    let mut pred_opt = definition_parts();
    pred_opt.parameters =
        vec![declared("status", ScalarAspectType::String).with_optional_presence()];
    pred_opt.predicates = vec![ApplicationQueryPredicate::from_untrusted_fields(
        "Entity".into(),
        "Task".into(),
        "Status".into(),
        "status".into(),
        ScalarAspectType::String,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&pred_opt),
        Err(ApplicationQueryDefinitionDenial::PredicateParameterMismatch)
    );
}

#[test]
fn literal_operands_and_value_predicates_validate_types_bounds_and_canonical_identity() {
    let status = field("status", "Entity", ScalarAspectType::String, false);
    let open = literal(
        "open",
        ScalarAspectType::String,
        AspectValue::String(InternedString::Raw("open".into())),
    );
    let done = literal(
        "open",
        ScalarAspectType::String,
        AspectValue::String(InternedString::Raw("done".into())),
    );
    let p_open = admit(vec![status.clone(), open], "status == open").unwrap();
    let p_done = admit(vec![status.clone(), done], "status == open").unwrap();
    let p_param = admit(
        vec![status.clone(), parameter("open", ScalarAspectType::String)],
        "status == open",
    )
    .unwrap();
    assert_ne!(p_open, p_done);
    assert_ne!(p_open, p_param);

    assert_eq!(
        admit(
            vec![
                status.clone(),
                literal("open", ScalarAspectType::String, AspectValue::Int64(1))
            ],
            "status == open"
        ),
        Err(Denial::InvalidLiteralValue)
    );
    assert_eq!(
        admit(
            vec![
                status.clone(),
                literal(
                    "open",
                    ScalarAspectType::String,
                    AspectValue::String(InternedString::Symbol(Symbol(1)))
                )
            ],
            "status == open"
        ),
        Err(Denial::InvalidLiteralValue)
    );
    assert_eq!(
        admit(
            vec![
                status,
                literal(
                    "open",
                    ScalarAspectType::String,
                    AspectValue::String(InternedString::Raw("a".repeat(64 * 1024 + 1)))
                )
            ],
            "status == open"
        ),
        Err(Denial::InvalidLiteralValue)
    );

    let mut pred_val = definition_parts();
    pred_val.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Entity".into(),
        "Task".into(),
        "Due".into(),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 100,
        }),
        ScalarAspectType::Date,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&pred_val),
        Ok(())
    );
    let mut bad_pred = definition_parts();
    bad_pred.predicates = vec![ApplicationQueryPredicate::from_untrusted_value(
        "Entity".into(),
        "Task".into(),
        "Due".into(),
        AspectValue::Int64(100),
        ScalarAspectType::Date,
    )];
    assert_eq!(
        validate_portable_application_query_freshly(&bad_pred),
        Err(ApplicationQueryDefinitionDenial::InvalidPredicateValue)
    );

    // A relation EqualParameter predicate must carry a parameter, never a literal value.
    let mut forged_rel = definition_parts();
    let child = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: forged_rel.query_type.clone(),
            root_entity: "Entity".into(),
            result_type: forged_rel.result_type.clone(),
            fields: vec![],
            relations: vec![],
        },
    );
    forged_rel.result_shape = ApplicationQueryResultShape::from_untrusted_parts(
        WorthQueryPortableApplicationQueryResultShapeParts {
            query_type: forged_rel.query_type.clone(),
            root_entity: "Entity".into(),
            result_type: forged_rel.result_type.clone(),
            fields: vec![],
            relations: vec![ApplicationQueryResultRelation::from_untrusted_parts(
                WorthQueryPortableApplicationQueryResultRelationParts {
                    query_type: forged_rel.query_type.clone(),
                    slot_type: WorthQueryPortableTypeIdentity::declared("worth.tests.rel-slot.v1"),
                    relation: "Rel".into(),
                    from: "Entity".into(),
                    to: "Entity".into(),
                    direction: ApplicationQueryResultTraversalDirection::Forward,
                    output_name: "rel".into(),
                    cardinality: ApplicationQueryCardinality::OptionalOne,
                    predicate: Some(ApplicationQueryRelationPredicate::EqualParameter(
                        ApplicationQueryPredicate::from_untrusted_value(
                            "Entity".into(),
                            "Task".into(),
                            "Status".into(),
                            AspectValue::String(InternedString::Raw("open".into())),
                            ScalarAspectType::String,
                        ),
                    )),
                    nested_shape: child,
                },
            )],
        },
    );
    assert_eq!(
        validate_portable_application_query_freshly(&forged_rel),
        Err(ApplicationQueryDefinitionDenial::PredicateParameterMismatch)
    );
}
