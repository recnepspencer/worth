use worth_foundational::expression_api::ExpressionReadKind;

use super::session_fixture::{launch, READY};
use crate::facade::intent::UiIntentApplicationFact;

const SHORT_CIRCUIT: &str = "condition ex.guarded { operand a application-boolean app.ready; operand b application-unsigned64 app.count; when (a && exact_cast<Int64>(b) > 1) }";

fn read_operands(
    session: &crate::facade::WorthUiActiveApplicationSession,
) -> Vec<(String, ExpressionReadKind)> {
    session
        .expression_record("ex.guarded")
        .expect("`ex.guarded` is installed")
        .consumed()
        .expect("an evaluated expression records what it consumed")
        .reads()
        .iter()
        .map(|read| (read.operand().to_owned(), read.kind()))
        .collect()
}

#[test]
fn consumed_lists_each_operand_the_evaluation_read_with_its_aspect() {
    let mut session = launch(SHORT_CIRCUIT);
    session
        .update_intent_boolean_fact(&UiIntentApplicationFact::boolean(READY).unwrap(), true)
        .unwrap();

    assert_eq!(
        read_operands(&session),
        [
            ("a".to_owned(), ExpressionReadKind::Value),
            ("b".to_owned(), ExpressionReadKind::Value),
        ]
    );
}

#[test]
fn an_operand_the_evaluation_never_read_is_absent_from_consumed() {
    let session = launch(SHORT_CIRCUIT);

    assert_eq!(
        read_operands(&session),
        [("a".to_owned(), ExpressionReadKind::Value)],
        "`false && ..` short-circuits, so `b` was bound but never read"
    );
    let record = session.expression_record("ex.guarded").unwrap();
    assert_eq!(
        record.operands().len(),
        2,
        "the operand facts still record every observation"
    );
}
