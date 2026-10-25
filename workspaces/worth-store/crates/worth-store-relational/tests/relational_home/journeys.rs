//! Ordinary home journeys: create, commit, close, reopen, continue.

use worth_execution::ExecutionRequest;

use super::durable_run::{apply, assert_holds, open, steps_after, steps_before};
use super::workload::{self, Script};

const SEED: u64 = 0x5eed_0201;
const COMMITS: usize = 18;

#[test]
fn the_reference_script_is_deterministic() {
    let first = Script::generate(SEED, COMMITS, 5);
    let second = Script::generate(SEED, COMMITS, 5);
    assert_eq!(first.states, second.states);
    assert_eq!(first.artifacts, second.artifacts);
    assert_eq!(first.transitions, second.transitions);
    assert_eq!(first.prefixes, second.prefixes);
}

#[test]
fn a_home_recovers_exactly_what_it_committed_and_keeps_going() {
    let script = Script::generate(SEED, COMMITS, 5);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let half = COMMITS / 2;
    let home = open(&location, &execution).expect("a new home opens");
    assert_eq!(apply(&home, &execution, steps_before(&script, half)), half);
    let first_half = workload::artifacts_of(home.runtime());
    assert_holds(&home, &script, &first_half);
    home.close();

    let home = open(&location, &execution).expect("the home reopens");
    assert_holds(&home, &script, &first_half);
    let applied = apply(&home, &execution, steps_after(&script, half));
    assert_eq!(applied, COMMITS - half);
    let everything = workload::artifacts_of(home.runtime());
    assert_eq!(&everything[..half], &first_half[..]);
    assert_holds(&home, &script, &everything);
    drop(home);

    let home = open(&location, &execution).expect("an abandoned home recovers");
    assert_holds(&home, &script, &everything);
    home.close();
}

#[test]
fn a_fork_committed_after_a_checkpoint_recovers() {
    let script = Script::generate(SEED, COMMITS, 5);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let home = open(&location, &execution).expect("a new home opens");
    assert_eq!(apply(&home, &execution, &script.steps), COMMITS);
    let everything = workload::artifacts_of(home.runtime());
    drop(home);
    let home = open(&location, &execution).expect("the home recovers");
    assert_holds(&home, &script, &everything);
    home.close();
}

#[test]
fn the_reference_script_generates_for_every_seed() {
    for seed in 0x5eed_0400..0x5eed_0440_u64 {
        let outcome = std::panic::catch_unwind(|| Script::generate(seed, COMMITS, 5));
        assert!(outcome.is_ok(), "seed {seed:#x} does not generate");
    }
}
