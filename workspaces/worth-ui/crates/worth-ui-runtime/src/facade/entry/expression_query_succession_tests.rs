use std::sync::Arc;

use worth_foundational::expression_api::ExpressionValue;

use super::active_application_session::expression_session_fixture::session_inputs;
use super::expression_query_fixture::{fixture, Fixture, DECLARATIONS};
use crate::facade::expression::{UiExpressionOutcome, UiExpressionUnavailableReason};
use crate::runtime::expression::UiExpressionRuntimeState;
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// `DECLARATIONS` behind a leading comment: the same meaning, new evidence.
fn commented() -> String {
    format!("// trivia\n{DECLARATIONS}")
}

/// A second expression owner activated on the session's current generation,
/// which the session never tells about a later change.
fn missed_owner(
    fixture: &Fixture,
) -> (
    UiExpressionRuntimeState,
    WorthUiActiveApplicationGenerationIdentity,
) {
    let session = &fixture.shell.session;
    let launched = session.active_generation_identity();
    let catalog = Arc::clone(
        session
            .application
            .prepared_authority()
            .expression_catalog(),
    );
    let missed = UiExpressionRuntimeState::activate(catalog, &session_inputs(session, &launched));
    (missed, launched)
}

fn absent() -> UiExpressionOutcome {
    UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ProjectionAbsent {
        operand: "p".into(),
    })
}

#[test]
fn following_re_proves_each_query_operand_and_rebuilds_the_readers_whose_frame_changed() {
    let mut fixture = fixture();
    let (mut read_absent, launched) = missed_owner(&fixture);
    let online = read_absent.catalog().slot_of("ex.online").unwrap();
    assert_eq!(read_absent.record(online).unwrap().outcome(), &absent());
    fixture.publish_pending();
    let (mut read_pending, _) = missed_owner(&fixture);
    assert!(matches!(
        read_pending.record(online).unwrap().outcome(),
        UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::Projection { .. })
    ));
    fixture.publish_status("ONLINE", 1);

    fixture.rebind_source(&commented());

    let session = &fixture.shell.session;
    let active = session.active_generation_identity();
    assert_ne!(active, launched, "the comment edit published a successor");
    let catalog = session
        .application
        .prepared_authority()
        .expression_catalog();
    for missed in [&mut read_absent, &mut read_pending] {
        let inputs = session_inputs(session, &active);
        let prepared = missed.prepare_succession(catalog, &inputs);
        let _rebuilt = missed.commit_succession(prepared, catalog, &inputs);
        for identity in ["ex.online", "ex.echo", "ex.echoed"] {
            let slot = missed.catalog().slot_of(identity).unwrap();
            let record = missed.record(slot).unwrap();
            assert_eq!(record.generation(), &active, "`{identity}`");
            assert_eq!(
                record.outcome(),
                session.expression_record(identity).unwrap().outcome(),
                "`{identity}` read an older frame, so the frame it now sees fails \
                 re-proof and it is rebuilt, never re-stamped"
            );
        }
        assert_eq!(
            missed.record(online).unwrap().outcome(),
            &UiExpressionOutcome::Condition(true)
        );
        let echo = missed.catalog().slot_of("ex.echo").unwrap();
        assert_eq!(
            missed.record(echo).unwrap().outcome(),
            &UiExpressionOutcome::Value(ExpressionValue::string("ONLINE"))
        );
    }
}

#[test]
fn an_owner_that_missed_a_generation_change_probes_no_published_frame() {
    let mut fixture = fixture();
    let (mut missed, launched) = missed_owner(&fixture);
    fixture.rebind_source(&commented());
    fixture.publish_pending();
    fixture.publish_status("ONLINE", 1);
    let session = &fixture.shell.session;
    let active = session.active_generation_identity();
    assert_ne!(active, launched, "the comment edit published a successor");
    let before = missed.counters();

    assert!(
        missed
            .invalidate_published_frame(&session_inputs(session, &active))
            .is_empty(),
        "an owner on a retired generation reports no changed condition"
    );

    assert_eq!(
        missed.counters(),
        before,
        "an owner on a retired generation probes and settles nothing"
    );
    let online = missed.catalog().slot_of("ex.online").unwrap();
    let record = missed.record(online).unwrap();
    assert_eq!(record.generation(), &launched);
    assert_eq!(record.outcome(), &absent());
}
