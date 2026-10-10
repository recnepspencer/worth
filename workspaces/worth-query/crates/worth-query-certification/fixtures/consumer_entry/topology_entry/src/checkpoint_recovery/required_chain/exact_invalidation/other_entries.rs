//! An entry the random sequence does not draw: a native Relational writer
//! that is not Query.

use super::*;

/// Writes `$field` of `$body` through an ordinary native Relational
/// transaction, bypassing every Query commit path.
macro_rules! native_write {
    ($application:expr, $scope:expr, $body:expr, $field:ident, $value:expr) => {{
        let branch = $application.current_world();
        let record = $application
            .on_branch(branch)
            .select()
            .unwrap()
            .resolve_entity(
                BodyKey::reference::<CheckpointSchema>(),
                $body,
                &$scope,
                primary_graph::WorthQueryPrincipalResolutionMode::Certification,
            )
            .unwrap()
            .relational_record_identity_parts();
        $application.publish_native_field_write_for_test(
            branch,
            record,
            $field::reference::<CheckpointSchema>(),
            length($value),
            &$scope,
        );
    }};
}

/// No demand of the rings at `untouched` reaches a producer or decides.
fn judge_untouched(
    court: &Court<'_, '_, '_, '_>,
    rings: &mut [Ring],
    untouched: [usize; 2],
    at: &str,
) {
    for index in untouched {
        let (costs, decisions) = court.demand_ring(rings, index, at);
        assert!(
            costs.iter().all(Cost::is_free) && decisions == 0,
            "{at}: ring {index} was not written: {costs:?}, {decisions} decisions"
        );
    }
}

/// A commit Query did not make is delivered exactly and marks what the same
/// write through a declared operation marks: the written ring decides again
/// over the model, and no other ring is reached.
#[test]
fn a_native_writer_marks_what_a_declared_operation_marks() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    Reading::decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "before any native write");
    }
    let inexact = inexact_deliveries();

    let at = "after a native write of a root input";
    rings[0].a_y = 6;
    native_write!(application, scope, rings[0].key("a"), PositionY, 6);
    let ([a, b, ..], decisions) = court.demand_ring(&mut rings, 0, at);
    assert_eq!(
        (a.producer_contacts, b.producer_contacts, decisions > 0),
        (1, 1, true),
        "{at}: the root executes again and its consumer decides over the new output"
    );
    judge_untouched(&court, &mut rings, [1, 2], at);

    let at = "after a native write of a chain output";
    rings[1].b_length = 44;
    native_write!(application, scope, rings[1].key("b"), Length, 44);
    let ([a, _, c, _], decisions) = court.demand_ring(&mut rings, 1, at);
    assert_eq!(
        (a.producer_contacts, c.producer_contacts, decisions > 0),
        (0, 1, true),
        "{at}: the root is untouched and the last consumer decides over the written output"
    );
    judge_untouched(&court, &mut rings, [0, 2], at);
    assert_eq!(
        inexact_deliveries() - inexact,
        0,
        "{at}: every native commit is delivered exactly"
    );
}
