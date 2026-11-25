//! C-1: a successor round's creates against its chain's (#119 N2, N3, M-ID).
use super::*;

/// Rounds of the owned Birth route, each `(keys, value)`, optionally with
/// an authored stage writing the carrier after the head.
fn rounds(stage: bool, rounds: &[(&[&str], &str)]) -> Run {
    let ids = RefCell::new(Vec::new());
    let run = run(stage, &|world, execution, candidate| {
        let (keys, value) = rounds[0];
        birth_round(world, execution, candidate, keys, value)?;
        let meaning = world.meaning("owned");
        *ids.borrow_mut() = candidate
            .appended_output_entities(execution, "owned", &meaning)?
            .expect("the appended route")
            .to_vec();
        if stage {
            candidate.append_authored_stage(
                execution,
                world.carrier,
                &none(),
                vec![update(world.carrier, "stage-carrier")],
            )?;
        }
        for (keys, value) in &rounds[1..] {
            birth_round(world, execution, candidate, keys, value)?;
        }
        Ok(())
    })
    .unwrap();
    let head = ids.into_inner();
    assert!(!head.is_empty());
    run
}

#[test]
fn kept_dropped_and_new_keys_commit_the_one_round_answer() {
    for stage in [false, true] {
        let chain = rounds(stage, &[(&["a", "b"], "1"), (&["a", "c"], "2")]);
        let one = rounds(stage, &[(&["a", "c"], "2")]);
        assert_eq!(
            chain.observed, one.observed,
            "the same images, stage={stage}"
        );
        assert_eq!(
            chain.outputs.keys().collect::<Vec<_>>(),
            vec!["a", "c"],
            "b is never published"
        );
        // N3/M-ID: the kept key keeps the identity its head allocated.
        let head = rounds(stage, &[(&["a", "b"], "1")]);
        assert_eq!(
            chain.outputs["a"], head.outputs["a"],
            "a keeps its identity"
        );
        assert_ne!(
            chain.outputs["c"], head.outputs["b"],
            "c never reuses b's slot"
        );
    }
}

#[test]
fn a_successor_round_replays_its_plan_allocations_like_the_one_round_answer() {
    let refusal = |rounds_: &[(&[&str], &str)]| {
        let run = rounds(false, rounds_);
        let stop = run
            .world
            .replay(&ExecutionRequest::serial(&request()))
            .expect_err("#393");
        let detail = format!("{stop:?}");
        detail[detail.find("denial: ").unwrap()..]
            .split(" }")
            .next()
            .unwrap()
            .to_owned()
    };
    assert_eq!(
        refusal(&[(&["a", "b"], "1"), (&["a", "c"], "2")]),
        refusal(&[(&["a", "c"], "2")]),
        "every plan allocation replays; only ownership completion remains (#393)"
    );
}
