use worth_foundational::expression_api::ExpressionValue;
use worth_ui_query_binding::UiProjectionUnavailableKind;

use super::active_application_session::expression_session_fixture::{complete, evaluate};
use super::expression_query_fixture::{fixture, Fixture};
use crate::facade::expression::{
    UiExpressionDenialReason, UiExpressionOutcome, UiExpressionUnavailableReason,
};
use crate::facade::intent::UiIntentApplicationFact;
use crate::mounting::UiMountedFrameOutcome;
use crate::runtime::expression::{UiExpressionCompletion, UiExpressionCompletionReceipt};
use crate::runtime::tests::expression::session_fixture::since;

#[test]
fn a_query_operand_is_unavailable_until_a_frame_carries_it_and_its_readers_follow() {
    let fixture = fixture();

    assert_eq!(
        fixture.outcome("ex.online"),
        UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
            operand: "p".into()
        })
    );
    let UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Upstream {
        operand,
        identity,
    }) = fixture.outcome("ex.echoed")
    else {
        panic!("a reader of an unavailable derived value is unavailable")
    };
    assert_eq!((&*operand, &*identity), ("e", "ex.echo"));
    let counters = fixture.counters();
    assert_eq!(
        counters.evaluations, 2,
        "only `ex.ready` and `ex.ratio` ran a kernel"
    );
    assert_eq!(counters.settled_without_evaluation, 5);
}

#[test]
fn a_native_launch_leaves_every_record_on_the_established_generation() {
    let fixture = fixture();
    let session = &fixture.shell.session;
    let active = session.active_generation_identity();

    for identity in ["ex.online", "ex.echo", "ex.echoed", "ex.ready", "ex.mixed"] {
        let record = session.expression_record(identity).unwrap();
        assert_eq!(record.generation(), &active, "`{identity}`");
        let reference = session.expression_result(identity).unwrap();
        assert!(session.is_current_expression_result(&reference));
    }
}

#[test]
fn a_pending_query_is_unavailable_and_a_current_query_text_binds_as_a_string() {
    let mut fixture = fixture();

    fixture.publish_pending();
    assert!(
        matches!(
            fixture.outcome("ex.online"),
            UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Projection {
                kind: UiProjectionUnavailableKind::Pending,
                ..
            })
        ),
        "the pending publication is not a value: {:?}",
        fixture.outcome("ex.online")
    );
    assert!(matches!(
        fixture.outcome("ex.echoed"),
        UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Upstream { .. })
    ));

    fixture.publish_status("ONLINE", 1);
    assert_eq!(
        fixture.outcome("ex.online"),
        UiExpressionOutcome::Condition(true)
    );
    assert_eq!(
        fixture.outcome("ex.echo"),
        UiExpressionOutcome::Value(ExpressionValue::string("ONLINE"))
    );
    assert_eq!(
        fixture.outcome("ex.echoed"),
        UiExpressionOutcome::Condition(true)
    );
}

#[test]
fn a_fact_update_after_query_frames_evaluates_only_the_fact_readers() {
    let mut fixture = fixture();
    fixture.publish_pending();
    fixture.publish_status("ONLINE", 1);
    let before = fixture.counters();

    fixture
        .shell
        .session
        .update_intent_boolean_fact(
            &UiIntentApplicationFact::boolean("app.ready").unwrap(),
            true,
        )
        .unwrap();

    let after = fixture.counters();
    assert_eq!(
        after.evaluations,
        before.evaluations + 2,
        "`ex.ready` and `ex.guarded`"
    );
    assert_eq!(
        after.operand_probes,
        before.operand_probes + 6,
        "`ex.ready` reads one operand and `ex.guarded` two: three reads when \
         each begins, and the same three re-proven when each completes"
    );
    assert_eq!(after.index_hits, before.index_hits + 2);
}

#[test]
fn a_false_operand_never_turns_a_non_current_operand_into_false() {
    let mut fixture = fixture();
    let unavailable = |fixture: &Fixture| {
        matches!(
            fixture.outcome("ex.guarded"),
            UiExpressionOutcome::Unavailable(_)
        )
    };
    assert!(unavailable(&fixture), "no frame carries the query operand");

    fixture.publish_pending();

    assert!(
        unavailable(&fixture),
        "`false && pending` is unavailable, never false: {:?}",
        fixture.outcome("ex.guarded")
    );
}

#[test]
fn a_denied_later_operand_outranks_an_unavailable_earlier_one() {
    let mut fixture = fixture();

    let denied = |fixture: &Fixture| {
        assert_eq!(
            fixture.outcome("ex.mixed"),
            UiExpressionOutcome::Denied(UiExpressionDenialReason::Upstream {
                operand: "z".into(),
                identity: "ex.ratio".into(),
            }),
            "`a` is unavailable and sorts first, but `z` is denied: no value can exist"
        );
    };
    denied(&fixture);
    fixture.publish_pending();
    denied(&fixture);
}

#[test]
fn presenting_an_unchanged_query_frame_evaluates_and_indexes_nothing() {
    let mut fixture = fixture();
    fixture.publish_pending();
    fixture.publish_status("ONLINE", 1);
    let before = fixture.counters();

    let outcome = fixture
        .shell
        .present_frame(100, 1)
        .unwrap_or_else(|_| panic!("the frame presents"));

    assert!(matches!(outcome, UiMountedFrameOutcome::Published(_)));
    let work = since(&fixture.shell.session, before);
    assert_eq!(
        work.operand_probes, 1,
        "the hook examined the one indexed projection slot, and nothing else"
    );
    assert_eq!(work.evaluations, 0);
    assert_eq!(work.index_hits, 0);
    assert_eq!(work.published_changes, 0);
    assert_eq!(work.suppressed_unchanged, 0);
}

/// Completes `completion` and asserts the owner refused it as stale and
/// changed no record of `identity`.
fn assert_refused_as_stale(
    fixture: &mut Fixture,
    completion: UiExpressionCompletion,
    identity: &str,
) {
    let retained = fixture
        .shell
        .session
        .expression_record(identity)
        .unwrap()
        .clone();
    let before = fixture.counters();

    let receipt = complete(&mut fixture.shell.session, completion);

    assert_eq!(receipt, UiExpressionCompletionReceipt::Stale);
    let work = since(&fixture.shell.session, before);
    assert_eq!(work.stale_completions, 1);
    assert_eq!(work.published_changes + work.suppressed_unchanged, 0);
    let record = fixture.shell.session.expression_record(identity).unwrap();
    assert_eq!(record.outcome(), retained.outcome());
    assert_eq!(record.outcome_revision(), retained.outcome_revision());
}

#[test]
fn a_ticket_held_across_a_published_query_change_completes_stale() {
    let mut fixture = fixture();
    fixture.publish_pending();
    fixture.publish_status("ONLINE", 1);
    let completion = evaluate(&mut fixture.shell.session, "ex.online");
    fixture.publish_status("OFFLINE", 2);
    assert_eq!(
        fixture.outcome("ex.online"),
        UiExpressionOutcome::Condition(false)
    );

    assert_refused_as_stale(&mut fixture, completion, "ex.online");
}

#[test]
fn a_ticket_that_read_an_absent_query_completes_stale_once_a_frame_carries_it() {
    let mut fixture = fixture();
    let completion = evaluate(&mut fixture.shell.session, "ex.online");
    fixture.publish_pending();

    assert_refused_as_stale(&mut fixture, completion, "ex.online");
}

#[test]
fn republishing_the_same_value_evaluates_only_the_direct_readers() {
    let mut fixture = fixture();
    fixture.publish_pending();
    fixture.publish_status("ONLINE", 1);
    let before = fixture.counters();
    let echoed = fixture
        .shell
        .session
        .expression_result("ex.echoed")
        .unwrap();

    fixture.publish_status("ONLINE", 2);

    let work = since(&fixture.shell.session, before);
    assert_eq!(
        work.evaluations, 3,
        "`ex.online`, `ex.echo` and `ex.guarded` read the status directly"
    );
    assert_eq!(
        work.published_changes, 0,
        "the same value at a new revision changes no outcome"
    );
    assert_eq!(
        work.settled_without_evaluation, 1,
        "`ex.mixed` reads it too, and settles without a kernel run: `z` is denied"
    );
    assert_eq!(
        work.suppressed_unchanged, 4,
        "each of the four direct readers re-settled to the outcome it had"
    );
    assert!(
        fixture.shell.session.is_current_expression_result(&echoed),
        "`ex.echoed` was never evaluated, so its result is untouched"
    );
    assert_eq!(
        fixture.outcome("ex.echoed"),
        UiExpressionOutcome::Condition(true)
    );
}
