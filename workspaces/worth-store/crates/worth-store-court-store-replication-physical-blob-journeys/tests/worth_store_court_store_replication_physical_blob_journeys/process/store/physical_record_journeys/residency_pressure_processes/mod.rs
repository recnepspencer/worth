#[allow(unused_imports)]
use worth_store_testkit_store::physical_record_journeys::residency_pressure_processes::*;

#[test]
fn bounded_residency_survives_pressure_across_three_processes() {
    let parent = tempfile::tempdir().unwrap();
    let root = parent.path().join("bounded-residency");
    let locators = parent.path().join("locators.txt");
    let writer =
        worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "residency_pressure_writer",
            &root,
            Some(locators.to_str().unwrap()),
        );
    assert!(writer
        .lines()
        .any(|line| line == "RESIDENCY_PRESSURE writer-ok"));
    assert!(directory_bytes(&root) >= RESIDENT_BYTES * 8);

    for _ in 0..2 {
        let reader = worth_store_testkit_store::physical_record_journeys::child_process::invocation::run_child(
            "residency_pressure_reader",
            &root,
            Some(locators.to_str().unwrap()),
        );
        assert!(reader
            .lines()
            .any(|line| line == "RESIDENCY_PRESSURE reader-ok"));
    }
}
