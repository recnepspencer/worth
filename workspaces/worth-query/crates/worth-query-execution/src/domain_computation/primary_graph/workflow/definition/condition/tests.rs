use super::*;

fn operand(name: &str) -> [&str; OPERAND_FIELDS] {
    [name, "query.v1", "parameter.v1", "result.v1", "binding.v1"]
}

fn expression(names: &[&str]) -> Option<CompiledWorkflowCondition> {
    CompiledWorkflowCondition::from_record(
        encode_draft(&[0x0a, 0xff]),
        None,
        None,
        None,
        Some(encode_operands(names.iter().map(|name| operand(name)))),
    )
}

#[test]
fn an_expression_record_round_trips_its_draft_and_operands() {
    let condition = expression(&["left", "right"]).expect("the record reads");
    assert_eq!(
        condition.expression,
        CompiledWorkflowConditionExpression::Draft(Box::new([0x0a, 0xff]))
    );
    assert_eq!(
        condition
            .operands
            .iter()
            .map(WorkflowConditionOperand::name)
            .collect::<Vec<_>>(),
        ["left", "right"]
    );
    assert_eq!(condition.operands[1].fields(), operand("right"));
}

#[test]
fn a_version_1_record_compiles_as_the_migrated_condition_with_its_original_identity() {
    let condition = CompiledWorkflowCondition::from_record(
        "query.v1".to_owned(),
        Some("parameter.v1".to_owned()),
        Some("result.v1".to_owned()),
        Some("binding.v1".to_owned()),
        None,
    )
    .expect("the version-1 record reads");
    assert_eq!(
        condition.expression,
        CompiledWorkflowConditionExpression::Migrated
    );
    assert_eq!(condition.operands[0].fields(), operand("condition"));
    assert_eq!(
        condition.identity_fields(),
        ["query.v1", "parameter.v1", "result.v1", "binding.v1"]
    );
    let expression = expression(&["condition"]).expect("the record reads");
    assert_ne!(expression.identity_fields(), condition.identity_fields());
}

#[test]
fn malformed_condition_records_do_not_read() {
    let read = |member: &str, operands: &str| {
        CompiledWorkflowCondition::from_record(
            member.to_owned(),
            None,
            None,
            None,
            Some(operands.to_owned()),
        )
    };
    let one = encode_operands([operand("left")]);
    assert!(read("0a", &one).is_some());
    for member in ["", "0", "0A", "zz"] {
        assert!(read(member, &one).is_none(), "member {member:?}");
    }
    let unordered = encode_operands([operand("right"), operand("left")]);
    let duplicated = encode_operands([operand("left"), operand("left")]);
    let partial = one.trim_end_matches("10:binding.v1").to_owned();
    for operands in [
        "",
        "4:left",
        "04:left",
        "0:",
        "9:left",
        "x:left",
        unordered.as_str(),
        duplicated.as_str(),
        partial.as_str(),
    ] {
        assert!(read("0a", operands).is_none(), "operands {operands:?}");
    }
    // A record carries either the version-1 fields or operands, never both.
    assert!(CompiledWorkflowCondition::from_record(
        "0a".to_owned(),
        Some("parameter.v1".to_owned()),
        None,
        None,
        Some(one),
    )
    .is_none());
    assert!(CompiledWorkflowCondition::from_record(
        "query.v1".to_owned(),
        Some("parameter.v1".to_owned()),
        Some("result.v1".to_owned()),
        None,
        None,
    )
    .is_none());
}
