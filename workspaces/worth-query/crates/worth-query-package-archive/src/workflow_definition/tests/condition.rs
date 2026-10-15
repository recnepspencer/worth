use super::*;

const CONDITION: u8 = 3;
const QUERY: [&str; 3] = ["query.v1", "parameter.v1", "result.v1"];

impl Draft {
    /// One condition node `a` whose body the caller writes, and no
    /// connections.
    fn condition(mut self, body: impl FnOnce(&mut BinaryOutput)) -> Vec<u8> {
        self.output.u32(1);
        self.output.raw_bytes(&[CONDITION]);
        self.output.text("a");
        body(&mut self.output);
        self.output.u32(0);
        self.bytes()
    }
}

fn expression(output: &mut BinaryOutput, draft: &[u8], names: &[&str]) {
    output.u32(count(draft.len()));
    output.raw_bytes(draft);
    output.u16(u16::try_from(names.len()).expect("test counts are small"));
    for name in names {
        output.text(name);
        QUERY.iter().for_each(|field| output.text(field));
    }
}

fn decoded_condition(bytes: &[u8]) -> DraftCondition {
    let draft = decode_workflow_definition_draft(bytes, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect("the condition draft decodes");
    match &draft.nodes[0].member {
        DraftMember::Condition(condition) => condition.clone(),
        other => panic!("expected a condition, found {other:?}"),
    }
}

fn operand(name: &str) -> DraftConditionOperand {
    DraftConditionOperand {
        name: name.to_owned(),
        identifier: QUERY[0].to_owned(),
        parameter_type: QUERY[1].to_owned(),
        result_type: QUERY[2].to_owned(),
    }
}

#[test]
fn a_current_condition_carries_its_draft_and_named_operands() {
    let bytes = Draft::current().condition(|output| expression(output, b"xy", &["left", "right"]));
    assert_eq!(
        decoded_condition(&bytes),
        DraftCondition::Expression {
            draft: Box::from(&b"xy"[..]),
            operands: Box::new([operand("left"), operand("right")]),
        }
    );
}

#[test]
fn malformed_condition_records_are_refused_by_kind() {
    let condition = |draft: &[u8], names: &[&str]| {
        Draft::current().condition(|output| expression(output, draft, names))
    };
    assert_eq!(
        refused(&condition(b"", &["left"])),
        Kind::InvalidRecordShape
    );
    assert_eq!(refused(&condition(b"x", &[])), Kind::InvalidRecordShape);
    assert_eq!(refused(&condition(b"x", &[""])), Kind::InvalidRecordShape);
    assert_eq!(
        refused(&condition(b"x", &["right", "left"])),
        Kind::NonCanonicalRecordSequence
    );
    assert_eq!(
        refused(&condition(b"x", &["left", "left"])),
        Kind::NonCanonicalRecordSequence
    );
    let overlong = Draft::current().condition(|output| {
        output.u32(1);
        output.raw_bytes(b"x");
        output.u16(u16::MAX);
    });
    assert_eq!(refused(&overlong), Kind::Truncated);
    let overlong_draft = Draft::current().condition(|output| output.u32(u32::MAX));
    assert_eq!(refused(&overlong_draft), Kind::Truncated);
}
