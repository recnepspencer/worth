//! Payload fields read from expressions: a `derived text`, a condition and a
//! `derived integer` project the result their owner retains now, record its
//! outcome revision, withhold with the owner's own posture, refuse a value
//! their field cannot carry, and stay pull-only.

use super::*;
use crate::facade::expression::{
    UiExpressionCurrentValue, UiExpressionDenialReason, UiExpressionOutcome,
    UiExpressionStaleReason, UiExpressionUnavailableReason, UiExpressionWithholding as Withholding,
};

#[path = "expression_payload_world.rs"]
mod payload_fixture;
use payload_fixture::{
    name_fact, payload_session, payload_submission, payload_world, projected, switch_fact,
    tally_fact, Label, PayloadIntent, PayloadWorld, CAPTION, COUNT, ENABLED, LABEL, LABEL_BUDGET,
};

#[path = "expression_payload_succession_tests.rs"]
mod succession_tests;

fn prepared(
    world: &mut PayloadWorld,
    sequence: u64,
) -> Result<UiPreparedIntentPayload, UiIntentPayloadStop> {
    let route = activation_route(&mut world.session, world.surface, sequence);
    world.session.prepare_intent_payload(route)
}

fn set_name(world: &mut PayloadWorld, value: &str) {
    world
        .session
        .update_intent_text_fact(&name_fact(), value)
        .unwrap();
}

fn set_switch(world: &mut PayloadWorld, value: bool) {
    world
        .session
        .update_intent_boolean_fact(&switch_fact(), value)
        .unwrap();
}

fn set_tally(world: &mut PayloadWorld, value: u64) {
    world
        .session
        .update_intent_unsigned64_fact(&tally_fact(), value)
        .unwrap();
}

fn withheld(field: &'static str, expression: &str, posture: Withholding) -> UiIntentPayloadStop {
    UiIntentPayloadStop::ExpressionWithheld {
        field,
        expression: expression.into(),
        posture,
    }
}

#[test]
fn a_payload_reads_the_derived_text_and_the_condition_its_expressions_hold_now() {
    let mut world = payload_world(Label::Derived(LABEL));
    let payload = prepared(&mut world, 1).unwrap();

    assert_eq!(projected(), Some(("alpha".into(), true, 1)));
    let basis = payload.input_basis();
    let cost = basis.cost();
    assert_eq!(cost.declared_fields(), 3);
    assert_eq!(cost.expression_inputs_read(), 3);
    assert_eq!(cost.query_inputs_read(), 0);
    assert_eq!(cost.application_inputs_read(), 0);
    assert_eq!(cost.admitted_utf8_bytes(), "alpha".len());
    assert_eq!(payload.retained_owner_reference_count(), 4);
    let revisions = basis
        .owner_revisions()
        .iter()
        .map(|revision| match revision {
            UiIntentInputOwnerRevision::Expression(revision) => (
                revision.field().stable_name(),
                revision.identity().to_owned(),
                revision.outcome_revision(),
            ),
            other => panic!("an expression source records an expression revision: {other:?}"),
        })
        .collect::<Vec<_>>();
    let revision = |identity| {
        world
            .session
            .expression_record(identity)
            .unwrap()
            .outcome_revision()
    };
    assert_eq!(
        revisions,
        [
            ("label", LABEL.to_owned(), revision(LABEL)),
            ("enabled", ENABLED.to_owned(), revision(ENABLED)),
            ("count", COUNT.to_owned(), revision(COUNT)),
        ]
    );
    drop(payload);

    set_switch(&mut world, false);
    set_name(&mut world, "beta");
    set_tally(&mut world, 9);
    let _ = prepared(&mut world, 3).unwrap();
    assert_eq!(
        projected(),
        Some(("beta".into(), false, 7)),
        "a condition that holds `false` now projects `false`"
    );
    let _ = world.session.shutdown();
}

#[test]
fn the_admission_trace_names_the_expression_revisions_the_payload_read() {
    // Each world admits once: an admitted intent occupies its route.
    let traced = |switch: bool| {
        let mut world = payload_world(Label::Derived(LABEL));
        if !switch {
            set_switch(&mut world, false);
        }
        let payload = prepared(&mut world, 1).unwrap();
        let reference = payload.input_basis().evidence_reference().unwrap();
        let proof = world.session.evaluate_intent_operability(payload);
        assert!(matches!(
            admit(&mut world, proof),
            UiIntentAdmissionDecision::Admitted(_)
        ));
        let worth_ui_inspection::UiIntentEvidenceLookup::Found(trace) =
            world.session.lookup_intent_causal_trace(reference)
        else {
            panic!("an admitted payload leaves its causal trace");
        };
        let label = world
            .session
            .expression_record(LABEL)
            .unwrap()
            .outcome_revision();
        let _ = world.session.shutdown();
        (trace.payload().unwrap(), label)
    };
    let (first, label) = traced(true);
    assert_eq!(first.owner_revision_count(), 3);
    assert_eq!(first.primary_owner_revision(), Some(label));
    assert_eq!(first.admitted_utf8_bytes(), "alpha".len());

    let (second, _) = traced(false);
    assert_eq!(
        second.primary_owner_revision(),
        Some(label),
        "the label did not change"
    );
    assert_ne!(
        second.owner_revision_digest(),
        first.owner_revision_digest(),
        "the condition's new outcome revision is part of the digest"
    );
}

#[test]
fn a_withheld_expression_stops_the_payload_with_its_own_posture_never_a_default() {
    let stale = UiExpressionOutcome::Stale {
        reason: UiExpressionStaleReason::ProjectionRetained {
            operand: "n".into(),
            kind: worth_ui_query_binding::UiProjectionRetainedActivityKind::Revalidating,
        },
        last_current: Some(UiExpressionCurrentValue::Condition(true)),
    };
    let denied = UiExpressionOutcome::Denied(UiExpressionDenialReason::OperandShapeMismatch {
        operand: "n".into(),
    });
    let unavailable =
        UiExpressionOutcome::Unavailable(UiExpressionUnavailableReason::ApplicationFactAbsent {
            operand: "n".into(),
            fact: "absent".into(),
        });
    for (field, identity) in [("label", LABEL), ("enabled", ENABLED), ("count", COUNT)] {
        let mut world = payload_world(Label::Derived(LABEL));
        let mut cases = vec![
            (denied.clone(), Withholding::Denied),
            (unavailable.clone(), Withholding::Unavailable),
            (stale.clone(), Withholding::Stale),
        ];
        if identity != ENABLED {
            // A derived value reads only its own type; any other is denied.
            cases.push((UiExpressionOutcome::Condition(true), Withholding::Denied));
        }
        for (sequence, (outcome, posture)) in (1u64..).step_by(2).zip(cases) {
            world
                .session
                .expressions
                .retain_outcome(identity, outcome.clone());
            let Err(stop) = prepared(&mut world, sequence) else {
                panic!("a withheld {identity} projects no payload: {outcome:?}");
            };
            assert_eq!(stop, withheld(field, identity, posture), "{outcome:?}");
        }
        let _ = world.session.shutdown();
    }
}

#[test]
fn an_owner_holding_no_result_for_the_generation_leaves_the_payload_stale() {
    let mut world = payload_world(Label::Derived(LABEL));
    world.session.expressions = crate::runtime::expression::UiExpressionRuntimeState::unsettled(
        std::sync::Arc::clone(
            world
                .session
                .application
                .prepared_authority()
                .expression_catalog(),
        ),
        world.session.active_generation_identity().clone(),
        &world.session.mounted,
    );

    let Err(stop) = prepared(&mut world, 1) else {
        panic!("an unsettled owner holds no result to project");
    };
    assert_eq!(stop, withheld("label", LABEL, Withholding::Stale));
    let _ = world.session.shutdown();
}

#[test]
fn a_derived_text_over_its_field_budget_is_refused() {
    let mut world = payload_world(Label::Derived(LABEL));
    set_name(&mut world, "overbudget");

    let Err(stop) = prepared(&mut world, 1) else {
        panic!("a derived text over its field budget projects no payload");
    };
    assert_eq!(
        stop,
        UiIntentPayloadStop::TextByteBudgetExceeded {
            field: "label",
            observed: "overbudget".len(),
            maximum: LABEL_BUDGET,
        }
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_derived_integer_its_field_cannot_carry_is_refused_never_clamped() {
    let mut world = payload_world(Label::Derived(LABEL));
    set_tally(&mut world, 2);
    let _ = prepared(&mut world, 1).unwrap();
    assert_eq!(projected().map(|(_, _, count)| count), Some(0));

    set_tally(&mut world, 1);
    let Err(stop) = prepared(&mut world, 3) else {
        panic!("a negative count projects no payload");
    };
    assert_eq!(
        stop,
        UiIntentPayloadStop::DerivedIntegerOutOfRange {
            field: "count",
            expression: COUNT.into(),
            observed: -1,
        }
    );

    set_tally(&mut world, u64::MAX);
    let Err(stop) = prepared(&mut world, 5) else {
        panic!("a tally no Int64 holds projects no payload");
    };
    assert_eq!(
        stop,
        withheld("count", COUNT, Withholding::Denied),
        "the exact cast denies the expression; nothing wraps"
    );
    let _ = world.session.shutdown();
}

#[test]
fn a_payload_only_expression_change_reobserves_nothing() {
    let mut world = payload_world(Label::Derived(LABEL));
    let (target, operable) = activate(&mut world.session, world.surface, 1);
    assert_eq!(operable.primary_cause(), None);
    let attempts = reobservations(&world.session);
    let work = world.session.expression_work_counters();

    set_name(&mut world, "beta");
    set_switch(&mut world, false);

    assert_eq!(
        world.session.expression_work_counters().published_changes,
        work.published_changes + 3,
        "both payload expressions reading the name, and the condition, changed"
    );
    assert_eq!(
        reobservations(&world.session),
        attempts,
        "payload expressions are pulled at preparation, never pushed"
    );
    let (again, decision) = activate(&mut world.session, world.surface, 3);
    assert_eq!((again, decision), (target, operable));
    let _ = world.session.shutdown();
}

#[test]
fn a_changed_payload_source_is_a_different_intent_contract() {
    let catalogs = [
        Label::Constant,
        Label::Derived(LABEL),
        Label::Derived(CAPTION),
        Label::Derived(LABEL),
    ]
    .map(|label| payload_session(label).0);
    let compare = |left: usize, right: usize| {
        catalogs[left]
            .application
            .prepared_authority()
            .intent_catalog()
            .compare_semantic_contract(
                catalogs[right]
                    .application
                    .prepared_authority()
                    .intent_catalog(),
            )
    };
    use crate::declaration::UiIntentCatalogSemanticComparison as Comparison;
    let authority = |index: usize| catalogs[index].application.prepared_authority();
    assert!(
        authority(1)
            .expression_catalog()
            .installs_same_expressions_as(authority(2).expression_catalog()),
        "every catalog installs the same expressions, so only the intent contract tells the sources apart"
    );
    assert_eq!(compare(0, 1), Comparison::Different, "literal to derived");
    assert_eq!(
        compare(1, 2),
        Comparison::Different,
        "derived to another derived"
    );
    assert_eq!(
        compare(1, 3),
        Comparison::Equivalent,
        "the same derived source"
    );
    for session in catalogs {
        let _ = session.shutdown();
    }
}

/// Admits `proof` under the payload definition.
fn admit(
    world: &mut PayloadWorld,
    proof: UiIntentOperabilityOutcome,
) -> UiIntentAdmissionDecision<PayloadIntent> {
    world.session.admit_intent(
        UiIntentDefinition::<PayloadIntent>::application_effect(),
        proof,
    )
}

/// A current operability proof over a freshly prepared payload.
fn proof(world: &mut PayloadWorld, sequence: u64) -> UiIntentOperabilityOutcome {
    let payload = prepared(world, sequence).unwrap();
    let proof = world.session.evaluate_intent_operability(payload);
    assert!(matches!(proof, UiIntentOperabilityOutcome::Operable(_)));
    proof
}
