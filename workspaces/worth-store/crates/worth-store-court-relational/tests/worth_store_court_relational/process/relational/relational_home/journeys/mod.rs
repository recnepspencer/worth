//! Ordinary home journeys: create, commit, close, reopen, continue.

#[allow(unused_imports)]
use crate::journeys::*;

use {
    crate::durable_run::apply, crate::durable_run::assert_holds, crate::durable_run::open,
    crate::durable_run::steps_after, crate::durable_run::steps_before,
};

#[test]
fn the_reference_script_is_deterministic() {
    let first = crate::workload::Script::generate(SEED, COMMITS, 5);
    let second = crate::workload::Script::generate(SEED, COMMITS, 5);
    assert_eq!(first.states, second.states);
    assert_eq!(first.artifacts, second.artifacts);
    assert_eq!(first.transitions, second.transitions);
    assert_eq!(first.prefixes, second.prefixes);
}

#[test]
fn a_home_recovers_exactly_what_it_committed_and_keeps_going() {
    let script = crate::workload::Script::generate(SEED, COMMITS, 5);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let half = COMMITS / 2;
    let home = open(&location, &execution).expect("a new home opens");
    assert_eq!(apply(&home, &execution, steps_before(&script, half)), half);
    let first_half = crate::workload::artifacts_of(home.runtime());
    assert_holds(&home, &script, &first_half);
    home.close();

    let home = open(&location, &execution).expect("the home reopens");
    assert_holds(&home, &script, &first_half);
    let applied = apply(&home, &execution, steps_after(&script, half));
    assert_eq!(applied, COMMITS - half);
    let everything = crate::workload::artifacts_of(home.runtime());
    assert_eq!(&everything[..half], &first_half[..]);
    assert_holds(&home, &script, &everything);
    drop(home);

    let home = open(&location, &execution).expect("an abandoned home recovers");
    assert_holds(&home, &script, &everything);
    home.close();
}

#[test]
fn a_fork_committed_after_a_checkpoint_recovers() {
    let script = crate::workload::Script::generate(SEED, COMMITS, 5);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let home = open(&location, &execution).expect("a new home opens");
    assert_eq!(apply(&home, &execution, &script.steps), COMMITS);
    let everything = crate::workload::artifacts_of(home.runtime());
    drop(home);
    let home = open(&location, &execution).expect("the home recovers");
    assert_holds(&home, &script, &everything);
    home.close();
}

#[test]
fn the_reference_script_generates_for_every_seed() {
    for seed in 0x5eed_0400..0x5eed_0440_u64 {
        let outcome =
            std::panic::catch_unwind(|| crate::workload::Script::generate(seed, COMMITS, 5));
        assert!(outcome.is_ok(), "seed {seed:#x} does not generate");
    }
}
