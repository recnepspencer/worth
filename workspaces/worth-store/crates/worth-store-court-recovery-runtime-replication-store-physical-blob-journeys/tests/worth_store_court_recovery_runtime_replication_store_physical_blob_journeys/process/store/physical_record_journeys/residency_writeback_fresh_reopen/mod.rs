#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::residency_writeback_fresh_reopen::*;

#[test]
fn ordinary_record_and_exact_writeback_survive_fresh_store_admission() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("store");
    let writer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_writer",
            &root,
            None,
        );
    let evidence = writer
        .lines()
        .find_map(|line| line.strip_prefix("PHYSICAL_WRITEBACK "))
        .expect("writer must publish digest and record locator");
    let (digest, locator) = evidence
        .split_once(' ')
        .expect("writer evidence must separate digest from locator");
    let observer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_observer",
            &root,
            Some(digest),
        );
    assert!(observer
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_OBSERVED"));
    let reopener =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "physical_writeback_reopener",
            &root,
            Some(locator),
        );
    assert!(reopener
        .lines()
        .any(|line| line == "PHYSICAL_WRITEBACK_REOPENED"));
}
