//! C-12: a work ceiling anywhere in a successor round (retraction, guards,
//! completion, allocation ordering) is a typed stop, never a panic, and
//! publishes nothing.
use super::*;
use worth_execution::ExecutionWorkCeiling;

#[test]
fn a_work_ceiling_anywhere_in_a_successor_round_stops_typed() {
    let host = request();
    let execution = ExecutionRequest::serial(&host);
    let world = World::new(&execution, true, true);
    let attempt = |ceiling: Option<u64>| {
        // Route absences are read outside the ceiling: only the assembly's
        // Native work is bounded.
        let absences = RefCell::new(
            (0..3)
                .map(|_| world.absence(&execution, "owned"))
                .collect::<Vec<_>>(),
        );
        let posture = |keys: &[&str]| {
            RelationalCoupledProducerPosture::Publication(RequiredOutputPublication::Birth {
                admission: world.owner.admit_required_output("owned", [3; 32]),
                absence: absences.borrow_mut().pop(),
                meaning: world.meaning("owned"),
                roles: keys.iter().map(|key| created(key)).collect(),
            })
        };
        let assembly = assemble(|execution, candidate| {
            candidate.append_publication(
                execution,
                posture(&["x", "y"]),
                &none(),
                creates(&["x", "y"], "1"),
            )?;
            candidate.append_authored_stage(
                execution,
                world.carrier,
                &none(),
                vec![update(world.carrier, "stage")],
            )?;
            successor(
                execution,
                candidate,
                posture(&["y", "z"]),
                creates(&["y", "z"], "2"),
            )?;
            successor(
                execution,
                candidate,
                posture(&["x", "z"]),
                creates(&["x", "z"], "3"),
            )
        });
        execution
            .run(ExecutionWorkCeiling::new(ceiling), |_| {
                world.prepare_stages(&execution, &[world.carrier], &assembly)
            })
            .unwrap()
    };
    let (full, report) = attempt(None);
    full.expect("the unbounded assembly completes");
    let total = report.charged_work();
    let before = world.basis();
    let mut stopped = 0;
    for step in 0..32u64 {
        let (result, _) = attempt(Some(total * step / 32));
        if let Err(err) = result {
            assert!(
                matches!(err, TransactionCommitError::Execution { .. }),
                "a typed execution stop: {err:?}"
            );
            stopped += 1;
        }
        assert_eq!(world.basis(), before, "nothing published");
    }
    assert!(stopped > 0, "some ceiling stops the assembly");
}
