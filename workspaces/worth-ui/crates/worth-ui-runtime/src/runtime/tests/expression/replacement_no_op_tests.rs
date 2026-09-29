use super::cutover_fixture::replace;
use super::session_fixture::{launch, source};
use crate::facade::expression::UiExpressionOutcome;

const READY: &str = "condition ex.ready { operand r application-boolean app.ready; when (r) }";
const COMMENTED: &str =
    "// trivia\ncondition ex.ready { operand r application-boolean app.ready; when (r) }";
const NEGATED: &str = "condition ex.ready { operand r application-boolean app.ready; when (!r) }";

#[test]
fn a_changed_condition_body_activates_while_trivia_stays_a_semantic_no_op() {
    let mut session = launch(READY);
    let _receipt = replace(&mut session, source(READY))
        .into_activation()
        .expect("the first replacement commits the mounted allocation");
    for (declarations, label) in [(READY, "an identical"), (COMMENTED, "a comment-only")] {
        assert!(
            replace(&mut session, source(declarations))
                .semantic_no_op()
                .is_some(),
            "{label} condition is not executable meaning"
        );
    }
    assert_eq!(
        session.expression_record("ex.ready").unwrap().outcome(),
        &UiExpressionOutcome::Condition(false)
    );

    assert!(
        replace(&mut session, source(NEGATED))
            .activation()
            .is_some(),
        "a changed condition body is executable meaning the intent contract cannot see"
    );
    assert_eq!(
        session.expression_record("ex.ready").unwrap().outcome(),
        &UiExpressionOutcome::Condition(true)
    );
}
