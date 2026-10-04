//! Entries the random sequence does not draw: a native Relational writer that
//! is not Query, and a demand at an observation older than the latest marks.

use super::*;

/// Writes `$field` of `$body` through an ordinary native Relational
/// transaction, bypassing every Query commit path.
#[cfg(feature = "test-output-delivery-faults")]
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
#[cfg(feature = "test-output-delivery-faults")]
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
#[cfg(feature = "test-output-delivery-faults")]
#[test]
fn a_native_writer_marks_what_a_declared_operation_marks() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f00);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    take_all_decisions();
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

/// A demand started at a retained observation settles at no cost while that
/// observation is the head. Once a commit moves the head it stops
/// `Superseded` at its start, whether or not the commit wrote what it reads:
/// it reaches no producer and decides nothing, and its caller demands again
/// at the head.
#[test]
fn a_demand_at_an_older_observation_stops_superseded() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None, ring_world::seed::<3>, Retained::AMPLE);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let court = Court::new(&application, &request, 0x9176_3f40);
    let mut rings: Vec<Ring> = (0..3).map(Ring::seeded).collect();
    take_all_decisions();
    for index in 0..rings.len() {
        court.demand_ring(&mut rings, index, "before the edit");
    }
    let retained = court.request.retain_read().unwrap();
    let superseded = |stop: &WorthQueryApplicationOutputDemandDenial| {
        matches!(
            stop,
            WorthQueryApplicationOutputDemandDenial::Demand(denial)
                if denial.kind() == WorthQueryOutputDemandDenialKind::Superseded
        )
    };

    let at = "while the retained observation is the head";
    let mut current = Open {
        body: rings[0].key("a"),
        demand: court
            .request
            .at(&retained)
            .demand(PlanarOutputDemand::new(rings[0].key("a")))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(court.application)
            .unwrap(),
        settled: false,
    };
    let cost = settled!(court, current, at);
    assert!(
        cost.is_free() && judge_decisions(&mut rings, at) == 0,
        "{at}: the retained demand settles on the clean output: {cost:?}"
    );
    drop(current);

    let at = "after an edit moved the head";
    rings[0].a_y = 6;
    court.write_y(&rings[0].key("a"), 6, at);
    // Ring 0 took the edit; ring 1 did not.
    for ring in [&rings[0], &rings[1]] {
        let older = court.request.at(&retained);
        let root = older
            .demand(PlanarOutputDemand::new(ring.key("a")))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(court.application)
            .map(drop);
        let consumer = older
            .demand(ChainDemand(ring.key("b")))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                court.application,
            )
            .map(drop);
        assert!(
            [&root, &consumer]
                .iter()
                .all(|start| start.as_ref().is_err_and(superseded)),
            "{at}: ring {} is not demanded at the older observation: {root:?}, {consumer:?}",
            ring.index
        );
    }
    assert_eq!(
        take_all_decisions().len(),
        0,
        "{at}: a superseded start decides nothing"
    );
    // The stops left nothing behind: at the head, the edited ring decides
    // over the model and the others cost nothing.
    let ([a, ..], decisions) = court.demand_ring(&mut rings, 0, at);
    assert!(
        a.producer_contacts == 1 && decisions > 0,
        "{at}: the edited root executes again at the head: {a:?}"
    );
    for index in 1..rings.len() {
        let (costs, decisions) = court.demand_ring(&mut rings, index, at);
        assert!(
            costs.iter().all(Cost::is_free) && decisions == 0,
            "{at}: ring {index} was not edited: {costs:?}"
        );
    }
}
