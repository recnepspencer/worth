//! PR C R3, Native half: `settle_output_input` settles a consumer only on the
//! route it actually retained, compared against the meaning it retained from
//! before the chain folded, never the folded part against itself.
use super::*;
use std::cell::Cell;

fn second(world: &World, execution: &ExecutionRequest<'_, '_>) -> RelationalCoupledProducerPosture {
    RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
        admission: world.owner.admit_required_output("second", [5; 32]),
        absence: Some(world.absence(execution, "second")),
        meaning: world.meaning("second"),
        roles: vec![created("second-output")],
    })
}

/// The `owned` head round, a `second` consumer that retains it or not, then
/// an equal successor round of `owned`. Returns the settlement answer.
fn settle(retained: bool) -> Result<bool, String> {
    let host = request();
    let execution = &ExecutionRequest::serial(&host);
    let world = World::new(execution, true, true);
    let answer = Cell::new(None);
    let assembly = assemble(|execution, candidate| {
        let owned = world.meaning("owned");
        candidate.append_publication(
            execution,
            birth(&world, execution, &["a"]),
            &none(),
            creates(&["a"], "1"),
        )?;
        let old_pin = candidate.pin_source(execution)?;
        let old_parent = candidate
            .applied_output_input(execution, &owned)?
            .expect("the head round's input");
        candidate.append_publication(
            execution,
            second(&world, execution),
            &none(),
            vec![create("second-output")],
        )?;
        let consumer = candidate
            .applied_output_input(execution, &world.meaning("second"))?
            .expect("the consumer's input");
        if retained {
            candidate.retain_output_inputs(execution, &consumer, &[&old_parent])?;
        }
        successor(
            execution,
            candidate,
            birth(&world, execution, &["a"]),
            creates(&["a"], "1"),
        )?;
        let replacement = candidate
            .applied_output_input(execution, &owned)?
            .expect("the folded chain's input");
        answer.set(Some(
            candidate
                .settle_output_input(execution, &consumer, &old_pin, &old_parent, &replacement)
                .expect("a settlement answer"),
        ));
        Ok(())
    });
    let prepared = world.prepare(execution, &assembly);
    answer
        .get()
        .ok_or_else(|| format!("assembly stopped before settlement: {:?}", prepared.err()))
}

/// The consumer retained the head round, and the folded chain publishes the
/// same roles and payload: it settles to the chain.
#[test]
fn a_consumer_of_the_head_round_settles_on_an_equal_fold() {
    assert_eq!(settle(true), Ok(true));
}

/// The consumer never retained the route, so there is nothing to settle:
/// comparing the folded part with itself must not report a settlement.
#[test]
fn a_consumer_that_never_retained_the_parent_does_not_settle() {
    assert_eq!(settle(false), Ok(false));
}
