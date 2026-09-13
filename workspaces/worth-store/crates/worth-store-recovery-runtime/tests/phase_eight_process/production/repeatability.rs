use super::super::comparison;
use super::harness::ProcessWorld;

#[test]
fn independent_observers_converge_and_recovery_runtime_identity_changes() {
    let world = ProcessWorld::start("candidate-publication", 0, 1);
    let first_runtime = world.recover("repeat-first");
    let first_observer = world.observe("repeat-first");
    let second_observer = world.observe("repeat-second");
    comparison::compare_independent_observers(&first_observer.report, &second_observer.report)
        .expect("independent observer runs must converge on rich evidence");

    let second_runtime = world.recover("repeat-second");
    assert_ne!(first_runtime.process_id, second_runtime.process_id);
    assert_eq!(first_runtime.marker.store, second_runtime.marker.store);
    assert_eq!(
        first_runtime.marker.root_generation,
        second_runtime.marker.root_generation
    );
    assert_ne!(first_runtime.marker.runtime, second_runtime.marker.runtime);
    assert_eq!(
        first_runtime.report.counters(),
        second_runtime.report.counters()
    );
}
