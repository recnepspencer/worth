//! C-6: a superseded round leaves no entry behind (#119 M-E2).
use super::*;

/// A cancelled record and a record a later round reverts are absent from the
/// commit: no changed record, no patch, no deletion flag.
#[test]
fn a_cancelled_or_reverted_record_leaves_no_entry() {
    let run = |rounds: &[(&[&str], &str, bool)]| {
        run(false, &|world, execution, candidate| {
            for &(keys, value, carrier) in rounds {
                let mut batches = creates(keys, value);
                if carrier {
                    batches.push(update(world.carrier, &format!("carrier:{value}")));
                }
                let posture = birth(world, execution, keys);
                if candidate.continues_owned_route(0) {
                    successor(execution, candidate, posture, batches)?;
                } else {
                    candidate.append_publication(execution, posture, &none(), batches)?;
                }
            }
            Ok(())
        })
        .unwrap()
    };
    let chain = run(&[(&["a", "b"], "1", true), (&["a"], "2", false)]);
    let one = run(&[(&["a"], "2", false)]);
    assert_eq!(chain.observed, one.observed);
    let head = run(&[(&["a", "b"], "1", true)]);
    let cancelled = head.outputs["b"];
    let read = cold::facts(
        &chain.world.runtime,
        &ExecutionRequest::serial(&request()),
        &[chain.world.carrier],
    );
    let base = cold::facts(
        &one.world.runtime,
        &ExecutionRequest::serial(&request()),
        &[one.world.carrier],
    );
    assert_eq!(
        states(&read),
        states(&base),
        "the reverted carrier is exactly its basis image"
    );
    let snapshot = chain
        .world
        .runtime
        .snapshots()
        .snapshot_for_observation(&chain.world.basis().observation())
        .unwrap();
    let truth = chain
        .world
        .runtime
        .read_truth()
        .read_snapshot(&snapshot, &ExecutionRequest::serial(&request()))
        .unwrap()
        .unwrap();
    assert!(
        truth.get_entity(cancelled).is_none(),
        "the cancelled identity is never live"
    );
    chain
        .world
        .runtime
        .snapshots()
        .release_snapshot(&snapshot)
        .unwrap();
}
