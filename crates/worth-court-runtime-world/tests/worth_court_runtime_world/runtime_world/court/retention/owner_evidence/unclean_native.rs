//! A kill between World open, Native effect, World occurrence and the
//! original's decision reopens to exactly the state its durable facts
//! determine (C1.1 and C1.2 bounded property).
use super::super::artifact_source::Fixture;
use super::*;
use proptest::{prelude::*, test_runner::FileFailurePersistence};
mod model;
mod refusals;
mod resolution;
use model::Model;

#[derive(Clone, Copy, Debug)]
enum Step {
    /// Open, Native effect and World occurrence are all durable.
    Acknowledged,
    /// The process dies after the durable open, before any owner effect.
    OpenOnly,
    /// The process dies after the durable Native effect, before the occurrence.
    EffectOnly,
    Reopen,
    /// Continue each recovered effect; release each effect-free original.
    Continue,
    /// Release every recovered original, effect-bearing or not.
    Release,
    /// The first original's decision write reports an indeterminate fate,
    /// then the process dies. `landed` says whether the write was durable.
    Undecided {
        landed: bool,
    },
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 48,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "proptest-regressions/unclean_native.txt",
        ))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn killed_world_publication_reopens_to_its_recorded_facts(
        steps in proptest::collection::vec(
            prop_oneof![
                Just(Step::Acknowledged),
                Just(Step::OpenOnly),
                Just(Step::EffectOnly),
                Just(Step::Reopen),
                Just(Step::Continue),
                Just(Step::Release),
                any::<bool>().prop_map(|landed| Step::Undecided { landed }),
            ],
            1..=7,
        )
    ) {
        run(&steps);
    }
}

fn run(steps: &[Step]) {
    let funding = crate::test_execution_policy::serial_request();
    let request = ExecutionRequest::serial(&funding);
    let mut fixture = Fixture::journal(&request);
    match fixture.offer(&request, "primary") {
        WorldDurableBootstrapOutcome::Performed(_) => {}
        other => panic!("real bootstrap: {other:?}"),
    }
    let mut model = Model::default();
    for step in steps.iter().copied() {
        let unresolved = !model.originals.is_empty();
        match step {
            Step::Reopen => fixture = model.reopen(fixture, &request),
            Step::Continue => model.decide_all(&fixture, &request, false),
            Step::Release => model.decide_all(&fixture, &request, true),
            Step::Undecided { landed } if unresolved => {
                model.undecided(&fixture, &request, landed);
                fixture = model.reopen(fixture, &request);
            }
            // Fresh work waits until every recovered original is decided.
            _ if unresolved => {
                model.assert_fenced(&fixture);
                model.decide_all(&fixture, &request, false)
            }
            Step::Undecided { .. } => {
                fixture = model.fresh(fixture, &request, Step::Acknowledged);
            }
            kind => fixture = model.fresh(fixture, &request, kind),
        }
    }
    model.decide_all(&fixture, &request, false);
    // The next fresh identities lie beyond every issued one, unfinished opens included.
    let fixture = model.fresh(fixture, &request, Step::Acknowledged);
    let fixture = model.reopen(fixture, &request);
    fixture.finish(&request);
}

fn root(fixture: &Fixture) -> ProductBranchObservation {
    fixture
        .world
        .observation_port()
        .observe_root_product_branch()
        .unwrap()
}

fn maximum() -> std::num::NonZeroUsize {
    std::num::NonZeroUsize::new(16).unwrap()
}
